use crate::selector::Locator;
use crate::selector::ast::*;
fn parse(input: &str) -> Selector {
    Locator::new(input).parse().unwrap()
}

fn parse_err(input: &str) -> String {
    Locator::new(input).parse().unwrap_err().to_string()
}

fn compound(role: Option<&str>) -> CompoundSelector {
    CompoundSelector {
        role: role.map(String::from),
        states: vec![],
        attrs: vec![],
        pseudo_classes: vec![],
    }
}

fn complex(
    head: Option<Combinator>,
    body: Vec<(CompoundSelector, Combinator)>,
    last: CompoundSelector,
) -> ComplexSelector {
    ComplexSelector { head, body, last }
}

// ---- Role selectors ----

#[test]
fn test_role_name() {
    let result = parse("push_button");
    let expected = Selector {
        alternatives: vec![complex(None, vec![], compound(Some("push_button")))],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_wildcard_role() {
    let result = parse("*");
    let expected = Selector {
        alternatives: vec![complex(None, vec![], compound(None))],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_role_with_hyphen() {
    let result = parse("push-button");
    let expected = Selector {
        alternatives: vec![complex(None, vec![], compound(Some("push-button")))],
    };
    assert_eq!(result, expected);
}

// ---- State classes ----

#[test]
fn test_single_state_class() {
    let result = parse("push_button.enabled");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![],
            CompoundSelector {
                role: Some("push_button".into()),
                states: vec!["enabled".into()],
                attrs: vec![],
                pseudo_classes: vec![],
            },
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_multiple_state_classes() {
    let result = parse("push_button.enabled.focusable");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![],
            CompoundSelector {
                role: Some("push_button".into()),
                states: vec!["enabled".into(), "focusable".into()],
                attrs: vec![],
                pseudo_classes: vec![],
            },
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_state_class_without_role() {
    let result = parse(".enabled");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![],
            CompoundSelector {
                role: None,
                states: vec!["enabled".into()],
                attrs: vec![],
                pseudo_classes: vec![],
            },
        )],
    };
    assert_eq!(result, expected);
}

// ---- String attributes ----

#[test]
fn test_str_attr_eq() {
    let result = parse("[name='Clear']");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![],
            CompoundSelector {
                role: None,
                states: vec![],
                attrs: vec![AttrSelector::Str {
                    name: StrAttrName::Name,
                    op: StringOp::Eq,
                    value: StrMatcher::Plain("Clear".into()),
                    flags: AttrFlags {
                        case_insensitive: false,
                    },
                }],
                pseudo_classes: vec![],
            },
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_str_attr_contains_word() {
    let result = parse("[name~='hello']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            op: StringOp::ContainsWord,
            ..
        }
    ));
}

#[test]
fn test_str_attr_starts_with() {
    let result = parse("[name^='Pre']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            op: StringOp::Starts,
            ..
        }
    ));
}

#[test]
fn test_str_attr_ends_with() {
    let result = parse("[name$='fix']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            op: StringOp::Ends,
            ..
        }
    ));
}

#[test]
fn test_str_attr_contains_substring() {
    let result = parse("[name*='sub']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            op: StringOp::Contains,
            ..
        }
    ));
}

#[test]
fn test_str_attr_with_role() {
    let result = parse("push_button[name='Clear']");
    let last = &result.alternatives[0].last;
    assert_eq!(last.role.as_deref(), Some("push_button"));
    assert!(matches!(
        &last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::Name,
            op: StringOp::Eq,
            value: StrMatcher::Plain(v),
            ..
        } if v == "Clear"
    ));
}

#[test]
fn test_str_attr_description() {
    let result = parse("[description='tooltip']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::Description,
            ..
        }
    ));
}

#[test]
fn test_str_attr_states() {
    let result = parse("[states='enabled']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::States,
            ..
        }
    ));
}

#[test]
fn test_str_attr_states_en_us() {
    let result = parse("[states_en_us='enabled']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::StatesEnUs,
            ..
        }
    ));
}

#[test]
fn test_str_attr_text() {
    let result = parse("[text='hello']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::Text,
            ..
        }
    ));
}

#[test]
fn test_str_attr_actions() {
    let result = parse("[actions='click']");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::Actions,
            ..
        }
    ));
}

#[test]
fn test_str_attr_case_insensitive_flag() {
    let result = parse("[name='ok' i]");
    if let AttrSelector::Str { flags, .. } = &result.alternatives[0].last.attrs[0] {
        assert!(flags.case_insensitive);
    } else {
        panic!("expected Str attr");
    }
}

