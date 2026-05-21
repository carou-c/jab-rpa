use crate::selector::ast::*;
use regex::Regex;

fn compound(role: Option<&str>) -> CompoundSelector {
    CompoundSelector {
        role: role.map(String::from),
        states: vec![],
        attrs: vec![],
        pseudo_classes: vec![],
    }
}

fn simple_selector(role: Option<&str>) -> Selector {
    Selector {
        alternatives: vec![ComplexSelector {
            head: None,
            body: vec![],
            last: compound(role),
        }],
    }
}

#[test]
fn test_selector_equality() {
    assert_eq!(
        simple_selector(Some("push_button")),
        simple_selector(Some("push_button")),
    );
}

#[test]
fn test_selector_inequality() {
    assert_ne!(
        simple_selector(Some("push_button")),
        simple_selector(Some("checkbox")),
    );
}

#[test]
fn test_selector_different_alternatives_count() {
    let a = Selector {
        alternatives: vec![
            ComplexSelector { head: None, body: vec![], last: compound(Some("a")) },
        ],
    };
    let b = Selector {
        alternatives: vec![
            ComplexSelector { head: None, body: vec![], last: compound(Some("a")) },
            ComplexSelector { head: None, body: vec![], last: compound(Some("b")) },
        ],
    };
    assert_ne!(a, b);
}

#[test]
fn test_str_matcher_plain_equality() {
    let a = StrMatcher::Plain("hello".into());
    let b = StrMatcher::Plain("hello".into());
    assert_eq!(a, b);
}

#[test]
fn test_str_matcher_plain_inequality() {
    let a = StrMatcher::Plain("hello".into());
    let b = StrMatcher::Plain("world".into());
    assert_ne!(a, b);
}

#[test]
fn test_str_matcher_regex_equality() {
    let a = StrMatcher::Regex(Regex::new("^hello.*").unwrap());
    let b = StrMatcher::Regex(Regex::new("^hello.*").unwrap());
    assert_eq!(a, b);
}

#[test]
fn test_str_matcher_regex_inequality() {
    let a = StrMatcher::Regex(Regex::new("^hello").unwrap());
    let b = StrMatcher::Regex(Regex::new("world$").unwrap());
    assert_ne!(a, b);
}

#[test]
fn test_str_matcher_plain_vs_regex_never_equal() {
    let a = StrMatcher::Plain("hello".into());
    let b = StrMatcher::Regex(Regex::new("hello").unwrap());
    assert_ne!(a, b);
}

#[test]
fn test_combinator_equality() {
    assert_eq!(Combinator::Child, Combinator::Child);
    assert_eq!(Combinator::Descendant, Combinator::Descendant);
    assert_eq!(Combinator::NextSibling, Combinator::NextSibling);
    assert_eq!(Combinator::SubsequentSibling, Combinator::SubsequentSibling);
}

#[test]
fn test_combinator_inequality() {
    assert_ne!(Combinator::Child, Combinator::Descendant);
}

#[test]
fn test_attr_flags_equality() {
    assert_eq!(AttrFlags { case_insensitive: true }, AttrFlags { case_insensitive: true });
    assert_ne!(AttrFlags { case_insensitive: true }, AttrFlags { case_insensitive: false });
}

#[test]
fn test_pseudo_class_selector_equality() {
    assert_eq!(PseudoClassSelector::Scope, PseudoClassSelector::Scope);
    assert_eq!(PseudoClassSelector::NthChild(1), PseudoClassSelector::NthChild(1));
    assert_ne!(PseudoClassSelector::NthChild(1), PseudoClassSelector::NthChild(2));
}

#[test]
fn test_is_relative_with_head() {
    let sel = ComplexSelector {
        head: Some(Combinator::Child),
        body: vec![],
        last: compound(Some("button")),
    };
    assert!(sel.is_relative());
}

#[test]
fn test_is_relative_with_scope_in_last() {
    let sel = ComplexSelector {
        head: None,
        body: vec![],
        last: CompoundSelector {
            role: None,
            states: vec![],
            attrs: vec![],
            pseudo_classes: vec![PseudoClassSelector::Scope],
        },
    };
    assert!(sel.is_relative());
}

#[test]
fn test_is_relative_with_scope_in_body() {
    let sel = ComplexSelector {
        head: None,
        body: vec![(
            CompoundSelector {
                role: None,
                states: vec![],
                attrs: vec![],
                pseudo_classes: vec![PseudoClassSelector::Scope],
            },
            Combinator::Descendant,
        )],
        last: compound(Some("button")),
    };
    assert!(sel.is_relative());
}

#[test]
fn test_is_not_relative() {
    let sel = ComplexSelector {
        head: None,
        body: vec![],
        last: compound(Some("button")),
    };
    assert!(!sel.is_relative());
}

#[test]
fn test_str_attr_name_equality() {
    assert_eq!(StrAttrName::Name, StrAttrName::Name);
    assert_ne!(StrAttrName::Name, StrAttrName::Description);
}

#[test]
fn test_int_op_equality() {
    assert_eq!(IntOp::Eq, IntOp::Eq);
    assert_ne!(IntOp::Eq, IntOp::Ne);
}

#[test]
fn test_int_attr_name_equality() {
    assert_eq!(IntAttrName::X, IntAttrName::X);
    assert_ne!(IntAttrName::X, IntAttrName::Y);
}

#[test]
fn test_bool_attr_name_equality() {
    assert_eq!(BoolAttrName::AccessibleAction, BoolAttrName::AccessibleAction);
    assert_ne!(BoolAttrName::AccessibleAction, BoolAttrName::AccessibleText);
}

#[test]
fn test_string_op_equality() {
    assert_eq!(StringOp::Eq, StringOp::Eq);
    assert_ne!(StringOp::Eq, StringOp::Contains);
}

#[test]
fn test_complex_selector_equality() {
    let a = ComplexSelector {
        head: None,
        body: vec![(compound(Some("a")), Combinator::Child)],
        last: compound(Some("b")),
    };
    let b = ComplexSelector {
        head: None,
        body: vec![(compound(Some("a")), Combinator::Child)],
        last: compound(Some("b")),
    };
    assert_eq!(a, b);
}
