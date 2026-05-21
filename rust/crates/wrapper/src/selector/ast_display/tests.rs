use crate::selector::ast::*;
use crate::selector::Locator;

fn parse(input: &str) -> Selector {
    Locator::new(input).parse().unwrap()
}

#[test]
fn test_selector_display_role_only() {
    let sel = parse("push_button");
    assert_eq!(sel.to_string(), "push_button");
}

#[test]
fn test_selector_display_wildcard() {
    let sel = parse("*");
    assert_eq!(sel.to_string(), "*");
}

#[test]
fn test_selector_display_state_class() {
    let sel = parse("push_button.enabled");
    assert_eq!(sel.to_string(), "push_button.enabled");
}

#[test]
fn test_selector_display_multiple_states() {
    let sel = parse("push_button.enabled.focusable");
    assert_eq!(sel.to_string(), "push_button.enabled.focusable");
}

#[test]
fn test_selector_display_string_attr() {
    let sel = parse("[name='Clear']");
    assert_eq!(sel.to_string(), "[name='Clear']");
}

#[test]
fn test_selector_display_int_attr() {
    let sel = parse("[x==5]");
    assert_eq!(sel.to_string(), "[x==5]");
}

#[test]
fn test_selector_display_bool_attr() {
    let sel = parse("[accessible_action]");
    assert_eq!(sel.to_string(), "[accessible_action]");
}

#[test]
fn test_selector_display_pseudo_scope() {
    let sel = parse(":has(:scope > text)");
    assert_eq!(sel.to_string(), ":has(:scope > text)");
}

#[test]
fn test_selector_display_pseudo_has() {
    let sel = parse(":has(push_button)");
    assert_eq!(sel.to_string(), ":has( push_button)");
}

#[test]
fn test_selector_display_pseudo_not() {
    let sel = parse(":not(dialog)");
    assert_eq!(sel.to_string(), ":not(dialog)");
}

#[test]
fn test_selector_display_pseudo_nth_child() {
    let sel = parse(":nth-child(3)");
    assert_eq!(sel.to_string(), ":nth-child(3)");
}

#[test]
fn test_selector_display_child_combinator() {
    let sel = parse("dialog > push_button");
    assert_eq!(sel.to_string(), "dialog > push_button");
}

#[test]
fn test_selector_display_descendant_combinator() {
    let sel = parse("dialog push_button");
    assert_eq!(sel.to_string(), "dialog push_button");
}

#[test]
fn test_selector_display_next_sibling() {
    let sel = parse("a + b");
    assert_eq!(sel.to_string(), "a + b");
}

#[test]
fn test_selector_display_subsequent_sibling() {
    let sel = parse("a ~ b");
    assert_eq!(sel.to_string(), "a ~ b");
}

#[test]
fn test_selector_display_complex_compound() {
    let sel = parse("dialog push_button.enabled[name='OK']");
    assert_eq!(sel.to_string(), "dialog push_button.enabled[name='OK']");
}

#[test]
fn test_selector_display_alternatives() {
    let sel = parse("push_button, checkbox");
    assert_eq!(sel.to_string(), "push_button, checkbox");
}

#[test]
fn test_selector_display_chained_combinators() {
    let sel = parse("a > b > c");
    assert_eq!(sel.to_string(), "a > b > c");
}

#[test]
fn test_selector_display_mixed_combinators() {
    let sel = parse("a > b c");
    assert_eq!(sel.to_string(), "a > b c");
}

#[test]
fn test_round_trip_simple() {
    let inputs = vec![
        "push_button",
        "*",
        "push_button.enabled",
        "[name='Clear']",
        "[x==5]",
        "[accessible_action]",
        ":has(:scope)",
        ":has(push_button)",
        ":not(dialog)",
        ":nth-child(1)",
        ":nth-last-child(3)",
    ];
    for input in inputs {
        let sel = parse(input);
        let display = sel.to_string();
        let reparsed = Locator::new(&display).parse().unwrap();
        assert_eq!(sel, reparsed, "round-trip failed for: {input}");
    }
}

#[test]
fn test_round_trip_combinators() {
    let inputs = vec![
        "a > b",
        "a b",
        "a + b",
        "a ~ b",
        "a > b > c",
        "a b c",
        "a > b c",
    ];
    for input in inputs {
        let sel = parse(input);
        let display = sel.to_string();
        let reparsed = Locator::new(&display).parse().unwrap();
        assert_eq!(sel, reparsed, "round-trip failed for: {input}");
    }
}

#[test]
fn test_combinator_display() {
    assert_eq!(Combinator::Child.to_string(), " > ");
    assert_eq!(Combinator::Descendant.to_string(), " ");
    assert_eq!(Combinator::NextSibling.to_string(), " + ");
    assert_eq!(Combinator::SubsequentSibling.to_string(), " ~ ");
}
