use std::collections::HashSet;

use regex::Regex;

use super::ast::*;
use crate::context_tree::{ContextNode, ContextTree};

pub fn select_nodes<'a>(
    tree: &'a ContextTree,
    selector: &Selector,
    relative_to: Option<&'a ContextNode>,
) -> Vec<&'a ContextNode> {
    let mut seen: HashSet<u64> = HashSet::new();
    let mut results = Vec::new();

    for complex in &selector.alternatives {
        for node in match_complex(tree, complex, relative_to) {
            if seen.insert(node.handle) {
                results.push(node);
            }
        }
    }

    results
}

fn match_complex<'a>(
    tree: &'a ContextTree,
    complex: &ComplexSelector,
    relative_to: Option<&'a ContextNode>,
) -> Vec<&'a ContextNode> {
    let scope: Vec<&ContextNode> = if let Some(combinator) = &complex.leading_combinator {
        match relative_to {
            Some(rt) => apply_combinator_to_node(tree, rt, *combinator),
            None => return Vec::new(),
        }
    } else {
        match relative_to {
            Some(rt) => {
                let mut nodes = vec![rt];
                descendants(tree, rt, &mut nodes);
                nodes
            }
            None => tree.nodes.values().collect(),
        }
    };

    let mut current: Vec<&ContextNode> = scope
        .into_iter()
        .filter(|n| matches_compound(tree, n, &complex.first))
        .collect();

    for (combinator, compound) in &complex.tail {
        let mut next = Vec::new();
        for node in current {
            let reached = apply_combinator_to_node(tree, node, *combinator);
            next.extend(
                reached
                    .into_iter()
                    .filter(|n| matches_compound(tree, n, compound)),
            );
        }
        current = next;
    }

    current
}

fn descendants<'a>(tree: &'a ContextTree, node: &ContextNode, acc: &mut Vec<&'a ContextNode>) {
    for child_handle in &node.children {
        if let Some(child) = tree.nodes.get(child_handle) {
            acc.push(child);
            descendants(tree, child, acc);
        }
    }
}

fn apply_combinator_to_node<'a>(
    tree: &'a ContextTree,
    node: &ContextNode,
    combinator: Combinator,
) -> Vec<&'a ContextNode> {
    match combinator {
        Combinator::Child => node
            .children
            .iter()
            .filter_map(|h| tree.nodes.get(h))
            .collect(),
        Combinator::Descendant => {
            let mut nodes = Vec::new();
            descendants(tree, node, &mut nodes);
            nodes
        }
        Combinator::NextSibling => {
            if let Some(parent_handle) = &node.parent
                && let Some(parent) = tree.nodes.get(parent_handle)
            {
                let idx = node.index_in_parent as usize;
                if idx + 1 < parent.children.len()
                    && let Some(sibling) = tree.nodes.get(&parent.children[idx + 1])
                {
                    return vec![sibling];
                }
            }

            Vec::new()
        }
        Combinator::SubsequentSibling => {
            let mut siblings = Vec::new();
            if let Some(parent_handle) = &node.parent
                && let Some(parent) = tree.nodes.get(parent_handle)
            {
                let idx = node.index_in_parent as usize;
                for h in parent.children.iter().skip(idx + 1) {
                    if let Some(sibling) = tree.nodes.get(h) {
                        siblings.push(sibling);
                    }
                }
            }

            siblings
        }
    }
}

fn matches_compound(tree: &ContextTree, node: &ContextNode, compound: &CompoundSelector) -> bool {
    if let Some(role) = &compound.role
        && node.role != *role
    {
        return false;
    }

    for state in &compound.states {
        if !node.states.iter().any(|s| s == state) && !node.states_en_us.iter().any(|s| s == state)
        {
            return false;
        }
    }

    for attr in &compound.attrs {
        if !matches_attribute(node, attr) {
            return false;
        }
    }

    for pseudo in &compound.pseudo_classes {
        if !matches_pseudo_class(tree, node, pseudo) {
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
        } => {
            let node_val = get_string_attr(node, name);
            match node_val {
                Some(v) => match_string_op(&v, *op, value, flags),
                None => false,
            }
        }
        AttrSelector::Int { name, op, value } => {
            let Some(target) = value else { return false };
            let node_val = get_int_attr(node, name);
            match node_val {
                Some(v) => match_int_op(v, *op, *target),
                None => false,
            }
        }
        AttrSelector::Bool { name } => get_bool_attr(node, name),
    }
}

