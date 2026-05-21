use crate::selector::Locator;

#[test]
fn test_locator_new_and_parse_ok() {
    let loc = Locator::new("push_button");
    assert_eq!(loc.selector, "push_button");
    assert!(loc.parse().is_ok());
}

#[test]
fn test_locator_parse_err() {
    let loc = Locator::new("> a");
    assert!(loc.parse().is_err());
}

#[test]
fn test_locator_pipeline_complex_selector() {
    let loc = Locator::new("dialog > push_button[name='Clear'].enabled");
    let result = loc.parse();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
    let sel = result.unwrap();
    assert_eq!(sel.alternatives.len(), 1);
}

#[test]
fn test_locator_pipeline_alternatives() {
    let loc = Locator::new("push_button, checkbox, radio_button");
    let sel = loc.parse().unwrap();
    assert_eq!(sel.alternatives.len(), 3);
}

#[test]
fn test_locator_pipeline_wildcard() {
    let loc = Locator::new("*");
    let sel = loc.parse().unwrap();
    assert!(sel.alternatives[0].last.role.is_none());
}

#[test]
fn test_locator_pipeline_pseudo_has() {
    let loc = Locator::new(":has(push_button)");
    let sel = loc.parse().unwrap();
    assert_eq!(sel.alternatives[0].last.pseudo_classes.len(), 1);
}

#[test]
fn test_locator_pipeline_not_relative() {
    // :not with relative selectors inside should be rejected
    assert!(Locator::new(":not(> button)").parse().is_err());
}

#[test]
fn test_locator_pipeline_all_attr_types() {
    let loc = Locator::new("[name='ok' i][x==5][accessible_action]");
    let sel = loc.parse().unwrap();
    assert_eq!(sel.alternatives[0].last.attrs.len(), 3);
}

#[test]
fn test_locator_parse_reevaluate() {
    let loc = Locator::new("a");
    assert!(loc.parse().is_ok());
    // The same locator can be parsed multiple times
    assert!(loc.parse().is_ok());
}
