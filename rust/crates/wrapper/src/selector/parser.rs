#![allow(clippy::type_complexity)]

use chumsky::extra;
use chumsky::prelude::*;
use regex::Regex;

use super::ast::*;
use super::lexer::Token;

pub type ParserError<'src> = extra::Err<Simple<'src, Token>>;

enum CompoundItem {
    Attr(AttrSelector),
    Pseudo(PseudoClassSelector),
}

#[derive(Clone)]
enum CompoundRole {
    Any,
    Role(String),
}

enum PseudoArg {
    Int(i32),
    Str(String),
    Selector(Selector),
}

pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Selector, ParserError<'src>> {
    let selector = recursive(|selector| {
        let ws0 = any()
            .filter(|t: &Token| matches!(t, Token::Whitespace))
            .repeated()
            .ignored();

        let ws1 = any()
            .filter(|t: &Token| matches!(t, Token::Whitespace))
            .repeated()
            .at_least(1)
            .ignored();

        let int_val = select! { Token::Int(i) => i };

        let ident_val = select! { Token::Ident(s) => s };

        let string_val = select! { Token::String(s) => {
            let inner = &s[1..s.len() - 1];
            let mut out = String::with_capacity(inner.len());
            let mut chars = inner.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('"') => out.push('"'),
                        Some('\'') => out.push('\''),
                        Some('\\') => out.push('\\'),
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some(c) => out.push(c),
                        None => {}
                    }
                } else {
                    out.push(c);
                }
            }
            out
        }};

        let string_op = choice((
            just(Token::Eq).to(StringOp::Eq),
            just(Token::TildeEq).to(StringOp::ContainsWord),
            just(Token::CaretEq).to(StringOp::Starts),
            just(Token::DolarEq).to(StringOp::Ends),
            just(Token::StarEq).to(StringOp::Contains),
        ));

        let int_op = choice((
            just(Token::EqEq).to(IntOp::Eq),
            just(Token::Ne).to(IntOp::Ne),
            just(Token::Le).to(IntOp::Le),
            just(Token::Ge).to(IntOp::Ge),
            just(Token::Lt).to(IntOp::Lt),
            just(Token::Gt).to(IntOp::Gt),
        ));

        let flags = ident_val.or_not().try_map(|name, span| {
            let mut ci = false;
            let mut re = false;
            if let Some(name) = name {
                for c in name.chars() {
                    match c {
                        'i' => ci = true,
                        'r' => re = true,
                        _ => return Err(Simple::new(None, span)),
                    }
                }
            }
            Ok((
                AttrFlags {
                    case_insensitive: ci,
                },
                re,
            ))
        });

        let str_attr_name = ident_val.try_map(|name, span| match name.as_str() {
            "name" => Ok(StrAttrName::Name),
            "description" => Ok(StrAttrName::Description),
            "states" => Ok(StrAttrName::States),
            "states_en_us" => Ok(StrAttrName::StatesEnUs),
            "text" => Ok(StrAttrName::Text),
            "actions" => Ok(StrAttrName::Actions),
            _ => Err(Simple::new(None, span)),
        });

        let str_attr = str_attr_name
            .then_ignore(ws0)
            .then(string_op)
            .then_ignore(ws0)
            .then(string_val)
            .then_ignore(ws0)
            .then(flags)
            .try_map(
                |(((name, op), value), (flags, re)): (
                    ((StrAttrName, StringOp), String),
                    (AttrFlags, bool),
                ),
                 span| {
                    let value = if re {
                        let Ok(regex) = Regex::new(&value) else {
                            return Err(Simple::new(None, span));
                        };
                        StrMatcher::Regex(regex)
                    } else {
                        StrMatcher::Plain(value)
                    };
                    Ok(AttrSelector::Str {
                        name,
                        op,
                        value,
                        flags,
                    })
                },
            );

        let int_attr_name = ident_val.try_map(|name, span| match name.as_str() {
            "x" => Ok(IntAttrName::X),
            "y" => Ok(IntAttrName::Y),
            "width" => Ok(IntAttrName::Width),
            "height" => Ok(IntAttrName::Height),
            "children_count" => Ok(IntAttrName::ChildrenCount),
            "depth" => Ok(IntAttrName::Depth),
            _ => Err(Simple::new(None, span)),
        });

        let int_attr = int_attr_name
            .then_ignore(ws0)
            .then(int_op)
            .then_ignore(ws0)
            .then(int_val)
            .map(
                |((name, op), value): ((IntAttrName, IntOp), i32)| AttrSelector::Int {
                    name,
                    op,
                    value: Some(value),
                },
            );

        let bool_attr_name = ident_val.try_map(|name, span| match name.as_str() {
            "accessible_action" => Ok(BoolAttrName::AccessibleAction),
            "accessible_text" => Ok(BoolAttrName::AccessibleText),
            "accessible_selection" => Ok(BoolAttrName::AccessibleSelection),
            _ => Err(Simple::new(None, span)),
        });

        let bool_attr = bool_attr_name.map(|name| AttrSelector::Bool { name });

        let attr_selector = just(Token::LBracket)
            .ignore_then(ws0)
            .ignore_then(choice((int_attr, str_attr, bool_attr)))
            .then_ignore(ws0)
            .then_ignore(just(Token::RBracket));

        let pseudo_arg = choice((
            int_val.map(PseudoArg::Int),
            ident_val.map(PseudoArg::Str),
            selector.map(PseudoArg::Selector),
        ));

        let pseudo_class_selector = just(Token::Colon)
            .ignore_then(ident_val)
            .then(
                just(Token::LParen)
                    .ignore_then(ws0)
                    .ignore_then(pseudo_arg)
                    .then_ignore(ws0)
                    .then_ignore(just(Token::RParen))
                    .or_not(),
            )
            .try_map(
                |(name, arg): (String, Option<PseudoArg>), span| match name.as_str() {
                    "scope" => match arg {
                        None => Ok(PseudoClassSelector::Scope),
                        _ => Err(Simple::new(None, span)),
                    },
                    "has" => match arg {
                        Some(PseudoArg::Selector(mut s)) => {
                            for complex in s.alternatives.iter_mut() {
                                if !complex.is_relative()
                                    && !complex
                                        .last
                                        .pseudo_classes
                                        .contains(&PseudoClassSelector::Scope)
                                {
                                    complex.head = Some(Combinator::Descendant)
                                }
                            }
                            Ok(PseudoClassSelector::Has(Box::new(s)))
                        }
                        _ => Err(Simple::new(None, span)),
                    },
                    "not" => match arg {
                        Some(PseudoArg::Selector(s)) => {
                            if s.alternatives.iter().any(ComplexSelector::is_relative) {
                                Err(Simple::new(None, span))
                            } else {
                                Ok(PseudoClassSelector::Not(Box::new(s)))
                            }
                        }
                        _ => Err(Simple::new(None, span)),
                    },
                    "require-state" => match arg {
                        Some(PseudoArg::Str(s)) => Ok(PseudoClassSelector::RequireState(s)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "exclude-state" => match arg {
                        Some(PseudoArg::Str(s)) => Ok(PseudoClassSelector::ExcludeState(s)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-child" => match arg {
                        Some(PseudoArg::Int(n)) => Ok(PseudoClassSelector::NthChild(n)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-last-child" => match arg {
                        Some(PseudoArg::Int(n)) => Ok(PseudoClassSelector::NthLastChild(n)),
                        _ => Err(Simple::new(None, span)),
                    },
                    _ => Err(Simple::new(None, span)),
                },
            );

        let compound_item = choice((
            attr_selector.map(CompoundItem::Attr),
            pseudo_class_selector.map(CompoundItem::Pseudo),
        ));

        let role = choice((
            just(Token::Star).to(CompoundRole::Any),
            ident_val.map(CompoundRole::Role),
        ));

        let compound_selector = role
            .or_not()
            .then(compound_item.clone().repeated().collect::<Vec<_>>())
            .try_map(|(role, items), span| {
                if role.is_none() && items.is_empty() {
                    Err(Simple::new(None, span))
                } else {
                    let mut attrs = Vec::new();
                    let mut pseudo_classes = Vec::new();
                    for item in items {
                        match item {
                            CompoundItem::Attr(a) => attrs.push(a),
                            CompoundItem::Pseudo(p) => pseudo_classes.push(p),
                        }
                    }
                    Ok(CompoundSelector {
                        role: match role {
                            Some(CompoundRole::Role(name)) => Some(name),
                            _ => None,
                        },
                        attrs,
                        pseudo_classes,
                    })
                }
            });

        let explicit_combinator = choice((
            just(Token::Gt).to(Combinator::Child),
            just(Token::Plus).to(Combinator::NextSibling),
            just(Token::Tilde).to(Combinator::SubsequentSibling),
        ));

        let combinator = ws0
            .ignore_then(explicit_combinator.clone())
            .then_ignore(ws0)
            .or(ws1.to(Combinator::Descendant));

        let complex_selector = combinator
            .clone()
            .or_not()
            .then(compound_selector.clone())
            .then(
                combinator
                    .then(compound_selector.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(
                |((head, first), tail): (
                    (Option<Combinator>, CompoundSelector),
                    Vec<(Combinator, CompoundSelector)>,
                )| {
                    let mut body = Vec::with_capacity(tail.len());
                    let mut current = first;

                    for (combinator, next) in tail.into_iter() {
                        body.push((current, combinator));
                        current = next;
                    }

                    body.reverse();

                    ComplexSelector {
                        head,
                        body,
                        last: current,
                    }
                },
            );

        complex_selector
            .separated_by(ws0.ignore_then(just(Token::Comma)).then_ignore(ws0))
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|alternatives| Selector { alternatives })
    });

    selector
        .try_map(|s, span| {
            if s.alternatives.iter().any(ComplexSelector::is_relative) {
                Err(Simple::new(None, span))
            } else {
                Ok(s)
            }
        })
        .then_ignore(end())
}

#[cfg(test)]
mod tests;
