use chumsky::extra;
use chumsky::prelude::*;

use super::ast::*;
use super::lexer::Token;

pub type ParserError<'src> = extra::Err<Simple<'src, Token>>;

enum CompoundItem {
    State(String),
    Attr(AttrSelector),
    Pseudo(PseudoClassSelector),
}

enum PseudoArg {
    Formula(NthFormula),
    Selector(Selector),
}

pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Selector, ParserError<'src>> {
    recursive(|selector| {
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

        let nth_formula = {
            let sign = just(Token::Plus).to(1i32).or(just(Token::Minus).to(-1i32));

            let a_part = sign
                .clone()
                .or_not()
                .then_ignore(ws0)
                .then(int_val.or_not())
                .then_ignore(ws0)
                .then_ignore(just(Token::Ident("n".to_string())))
                .map(|(s, i): (Option<i32>, Option<i32>)| s.unwrap_or(1) * i.unwrap_or(1))
                .or_not()
                .map(|a: Option<i32>| a.unwrap_or(0));

            let b_part = ws0
                .ignore_then(sign.clone())
                .then_ignore(ws0)
                .then(int_val)
                .map(|(s, i): (i32, i32)| s * i)
                .or_not()
                .map(|b: Option<i32>| b.unwrap_or(0));

            a_part
                .then(b_part)
                .map(|(a, b): (i32, i32)| NthFormula::new(a, b))
        };

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
            Ok(AttrFlags {
                case_insensitive: ci,
                regex: re,
            })
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
            .map(
                |(((name, op), value), flags): (((StrAttrName, StringOp), String), AttrFlags)| {
                    AttrSelector::Str {
                        name,
                        op,
                        value,
                        flags,
                    }
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
            nth_formula.map(PseudoArg::Formula),
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
                    "has" => match arg {
                        Some(PseudoArg::Selector(s)) => Ok(PseudoClassSelector::Has(Box::new(s))),
                        _ => Err(Simple::new(None, span)),
                    },
                    "not" => match arg {
                        Some(PseudoArg::Selector(s)) => Ok(PseudoClassSelector::Not(Box::new(s))),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-child" => match arg {
                        Some(PseudoArg::Formula(f)) => Ok(PseudoClassSelector::NthChild(f)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-last-child" => match arg {
                        Some(PseudoArg::Formula(f)) => Ok(PseudoClassSelector::NthLastChild(f)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-of-type" => match arg {
                        Some(PseudoArg::Formula(f)) => Ok(PseudoClassSelector::NthOfType(f)),
                        _ => Err(Simple::new(None, span)),
                    },
                    "nth-last-of-type" => match arg {
                        Some(PseudoArg::Formula(f)) => Ok(PseudoClassSelector::NthLastOfType(f)),
                        _ => Err(Simple::new(None, span)),
                    },
                    _ => Err(Simple::new(None, span)),
                },
            );

        let class_selector = just(Token::Dot).ignore_then(ident_val);

        let compound_item = choice((
            class_selector.map(CompoundItem::State),
            attr_selector.map(CompoundItem::Attr),
            pseudo_class_selector.map(CompoundItem::Pseudo),
        ));

        let compound_selector = ident_val
            .or_not()
            .then(compound_item.repeated().collect::<Vec<_>>())
            .map(|(role, items): (Option<String>, Vec<CompoundItem>)| {
                let mut states = Vec::new();
                let mut attrs = Vec::new();
                let mut pseudo_classes = Vec::new();
                for item in items {
                    match item {
                        CompoundItem::State(s) => states.push(s),
                        CompoundItem::Attr(a) => attrs.push(a),
                        CompoundItem::Pseudo(p) => pseudo_classes.push(p),
                    }
                }
                CompoundSelector {
                    role,
                    states,
                    attrs,
                    pseudo_classes,
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
            .or(empty().to(Combinator::Descendant))
            .then_ignore(ws0)
            .then(compound_selector.clone())
            .then(
                combinator
                    .then(compound_selector.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(
                |((leading_combinator, first), tail): (
                    (Combinator, CompoundSelector),
                    Vec<(Combinator, CompoundSelector)>,
                )| {
                    ComplexSelector {
                        leading_combinator,
                        first,
                        tail,
                    }
                },
            );

        complex_selector
            .separated_by(ws0.ignore_then(just(Token::Comma)).then_ignore(ws0))
            .collect::<Vec<_>>()
            .map(|alternatives| Selector { alternatives })
            .then_ignore(end())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_tree::selector::lexer::Token;
    use logos::Logos;

    fn parse(s: &str) -> Result<Selector, String> {
        let tokens: Vec<Token> = Token::lexer(s).collect::<Result<Vec<_>, _>>().unwrap();
        match parser().parse(&tokens).into_result() {
            Ok(ok) => Ok(ok),
            Err(simple) => Err(simple
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    fn parse_err(s: &str) -> String {
        let tokens: Vec<Token> = Token::lexer(s).collect::<Result<Vec<_>, _>>().unwrap();
        parser()
            .parse(&tokens)
            .into_errors()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cs(
        leading_combinator: Combinator,
        role: Option<&str>,
        states: Vec<&str>,
        attrs: Vec<AttrSelector>,
        pseudo_classes: Vec<PseudoClassSelector>,
        tail: Vec<(Combinator, CompoundSelector)>,
    ) -> ComplexSelector {
        ComplexSelector {
            leading_combinator,
            first: CompoundSelector {
                role: role.map(|s| s.to_string()),
                states: states.into_iter().map(|s| s.to_string()).collect(),
                attrs,
                pseudo_classes,
            },
            tail,
        }
    }

    fn compound(
        role: Option<&str>,
        states: Vec<&str>,
        attrs: Vec<AttrSelector>,
        pseudo_classes: Vec<PseudoClassSelector>,
    ) -> CompoundSelector {
        CompoundSelector {
            role: role.map(|s| s.to_string()),
            states: states.into_iter().map(|s| s.to_string()).collect(),
            attrs,
            pseudo_classes,
        }
    }

    fn sel(alternatives: Vec<ComplexSelector>) -> Selector {
        Selector { alternatives }
    }

    // ---- Type / Role selectors ----

    #[test]
    fn role_selector() {
        let result = parse("button").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                Some("button"),
                vec![],
                vec![],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn role_selector_with_hyphen() {
        let result = parse("check-box").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                Some("check-box"),
                vec![],
                vec![],
                vec![],
                vec![]
            )])
        );
    }

    // ---- Class / State selectors ----

    #[test]
    fn class_selector() {
        let result = parse(".focused").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec!["focused"],
                vec![],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn type_with_class() {
        let result = parse("button.focused").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                Some("button"),
                vec!["focused"],
                vec![],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn multiple_classes() {
        let result = parse("button.focused.enabled").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                Some("button"),
                vec!["focused", "enabled"],
                vec![],
                vec![],
                vec![]
            )])
        );
    }

    // ---- String attribute selectors ----

    #[test]
    fn str_attr_eq() {
        let result = parse(r#"[name="ok"]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Name,
                    op: StringOp::Eq,
                    value: "ok".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_contains_word() {
        let result = parse(r#"[description~=foo]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Description,
                    op: StringOp::ContainsWord,
                    value: "foo".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_case_insensitive() {
        let result = parse(r#"[name="ok" i]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Name,
                    op: StringOp::Eq,
                    value: "ok".to_string(),
                    flags: AttrFlags {
                        case_insensitive: true,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_regex() {
        let result = parse(r#"[text~=foo r]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Text,
                    op: StringOp::ContainsWord,
                    value: "foo".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: true
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_starts_with() {
        let result = parse(r#"[actions^=click]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Actions,
                    op: StringOp::Starts,
                    value: "click".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_ends_with() {
        let result = parse(r#"[states$=enabled]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::States,
                    op: StringOp::Ends,
                    value: "enabled".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn str_attr_contains() {
        let result = parse(r#"[states_en_us*=focus]"#).unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::StatesEnUs,
                    op: StringOp::Contains,
                    value: "focus".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    // ---- Int attribute selectors ----

    #[test]
    fn int_attr_eq() {
        let result = parse("[x==10]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Int {
                    name: IntAttrName::X,
                    op: IntOp::Eq,
                    value: Some(10)
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn int_attr_ne() {
        let result = parse("[y!=5]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Int {
                    name: IntAttrName::Y,
                    op: IntOp::Ne,
                    value: Some(5)
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn int_attr_gt() {
        let result = parse("[width>100]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Int {
                    name: IntAttrName::Width,
                    op: IntOp::Gt,
                    value: Some(100)
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn int_attr_le() {
        let result = parse("[depth<=3]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Int {
                    name: IntAttrName::Depth,
                    op: IntOp::Le,
                    value: Some(3)
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn int_attr_height() {
        let result = parse("[height>=0]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Int {
                    name: IntAttrName::Height,
                    op: IntOp::Ge,
                    value: Some(0)
                }],
                vec![],
                vec![]
            )])
        );
    }

    // ---- Bool attribute selectors ----

    #[test]
    fn bool_attr_action() {
        let result = parse("[accessible_action]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Bool {
                    name: BoolAttrName::AccessibleAction
                }],
                vec![],
                vec![]
            )])
        );
    }

    #[test]
    fn bool_attr_text() {
        let result = parse("[accessible_text]").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Bool {
                    name: BoolAttrName::AccessibleText
                }],
                vec![],
                vec![]
            )])
        );
    }

    // ---- Combinators ----

    #[test]
    fn child_combinator() {
        let result = parse("parent > child").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: CompoundSelector {
                    role: Some("parent".to_string()),
                    states: vec![],
                    attrs: vec![],
                    pseudo_classes: vec![],
                },
                tail: vec![(
                    Combinator::Child,
                    CompoundSelector {
                        role: Some("child".to_string()),
                        states: vec![],
                        attrs: vec![],
                        pseudo_classes: vec![],
                    }
                )],
            }])
        );
    }

    #[test]
    fn descendant_combinator() {
        let result = parse("ancestor desc").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: CompoundSelector {
                    role: Some("ancestor".to_string()),
                    states: vec![],
                    attrs: vec![],
                    pseudo_classes: vec![],
                },
                tail: vec![(
                    Combinator::Descendant,
                    CompoundSelector {
                        role: Some("desc".to_string()),
                        states: vec![],
                        attrs: vec![],
                        pseudo_classes: vec![],
                    }
                )],
            }])
        );
    }

    #[test]
    fn next_sibling() {
        let result = parse("prev + next").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: compound(Some("prev"), vec![], vec![], vec![]),
                tail: vec![(
                    Combinator::NextSibling,
                    compound(Some("next"), vec![], vec![], vec![])
                )],
            }])
        );
    }

    #[test]
    fn subsequent_sibling() {
        let result = parse("start ~ all").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: compound(Some("start"), vec![], vec![], vec![]),
                tail: vec![(
                    Combinator::SubsequentSibling,
                    compound(Some("all"), vec![], vec![], vec![])
                )],
            }])
        );
    }

    // ---- Pseudo-classes ----

    #[test]
    fn pseudo_has() {
        let result = parse(":has(button)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::Has(Box::new(sel(vec![cs(
                    Combinator::Descendant,
                    Some("button"),
                    vec![],
                    vec![],
                    vec![],
                    vec![]
                )])))],
                vec![]
            )])
        );
    }

    #[test]
    fn pseudo_not() {
        let result = parse(":not(.disabled)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::Not(Box::new(sel(vec![cs(
                    Combinator::Descendant,
                    None,
                    vec!["disabled"],
                    vec![],
                    vec![],
                    vec![]
                )])))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_odd() {
        let result = parse(":nth-child(odd)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(2, 1))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_even() {
        let result = parse(":nth-child(even)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(2, 0))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_an_plus_b() {
        let result = parse(":nth-child(2n+1)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(2, 1))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_just_n() {
        let result = parse(":nth-child(n)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(1, 0))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_negative_a() {
        let result = parse(":nth-child(-n+3)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(-1, 3))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_child_negative_b() {
        let result = parse(":nth-child(2n-3)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthChild(NthFormula::new(2, -3))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_last_child() {
        let result = parse(":nth-last-child(-n+1)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthLastChild(NthFormula::new(-1, 1))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_of_type() {
        let result = parse(":nth-of-type(3n)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthOfType(NthFormula::new(3, 0))],
                vec![]
            )])
        );
    }

    #[test]
    fn nth_last_of_type() {
        let result = parse(":nth-last-of-type(4n+2)").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![],
                vec![PseudoClassSelector::NthLastOfType(NthFormula::new(4, 2))],
                vec![]
            )])
        );
    }

    // ---- Alternatives (comma-separated) ----

    #[test]
    fn comma_separated() {
        let result = parse("button, input").unwrap();
        assert_eq!(
            result,
            sel(vec![
                cs(
                    Combinator::Descendant,
                    Some("button"),
                    vec![],
                    vec![],
                    vec![],
                    vec![]
                ),
                cs(
                    Combinator::Descendant,
                    Some("input"),
                    vec![],
                    vec![],
                    vec![],
                    vec![]
                ),
            ])
        );
    }

    #[test]
    fn multiple_alternatives() {
        let result = parse("button, input.focused, [name=ok]").unwrap();
        assert_eq!(
            result,
            sel(vec![
                cs(
                    Combinator::Descendant,
                    Some("button"),
                    vec![],
                    vec![],
                    vec![],
                    vec![]
                ),
                cs(
                    Combinator::Descendant,
                    Some("input"),
                    vec!["focused"],
                    vec![],
                    vec![],
                    vec![]
                ),
                cs(
                    Combinator::Descendant,
                    None,
                    vec![],
                    vec![AttrSelector::Str {
                        name: StrAttrName::Name,
                        op: StringOp::Eq,
                        value: "ok".to_string(),
                        flags: AttrFlags {
                            case_insensitive: false,
                            regex: false
                        },
                    }],
                    vec![],
                    vec![]
                ),
            ])
        );
    }

    // ---- Leading combinators ----

    #[test]
    fn leading_child() {
        let result = parse("> .child").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Child,
                first: compound(None, vec!["child"], vec![], vec![]),
                tail: vec![],
            }])
        );
    }

    #[test]
    fn leading_next_sibling() {
        let result = parse("+ .sibling").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::NextSibling,
                first: compound(None, vec!["sibling"], vec![], vec![]),
                tail: vec![],
            }])
        );
    }

    // ---- Full compound selectors ----

    #[test]
    fn full_compound() {
        let result = parse(r#"button.focused.enabled[name="ok"][x==0]:not(.hidden)"#).unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: CompoundSelector {
                    role: Some("button".to_string()),
                    states: vec!["focused".to_string(), "enabled".to_string()],
                    attrs: vec![
                        AttrSelector::Str {
                            name: StrAttrName::Name,
                            op: StringOp::Eq,
                            value: "ok".to_string(),
                            flags: AttrFlags {
                                case_insensitive: false,
                                regex: false
                            },
                        },
                        AttrSelector::Int {
                            name: IntAttrName::X,
                            op: IntOp::Eq,
                            value: Some(0),
                        },
                    ],
                    pseudo_classes: vec![PseudoClassSelector::Not(Box::new(sel(vec![cs(
                        Combinator::Descendant,
                        None,
                        vec!["hidden"],
                        vec![],
                        vec![],
                        vec![]
                    )]))),],
                },
                tail: vec![],
            }])
        );
    }

    // ---- Whitespace tolerance ----

    #[test]
    fn whitespace_around_combinator() {
        let result = parse("parent  >  child").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: compound(Some("parent"), vec![], vec![], vec![]),
                tail: vec![(
                    Combinator::Child,
                    compound(Some("child"), vec![], vec![], vec![])
                )],
            }])
        );
    }

    #[test]
    fn multiple_whitespace_descendant() {
        let result = parse("ancestor    descendant").unwrap();
        assert_eq!(
            result,
            sel(vec![ComplexSelector {
                leading_combinator: Combinator::Descendant,
                first: compound(Some("ancestor"), vec![], vec![], vec![]),
                tail: vec![(
                    Combinator::Descendant,
                    compound(Some("descendant"), vec![], vec![], vec![])
                )],
            }])
        );
    }

    // ---- Error cases ----

    #[test]
    fn invalid_pseudo_class() {
        let errors = parse_err(":unknown(button)");
        assert!(!errors.is_empty());
    }

    #[test]
    fn invalid_flag() {
        let errors = parse_err(r#"[name="foo" x]"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn unclosed_bracket() {
        let errors = parse_err("[name=foo");
        assert!(!errors.is_empty());
    }

    #[test]
    fn empty_brackets() {
        let errors = parse_err("[]");
        assert!(!errors.is_empty());
    }

    // ---- Single-quoted strings ----

    #[test]
    fn single_quoted_string() {
        let result = parse("[name='ok']").unwrap();
        assert_eq!(
            result,
            sel(vec![cs(
                Combinator::Descendant,
                None,
                vec![],
                vec![AttrSelector::Str {
                    name: StrAttrName::Name,
                    op: StringOp::Eq,
                    value: "ok".to_string(),
                    flags: AttrFlags {
                        case_insensitive: false,
                        regex: false
                    }
                }],
                vec![],
                vec![]
            )])
        );
    }

    // ---- String with special chars ----

    #[test]
    fn string_with_quotes_inside() {
        let result = parse(r#"[name="hello \"world\""]"#).unwrap();
        let sel = result;
        assert_eq!(sel.alternatives.len(), 1);
        match &sel.alternatives[0].first.attrs[0] {
            AttrSelector::Str { value, .. } => {
                assert_eq!(value, "hello \"world\"");
            }
            _ => panic!("expected Str attr"),
        }
    }
}