#[test]
fn test_str_attr_regex_flag() {
    let result = parse("[name='OK.*' r]");
    if let AttrSelector::Str { value, .. } = &result.alternatives[0].last.attrs[0] {
        assert!(matches!(value, StrMatcher::Regex(_)));
    } else {
        panic!("expected Str attr");
    }
}

#[test]
fn test_str_attr_both_flags() {
    let result = parse("[name='ok' ir]");
    if let AttrSelector::Str { value, flags, .. } = &result.alternatives[0].last.attrs[0] {
        assert!(flags.case_insensitive);
        assert!(matches!(value, StrMatcher::Regex(_)));
    } else {
        panic!("expected Str attr");
    }
}

// ---- Int attributes ----

#[test]
fn test_int_attr_eq() {
    let result = parse("[x==5]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::X,
            op: IntOp::Eq,
            value: Some(5),
        }
    ));
}

#[test]
fn test_int_attr_ne() {
    let result = parse("[x!=10]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            op: IntOp::Ne,
            value: Some(10),
            ..
        }
    ));
}

#[test]
fn test_int_attr_le() {
    let result = parse("[y<=100]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::Y,
            op: IntOp::Le,
            value: Some(100),
        }
    ));
}

#[test]
fn test_int_attr_ge() {
    let result = parse("[width>=0]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::Width,
            op: IntOp::Ge,
            value: Some(0),
        }
    ));
}

#[test]
fn test_int_attr_lt() {
    let result = parse("[height<200]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::Height,
            op: IntOp::Lt,
            value: Some(200),
        }
    ));
}

#[test]
fn test_int_attr_gt() {
    let result = parse("[children_count>0]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::ChildrenCount,
            op: IntOp::Gt,
            value: Some(0),
        }
    ));
}

#[test]
fn test_int_attr_depth() {
    let result = parse("[depth==3]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Int {
            name: IntAttrName::Depth,
            ..
        }
    ));
}

// ---- Bool attributes ----

#[test]
fn test_bool_attr_accessible_action() {
    let result = parse("[accessible_action]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Bool {
            name: BoolAttrName::AccessibleAction,
        }
    ));
}

#[test]
fn test_bool_attr_accessible_text() {
    let result = parse("[accessible_text]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Bool {
            name: BoolAttrName::AccessibleText,
        }
    ));
}

#[test]
fn test_bool_attr_accessible_selection() {
    let result = parse("[accessible_selection]");
    assert!(matches!(
        result.alternatives[0].last.attrs[0],
        AttrSelector::Bool {
            name: BoolAttrName::AccessibleSelection,
        }
    ));
}

// ---- Pseudo-classes ----

#[test]
fn test_pseudo_scope() {
    let result = parse(":has(:scope)");
    assert!(
        result.alternatives[0]
            .last
            .pseudo_classes
            .contains(&PseudoClassSelector::Has(Box::new(Selector {
                alternatives: vec![ComplexSelector {
                    head: None,
                    body: Vec::new(),
                    last: CompoundSelector {
                        role: None,
                        states: Vec::new(),
                        attrs: Vec::new(),
                        pseudo_classes: vec![PseudoClassSelector::Scope],
                    }
                }]
            })))
    );
}

#[test]
fn test_pseudo_has() {
    let result = parse(":has(push_button)");
    assert!(matches!(
        &result.alternatives[0].last.pseudo_classes[0],
        PseudoClassSelector::Has(inner) if inner.alternatives[0].last.role.as_deref() == Some("push_button")
    ));
}

#[test]
fn test_pseudo_has_relative_prepends_descendant() {
    let result = parse(":has(push_button)");
    if let PseudoClassSelector::Has(inner) = &result.alternatives[0].last.pseudo_classes[0] {
        assert_eq!(inner.alternatives[0].head, Some(Combinator::Descendant));
    } else {
        panic!("expected Has");
    }
}

#[test]
fn test_pseudo_has_explicit_combinator() {
    let result = parse(":has(> push_button)");
    if let PseudoClassSelector::Has(inner) = &result.alternatives[0].last.pseudo_classes[0] {
        assert_eq!(inner.alternatives[0].head, Some(Combinator::Child));
    } else {
        panic!("expected Has");
    }
}

#[test]
fn test_pseudo_not() {
    let result = parse(":not(dialog)");
    assert!(matches!(
        &result.alternatives[0].last.pseudo_classes[0],
        PseudoClassSelector::Not(inner) if inner.alternatives[0].last.role.as_deref() == Some("dialog")
    ));
}

#[test]
fn test_pseudo_nth_child() {
    let result = parse(":nth-child(3)");
    assert_eq!(
        result.alternatives[0].last.pseudo_classes[0],
        PseudoClassSelector::NthChild(3)
    );
}

