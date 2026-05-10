use std::collections::HashMap;
use std::sync::Arc;

use jab_wrapper::{JabWrapper, types::JavaObject};
use crate::proto;
use crate::utils::utf16_to_string;

pub(crate) type NodeHandle = u64;
pub(crate) const ROOT_HANDLE: NodeHandle = 0;

#[derive(Debug)]
pub(crate) struct ContextNode {
    pub(crate) obj: JavaObject,
    pub(crate) handle: NodeHandle,
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) states: Vec<String>,
    pub(crate) states_en_us: Vec<String>,
    pub(crate) description: String,
    pub(crate) children: Vec<NodeHandle>,
    pub(crate) text: String,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) accessible_action: bool,
    pub(crate) accessible_text: bool,
    pub(crate) accessible_selection: bool,
    pub(crate) children_count: i32,
    pub(crate) index_in_parent: i32,
    pub(crate) parent: Option<NodeHandle>,
    pub(crate) depth: i32,
}

#[derive(Debug)]
pub(crate) struct ContextTree {
    pub(crate) nodes: HashMap<NodeHandle, ContextNode>,
    pub(crate) next_handle: NodeHandle,
}

impl ContextNode {
    fn new(
        obj: JavaObject,
        depth: i32,
        handle: NodeHandle,
        parent: Option<NodeHandle>,
        jab: &std::sync::Arc<JabWrapper>,
    ) -> Self {
        let mut node = Self {
            obj,
            handle,
            name: String::new(),
            role: String::new(),
            states: Vec::new(),
            states_en_us: Vec::new(),
            description: String::new(),
            children: Vec::new(),
            text: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            accessible_action: false,
            accessible_text: false,
            accessible_selection: false,
            children_count: 0,
            index_in_parent: 0,
            parent,
            depth,
        };

        if let Some(info) = jab.get_obj_info(&node.obj) {
            node.name = utf16_to_string(&info.name);
            node.role = utf16_to_string(&info.role);
            node.states = utf16_to_string(&info.states)
                .split(',')
                .map(str::to_uppercase)
                .collect();
            node.states_en_us = utf16_to_string(&info.states_en_US)
                .split(',')
                .map(str::to_uppercase)
                .collect();
            node.description = utf16_to_string(&info.description);
            node.x = info.x;
            node.y = info.y;
            node.width = info.width;
            node.height = info.height;
            node.accessible_action = info.accessibleAction != 0;
            node.accessible_text = info.accessibleText != 0;
            node.accessible_selection = info.accessibleSelection != 0;
            node.children_count = info.childrenCount;
            node.index_in_parent = info.indexInParent;

            node.children.reserve(node.children_count as usize);

            if node.accessible_text
                && let Ok(text) = jab.get_text(&node.obj)
            {
                node.text = text;
            }
        }
        node
    }
}

impl ContextTree {
    pub(crate) fn root(&self) -> &ContextNode {
        &self.nodes[&ROOT_HANDLE]
    }

    pub(crate) fn into_root(mut self) -> JavaObject {
        self.nodes
            .remove(&ROOT_HANDLE)
            .expect("Root node missing")
            .obj
    }

