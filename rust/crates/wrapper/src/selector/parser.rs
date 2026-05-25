#![allow(clippy::type_complexity)]

use chumsky::extra;
use chumsky::prelude::*;
use regex::Regex;

use super::ast::*;
use super::lexer::Token;

pub type ParserError<'src> = extra::Err<Rich<'src, Token>>;

enum CompoundItem {
    Attr(AttrSelector),
    Pseudo(PseudoClassSelector),
}

#[derive(Clone)]
enum CompoundRole {
    Any,
    Role(String),
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
                        _ => return Err(Rich::custom(span, "invalid flag, expected 'i' or 'r'")),
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
            _ => Err(Rich::custom(span, "unknown string attribute, expected 'name', 'description', 'states', 'states_en_us', 'text', or 'actions'")),
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
                            return Err(Rich::custom(span, "invalid regex pattern"));
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
            _ => Err(Rich::custom(span, "unknown integer attribute, expected 'x', 'y', 'width', 'height', 'children_count', or 'depth'")),
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
            _ => Err(Rich::custom(span, "unknown boolean attribute, expected 'accessible_action', 'accessible_text', or 'accessible_selection'")),
        });

        let bool_attr = bool_attr_name.map(|name| AttrSelector::Bool { name });

        let attr_selector = just(Token::LBracket)
            .ignore_then(ws0)
            .ignore_then(choice((int_attr, str_attr, bool_attr)))
            .then_ignore(ws0)
            .then_ignore(just(Token::RBracket));

        let ident_pseudo_arg = ident_val.delimited_by(
            just(Token::LParen).then_ignore(ws0),
            ws0.ignore_then(just(Token::RParen)),
        );
        let int_pseudo_arg = int_val.delimited_by(
            just(Token::LParen).then_ignore(ws0),
            ws0.ignore_then(just(Token::RParen)),
        );
        let selector_pseudo_arg = selector.delimited_by(
            just(Token::LParen).then_ignore(ws0),
            ws0.ignore_then(just(Token::RParen)),
        );

        let pseudo_class_selector = just(Token::Colon).ignore_then(choice((
            just(Token::Ident("scope".to_string())).map(|_| PseudoClassSelector::Scope),
            just(Token::Ident("has".to_string()))
                .then(selector_pseudo_arg.clone())
                .map(|(_, mut s): (_, Selector)| {
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
                    PseudoClassSelector::Has(Box::new(s))
                }),
            just(Token::Ident("not".to_string()))
                .then(selector_pseudo_arg.clone())
                .try_map(|(_, s): (_, Selector), span| {
                    if s.alternatives.iter().any(ComplexSelector::is_relative) {
                        Err(Rich::custom(
                            span,
                            "relative selectors are not allowed inside :not()",
                        ))
                    } else {
                        Ok(PseudoClassSelector::Not(Box::new(s)))
                    }
                }),
            just(Token::Ident("require-state".to_string()))
                .then(ident_pseudo_arg.clone())
                .map(|(_, state)| PseudoClassSelector::RequireState(state)),
            just(Token::Ident("exclude-state".to_string()))
                .then(ident_pseudo_arg.clone())
                .map(|(_, state)| PseudoClassSelector::ExcludeState(state)),
            just(Token::Ident("nth-child".to_string()))
                .then(int_pseudo_arg.clone())
                .map(|(_, n)| PseudoClassSelector::NthChild(n)),
            just(Token::Ident("nth-last-child".to_string()))
                .then(int_pseudo_arg.clone())
                .map(|(_, n)| PseudoClassSelector::NthLastChild(n)),
        )));

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
                    Err(Rich::custom(
                        span,
                        "expected a role or at least one attribute or pseudo-class",
                    ))
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
                Err(Rich::custom(
                    span,
                    "relative selectors are not allowed at the top level",
                ))
            } else {
                Ok(s)
            }
        })
        .then_ignore(end())
}

#[cfg(test)]
mod tests;
