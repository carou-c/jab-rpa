use rayon::prelude::*;

use super::{ContextNode, ContextTree};
use crate::selector::ast::*;

pub fn matches_selector(
    node: &ContextNode,
    selector: &Selector,
    relative_to: Option<&ContextNode>,
    tree: &ContextTree,
) -> bool {
    selector.alternatives.iter().any(|complex| {
        matches_complex(
            node,
            &complex.head,
            &complex.body,
            &complex.last,
            relative_to,
            tree,
        )
    })
}

fn matches_complex(
    node: &ContextNode,
    complex_head: &Option<Combinator>,
    complex_body: &[(CompoundSelector, Combinator)],
    complex_last: &CompoundSelector,
    relative_to: Option<&ContextNode>,
    tree: &ContextTree,
) -> bool {
    if !matches_compound(node, complex_last, relative_to, tree) {
        return false;
    }

    // Lil strange assignment
    // Basically moves the selector rightwise
    //
    // head [body_n, ..., body_0] last =>
    // head [body_n, ..., body_1] body_0.compound
    //
    // Some(head) [] last =>
    // None [] :scope
    //
    // None [] last =>
    // <End Recursion>
    let scope_selector;
    let (compound, combinator, head) = match complex_body.first() {
        Some((compound, combinator)) => (compound, combinator, complex_head),
        None => {
            let Some(head) = complex_head else {
                return true;
            };

            scope_selector = CompoundSelector {
                role: None,
                attrs: Vec::new(),
                pseudo_classes: vec![PseudoClassSelector::Scope],
            };
            (&scope_selector, head, &None)
        }
    };

    let complex_body = complex_body.get(1..).unwrap_or(&[]);

    match combinator {
        Combinator::Child => {
            let Some(p_h) = node.parent else { return false };
            let Some(p) = tree.nodes.get(&p_h) else {
                return false;
            };
            matches_complex(p, head, complex_body, compound, relative_to, tree)
        }
        Combinator::Descendant => {
            let mut current = node;
            while current.depth > 0 {
                let Some(p_h) = current.parent else {
                    return false;
                };
                let Some(p) = tree.nodes.get(&p_h) else {
                    return false;
                };
                current = p;
                if matches_complex(current, head, complex_body, compound, relative_to, tree) {
                    return true;
                }
            }

            false
        }
        Combinator::NextSibling => {
            let Some(p_h) = node.parent else { return false };
            let Some(p) = tree.nodes.get(&p_h) else {
                return false;
            };
            let sibling_index = node.index_in_parent - 1;
            if sibling_index < 0 {
                return false;
            }
            let Some(sibling_handle) = p.children.get(sibling_index as usize) else {
                return false;
            };
            let Some(sibling) = tree.nodes.get(sibling_handle) else {
                return false;
            };
            matches_complex(sibling, head, complex_body, compound, relative_to, tree)
        }
        Combinator::SubsequentSibling => {
            let Some(p_h) = node.parent else { return false };
            let Some(p) = tree.nodes.get(&p_h) else {
                return false;
            };
            for sibling_index in 0..node.index_in_parent {
                let Some(sibling_handle) = p.children.get(sibling_index as usize) else {
                    continue;
                };
                let Some(sibling) = tree.nodes.get(sibling_handle) else {
                    continue;
                };
                if matches_complex(sibling, head, complex_body, compound, relative_to, tree) {
                    return true;
                }
            }

            false
        }
    }
}

fn matches_compound(
    node: &ContextNode,
    compound: &CompoundSelector,
    relative_to: Option<&ContextNode>,
    tree: &ContextTree,
) -> bool {
    let CompoundSelector {
        role,
        attrs,
        pseudo_classes,
    } = compound;

    if let Some(role) = role
        && &node.role != role
    {
        return false;
    }

    for pseudo in pseudo_classes.iter().filter(|p| {
        matches!(
            p,
            PseudoClassSelector::RequireState(_) | PseudoClassSelector::ExcludeState(_)
        )
    }) {
        if !matches_pseudo_class(node, pseudo, relative_to, tree) {
            return false;
        }
    }

    for attr in attrs {
        if !matches_attribute(node, attr) {
            return false;
        }
    }

    for pseudo in pseudo_classes.iter().filter(|p| {
        !matches!(
            p,
            PseudoClassSelector::RequireState(_) | PseudoClassSelector::ExcludeState(_)
        )
    }) {
        if !matches_pseudo_class(node, pseudo, relative_to, tree) {
            return false;
        }
    }

    true
}