    pub(crate) fn from_root(root_obj: JavaObject, max_depth: Option<i32>, jab: &Arc<JabWrapper>) -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            next_handle: ROOT_HANDLE + 1,
        };

        let mut root = ContextNode::new(root_obj, 0, ROOT_HANDLE, None, jab);
        tree.build_subtree(&mut root, max_depth, jab);
        tree.nodes.insert(ROOT_HANDLE, root);

        tree
    }

    fn build_subtree(
        &mut self,
        node: &mut ContextNode,
        max_depth: Option<i32>,
        jab: &std::sync::Arc<JabWrapper>,
    ) {
        if let Some(max) = max_depth
            && node.depth >= max
        {
            return;
        }

        for i in 0..node.children_count {
            let child_obj = unsafe { jab.get_child_from_obj(&node.obj, i) };

            let handle = self.next_handle;
            self.next_handle += 1;
            let mut child_node =
                ContextNode::new(child_obj, node.depth + 1, handle, Some(node.handle), jab);

            self.build_subtree(&mut child_node, max_depth, jab);

            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }
    }

    pub(crate) fn get_nodes(&self, locator: &proto::Locator) -> Vec<&ContextNode> {
        let mut results = Vec::new();
        self.collect_matching(self.root(), locator, &[], &mut results);
        results
    }

    fn collect_matching<'a>(
        &'a self,
        node: &'a ContextNode,
        locator: &proto::Locator,
        ancestors: &[&'a ContextNode],
        results: &mut Vec<&'a ContextNode>,
    ) {
        if self.node_matches(node, locator, ancestors) {
            results.push(node);
        }

        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(node);

        for child in &node.children {
            if let Some(child_node) = self.nodes.get(child) {
                self.collect_matching(child_node, locator, &child_ancestors, results);
            }
        }
    }

    fn node_matches(
        &self,
        node: &ContextNode,
        locator: &proto::Locator,
        ancestors: &[&ContextNode],
    ) -> bool {
        if !matches_string_field(&node.name, &locator.name) {
            return false;
        }
        if !matches_string_field(&node.role, &locator.role) {
            return false;
        }
        if !matches_string_field(&node.description, &locator.description) {
            return false;
        }
        if !matches_string_field(&node.text, &locator.text) {
            return false;
        }

        if let Some(ref il) = locator.index_in_parent
            && node.index_in_parent != il.index
        {
            return false;
        }

        if locator
            .has_state
            .iter()
            .any(|state| !node.states.contains(&state.to_uppercase()))
        {
            return false;
        }

        if locator
            .not_has_state
            .iter()
            .any(|state| node.states.contains(&state.to_uppercase()))
        {
            return false;
        }

        if let Some(ref asc) = locator.ascendant {
            let found = if asc.is_parent {
                if let Some(parent) = ancestors.last() {
                    matches_node_simple_opt(parent, &asc.locator)
                } else {
                    false
                }
            } else {
                ancestors
                    .iter()
                    .any(|&ancestor| matches_node_simple_opt(ancestor, &asc.locator))
            };
            if !found {
                return false;
            }
        }

        for desc_locator in &locator.descendants {
            if !self.has_descendant_matching(node, desc_locator) {
                return false;
            }
        }

        true
    }

    fn has_descendant_matching(
        &self,
        node: &ContextNode,
        desc_locator: &proto::DescendantLocator,
    ) -> bool {
        let loc_ref: &Option<proto::Locator> = &desc_locator.locator;
        if desc_locator.is_child {
            node.children
                .iter()
                .filter_map(|child| self.nodes.get(child))
                .any(|child| matches_node_simple_opt_ref(child, loc_ref))
        } else {
            node.children
                .iter()
                .filter_map(|child| self.nodes.get(child))
                .any(|child| {
                    matches_node_simple_opt_ref(child, loc_ref)
                        || self.has_descendant_matching(child, desc_locator)
                })
        }
    }
}

fn matches_string_field(field_value: &str, locator: &Option<proto::StringLocator>) -> bool {
    match locator {
        None => true,
        Some(sl) => {
            if sl.regex {
                if let Ok(re) = regex::Regex::new(&sl.find) {
                    re.is_match(field_value)
                } else {
                    false
                }
            } else {
                field_value == sl.find
            }
        }
    }
}

fn matches_node_simple(node: &ContextNode, locator: &proto::Locator) -> bool {
    if !matches_string_field(&node.name, &locator.name) {
        return false;
    }
    if !matches_string_field(&node.role, &locator.role) {
        return false;
    }
    if !matches_string_field(&node.description, &locator.description) {
        return false;
    }
    if !matches_string_field(&node.text, &locator.text) {
        return false;
    }
    if let Some(ref il) = locator.index_in_parent
        && node.index_in_parent != il.index
    {
        return false;
    }

    true
}

fn matches_node_simple_opt(node: &ContextNode, locator: &Option<Box<proto::Locator>>) -> bool {
    match locator {
        None => false,
        Some(box_locator) => matches_node_simple(node, box_locator),
    }
}

fn matches_node_simple_opt_ref(node: &ContextNode, locator: &Option<proto::Locator>) -> bool {
    match locator {
        None => false,
        Some(locator_ref) => matches_node_simple(node, locator_ref),
    }
}