#[test]
fn test_pseudo_nth_last_child() {
    let result = parse(":nth-last-child(1)");
    assert_eq!(
        result.alternatives[0].last.pseudo_classes[0],
        PseudoClassSelector::NthLastChild(1)
    );
}

// ---- Combinators ----

#[test]
fn test_child_combinator() {
    let result = parse("a > b");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![(compound(Some("a")), Combinator::Child)],
            compound(Some("b")),
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_descendant_combinator() {
    let result = parse("a b");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![(compound(Some("a")), Combinator::Descendant)],
            compound(Some("b")),
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_next_sibling_combinator() {
    let result = parse("a + b");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![(compound(Some("a")), Combinator::NextSibling)],
            compound(Some("b")),
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_subsequent_sibling_combinator() {
    let result = parse("a ~ b");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![(compound(Some("a")), Combinator::SubsequentSibling)],
            compound(Some("b")),
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_chained_child_combinators() {
    let result = parse("a > b > c");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![
                (compound(Some("b")), Combinator::Child),
                (compound(Some("a")), Combinator::Child),
            ],
            compound(Some("c")),
        )],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_mixed_combinators() {
    let result = parse("a > b c");
    let expected = Selector {
        alternatives: vec![complex(
            None,
            vec![
                (compound(Some("b")), Combinator::Descendant),
                (compound(Some("a")), Combinator::Child),
            ],
            compound(Some("c")),
        )],
    };
    assert_eq!(result, expected);
}

// ---- Alternatives ----

#[test]
fn test_two_alternatives() {
    let result = parse("a, b");
    let expected = Selector {
        alternatives: vec![
            complex(None, vec![], compound(Some("a"))),
            complex(None, vec![], compound(Some("b"))),
        ],
    };
    assert_eq!(result, expected);
}

#[test]
fn test_three_alternatives() {
    let result = parse("a, b, c");
    assert_eq!(result.alternatives.len(), 3);
}

#[test]
fn test_alternatives_with_combinators() {
    let result = parse("a > b, c + d");
    assert_eq!(result.alternatives.len(), 2);
    assert_eq!(result.alternatives[0].body.len(), 1);
    assert_eq!(result.alternatives[1].body.len(), 1);
}

// ---- Complex selectors ----

#[test]
fn test_complex_compound() {
    let result = parse("dialog > push_button[name='OK'].enabled");
    let alt = &result.alternatives[0];
    assert_eq!(alt.body.len(), 1);
    assert_eq!(alt.body[0].0.role.as_deref(), Some("dialog"));
    assert_eq!(alt.body[0].1, Combinator::Child);
    assert_eq!(alt.last.role.as_deref(), Some("push_button"));
    assert!(alt.last.states.contains(&"enabled".into()));
    assert!(matches!(
        &alt.last.attrs[0],
        AttrSelector::Str {
            name: StrAttrName::Name,
            op: StringOp::Eq,
            value: StrMatcher::Plain(v),
            ..
        } if v == "OK"
    ));
}

#[test]
fn test_selector_with_multiple_attrs() {
    let result = parse("[name='OK'][x==5]");
    assert_eq!(result.alternatives[0].last.attrs.len(), 2);
}

#[test]
fn test_selector_with_role_and_multiple_attributes() {
    let result = parse("push_button[name='Clear'].enabled:not(dialog)");
    let last = &result.alternatives[0].last;
    assert_eq!(last.role.as_deref(), Some("push_button"));
    assert!(last.states.contains(&"enabled".into()));
    assert_eq!(last.attrs.len(), 1);
    assert_eq!(last.pseudo_classes.len(), 1);
}

// ---- Error cases ----

#[test]
fn test_error_empty_input() {
    let err = parse_err("");
    assert!(err.contains("Parse error"));
}

#[test]
fn test_error_relative_selector_at_top_level() {
    assert!(Locator::new("> a").parse().is_err());
}

#[test]
fn test_error_scope_relative_with_combinator() {
    // :scope at top level is relative → rejected
    assert!(Locator::new(":scope").parse().is_err());
}

#[test]
fn test_error_unclosed_bracket() {
    assert!(Locator::new("[name='OK'").parse().is_err());
}

#[test]
fn test_error_unknown_pseudo_class() {
    assert!(Locator::new(":unknown").parse().is_err());
}

#[test]
fn test_error_unknown_attr_name() {
    assert!(Locator::new("[unknown_attr=5]").parse().is_err());
}

#[test]
fn test_error_invalid_regex() {
    assert!(Locator::new("[name='[invalid' r]").parse().is_err());
}

#[test]
fn test_error_invalid_int_attr_comparison() {
    assert!(Locator::new("[x='string']").parse().is_err());
}