fn get_string_attr(node: &ContextNode, name: &StrAttrName) -> Option<String> {
    match name {
        StrAttrName::Name => Some(node.name.clone()),
        StrAttrName::Description => Some(node.description.clone()),
        StrAttrName::States => Some(node.states.join(" ")),
        StrAttrName::StatesEnUs => Some(node.states_en_us.join(" ")),
        StrAttrName::Text => Some(node.resolve_text().to_string()),
        StrAttrName::Actions => Some(node.resolve_action_names().to_string()),
    }
}

fn get_int_attr(node: &ContextNode, name: &IntAttrName) -> Option<i32> {
    match name {
        IntAttrName::X => Some(node.x),
        IntAttrName::Y => Some(node.y),
        IntAttrName::Width => Some(node.width),
        IntAttrName::Height => Some(node.height),
        IntAttrName::ChildrenCount => Some(node.children_count),
        IntAttrName::Depth => Some(node.depth),
    }
}

fn get_bool_attr(node: &ContextNode, name: &BoolAttrName) -> bool {
    match name {
        BoolAttrName::AccessibleAction => node.accessible_action,
        BoolAttrName::AccessibleText => node.accessible_text,
        BoolAttrName::AccessibleSelection => node.accessible_selection,
    }
}

fn match_string_op(value: &str, op: StringOp, target: &str, flags: &super::ast::AttrFlags) -> bool {
    let (val, tgt) = if flags.case_insensitive {
        (value.to_lowercase(), target.to_lowercase())
    } else {
        (value.to_string(), target.to_string())
    };

    if flags.regex {
        match Regex::new(&tgt) {
            Ok(re) => re.is_match(&val),
            Err(_) => false,
        }
    } else {
        match op {
            StringOp::Eq => val == tgt,
            StringOp::ContainsWord => val.split_whitespace().any(|w| w == tgt),
            StringOp::Starts => val.starts_with(&tgt),
            StringOp::Ends => val.ends_with(&tgt),
            StringOp::Contains => val.contains(&tgt),
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
    tree: &ContextTree,
    node: &ContextNode,
    pseudo: &PseudoClassSelector,
) -> bool {
    match pseudo {
        PseudoClassSelector::Has(inner) => {
            let results = select_nodes(tree, inner, Some(node));
            !results.is_empty()
        }
        PseudoClassSelector::Not(inner) => {
            let results = select_nodes(tree, inner, None);
            !results.iter().any(|n| n.handle == node.handle)
        }
        PseudoClassSelector::NthChild(formula) => {
            matches_nth_child(tree, node, formula, false, false)
        }
        PseudoClassSelector::NthLastChild(formula) => {
            matches_nth_child(tree, node, formula, true, false)
        }
        PseudoClassSelector::NthOfType(formula) => {
            matches_nth_child(tree, node, formula, false, true)
        }
        PseudoClassSelector::NthLastOfType(formula) => {
            matches_nth_child(tree, node, formula, true, true)
        }
    }
}

fn matches_nth_child(
    tree: &ContextTree,
    node: &ContextNode,
    formula: &NthFormula,
    from_end: bool,
    same_type: bool,
) -> bool {
    let parent_handle = match &node.parent {
        Some(h) => h,
        None => return false,
    };

    let parent = match tree.nodes.get(parent_handle) {
        Some(p) => p,
        None => return false,
    };

    let siblings: Vec<&ContextNode> = parent
        .children
        .iter()
        .filter_map(|h| tree.nodes.get(h))
        .filter(|sibling| {
            if same_type {
                sibling.role == node.role
            } else {
                true
            }
        })
        .collect();

    let pos = match siblings.iter().position(|s| s.handle == node.handle) {
        Some(p) => p,
        None => return false,
    };

    let idx = if from_end {
        siblings.len() - pos
    } else {
        pos + 1
    };

    formula.matches(idx)
}