fn matches_attribute(node: &ContextNode, attr: &AttrSelector) -> bool {
    match attr {
        AttrSelector::Str {
            name,
            op,
            value,
            flags,
        } => match_string_op(get_string_attr(node, name), *op, value, flags),
        AttrSelector::Int { name, op, value } => {
            let Some(target) = value else { return false };
            match_int_op(get_int_attr(node, name), *op, *target)
        }
        AttrSelector::Bool { name } => get_bool_attr(node, name),
    }
}

fn get_string_attr<'a>(node: &'a ContextNode, name: &StrAttrName) -> &'a str {
    match name {
        StrAttrName::Name => &node.name,
        StrAttrName::Description => &node.description,
        StrAttrName::States => node.resolve_states(),
        StrAttrName::StatesEnUs => node.resolve_states_en_us(),
        StrAttrName::Text => node.resolve_text(),
        StrAttrName::Actions => node.resolve_action_names(),
    }
}

fn get_int_attr(node: &ContextNode, name: &IntAttrName) -> i32 {
    match name {
        IntAttrName::X => node.x,
        IntAttrName::Y => node.y,
        IntAttrName::Width => node.width,
        IntAttrName::Height => node.height,
        IntAttrName::ChildrenCount => node.children_count,
        IntAttrName::Depth => node.depth,
    }
}

fn get_bool_attr(node: &ContextNode, name: &BoolAttrName) -> bool {
    match name {
        BoolAttrName::AccessibleAction => node.accessible_action,
        BoolAttrName::AccessibleText => node.accessible_text,
        BoolAttrName::AccessibleSelection => node.accessible_selection,
    }
}

fn match_string_op(value: &str, op: StringOp, target: &StrMatcher, flags: &AttrFlags) -> bool {
    match target {
        StrMatcher::Regex(re) => re.is_match(value),
        StrMatcher::Plain(s) => {
            let (val, tgt) = if flags.case_insensitive {
                (value.to_lowercase(), s.to_lowercase())
            } else {
                (value.to_string(), s.to_string())
            };
            match op {
                StringOp::Eq => val == tgt,
                StringOp::ContainsWord => val.split_whitespace().any(|w| w == tgt),
                StringOp::Starts => val.starts_with(&tgt),
                StringOp::Ends => val.ends_with(&tgt),
                StringOp::Contains => val.contains(&tgt),
            }
        }
    }
}

fn match_int_op(node_value: i32, op: IntOp, target: i32) -> bool {
    match op {
        IntOp::Eq => node_value == target,
        IntOp::Ne => node_value != target,
        IntOp::Le => node_value <= target,
        IntOp::Ge => node_value >= target,
        IntOp::Lt => node_value < target,
        IntOp::Gt => node_value > target,
    }
}

fn matches_pseudo_class(
    node: &ContextNode,
    pseudo: &PseudoClassSelector,
    relative_to: Option<&ContextNode>,
    tree: &ContextTree,
) -> bool {
    match pseudo {
        PseudoClassSelector::Scope => {
            let Some(relative_to) = relative_to else {
                return false;
            };

            node.handle == relative_to.handle
        }
        PseudoClassSelector::Has(inner) => {
            if inner.alternatives.iter().any(|complex| {
                (complex.head == Some(Combinator::NextSibling))
                    || (complex.head == Some(Combinator::SubsequentSibling))
            }) {
                node.parent
                    .and_then(|h| tree.nodes.get(&h))
                    .map_or_else(
                        || tree.subtree(node).collect::<Vec<_>>().into_par_iter(),
                        |parent| tree.subtree(parent).collect::<Vec<_>>().into_par_iter(),
                    )
                    .filter_map(|handle| tree.nodes.get(&handle))
                    .any(|candidate| matches_selector(candidate, inner, Some(node), tree))
            } else {
                tree.subtree(node)
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .filter_map(|handle| tree.nodes.get(&handle))
                    .any(|candidate| matches_selector(candidate, inner, Some(node), tree))
            }
        }
        PseudoClassSelector::Not(inner) => !matches_selector(node, inner, relative_to, tree),
        PseudoClassSelector::RequireState(s) => node.states_en_us.contains(s),
        PseudoClassSelector::ExcludeState(s) => !node.states_en_us.contains(s),
        PseudoClassSelector::NthChild(n) => matches_nth_child(node, *n),
        PseudoClassSelector::NthLastChild(n) => matches_nth_last_child(tree, node, *n),
    }
}

fn matches_nth_child(node: &ContextNode, n: i32) -> bool {
    node.index_in_parent + 1 == n
}

fn matches_nth_last_child(tree: &ContextTree, node: &ContextNode, n: i32) -> bool {
    let Some(parent_handle) = node.parent else {
        return n == 1;
    };
    let Some(parent) = tree.nodes.get(&parent_handle) else {
        return false;
    };
    parent.children_count - node.index_in_parent == n
}
