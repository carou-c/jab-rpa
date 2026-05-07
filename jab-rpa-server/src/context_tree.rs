use crate::proto;
use crate::jab_wrapper::JabWrapper;
use std::sync::Weak;

#[derive(Debug)]
pub struct ContextNode {
    pub vm_id: i32,
    pub context: i64,
    pub handle: u64,
    pub name: String,
    pub role: String,
    pub description: String,
    pub children: Vec<ContextNode>,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub accessible_action: bool,
    pub accessible_text: bool,
    pub accessible_selection: bool,
    pub visible_children_count: i32,
    pub index_in_parent: i32,
    wrapper: Option<Weak<JabWrapper>>,
}

impl Clone for ContextNode {
    fn clone(&self) -> Self {
        ContextNode {
            vm_id: self.vm_id,
            context: self.context,
            handle: self.handle,
            name: self.name.clone(),
            role: self.role.clone(),
            description: self.description.clone(),
            children: self.children.clone(),
            text: self.text.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            accessible_action: self.accessible_action,
            accessible_text: self.accessible_text,
            accessible_selection: self.accessible_selection,
            visible_children_count: self.visible_children_count,
            index_in_parent: self.index_in_parent,
            wrapper: None, // Don't clone the wrapper - only original releases
        }
    }
}

impl Drop for ContextNode {
    fn drop(&mut self) {
        if let Some(ref wrapper_weak) = self.wrapper
            && let Some(wrapper) = wrapper_weak.upgrade() {
                eprintln!("[ContextNode::drop] Releasing handle={}", self.handle);
                wrapper.release_element(self.handle);
            }
    }
}

#[derive(Debug)]
pub struct ContextTree {
    pub root: Option<ContextNode>,
    pub max_depth: Option<i32>,
    wrapper: Option<Weak<JabWrapper>>,
}

impl Clone for ContextTree {
    fn clone(&self) -> Self {
        ContextTree {
            root: self.root.clone(),
            max_depth: self.max_depth,
            wrapper: self.wrapper.clone(),
        }
    }
}

impl ContextTree {
    pub fn from_root(
        vm_id: i32,
        root_context: i64,
        max_depth: Option<i32>,
        jab: &std::sync::Arc<JabWrapper>,
    ) -> Self {
        let mut tree = ContextTree {
            root: None,
            max_depth,
            wrapper: Some(std::sync::Arc::downgrade(jab)),
        };

        if root_context == 0 {
            return tree;
        }

        tree.root = Some(Self::build_node(vm_id, root_context, 0, max_depth, jab));

        tree
    }

    fn build_node(
        vm_id: i32,
        context: i64,
        depth: i32,
        max_depth: Option<i32>,
        jab: &std::sync::Arc<JabWrapper>,
    ) -> ContextNode {
        let handle = jab.register_element(vm_id, context);

        let mut node = ContextNode {
            vm_id,
            context,
            handle,
            name: String::new(),
            role: String::new(),
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
            visible_children_count: 0,
            index_in_parent: 0,
            wrapper: Some(std::sync::Arc::downgrade(jab)),
        };

        if let Some(info) = jab.get_accessible_context_info(vm_id, context) {
            node.name = String::from_utf16_lossy(&info.name)
                .trim_end_matches('\0')
                .to_string();
            node.role = String::from_utf16_lossy(&info.role)
                .trim_end_matches('\0')
                .to_string();
            node.description = String::from_utf16_lossy(&info.description)
                .trim_end_matches('\0')
                .to_string();
            node.x = info.x;
            node.y = info.y;
            node.width = info.width;
            node.height = info.height;
            node.accessible_action = info.accessibleAction != 0;
            node.accessible_text = info.accessibleText != 0;
            node.accessible_selection = info.accessibleSelection != 0;
            node.visible_children_count = info.childrenCount;
            node.index_in_parent = info.indexInParent;
        }

        if let Some(max) = max_depth
            && depth >= max
        {
            return node;
        }

        unsafe {
            let child_count = if let Some(info) = jab.get_accessible_context_info(vm_id, context) {
                info.childrenCount
            } else {
                0
            };

            for i in 0..child_count {
                let child_context =
                    super::bindings::GetAccessibleChildFromContext(vm_id as _, context, i);
                if child_context != 0 {
                    node.children.push(Self::build_node(
                        vm_id,
                        child_context,
                        depth + 1,
                        max_depth,
                        jab,
                    ));
                }
            }
        }

        node
    }

    pub fn get_elements(&self, locator: &proto::Locator) -> Vec<&ContextNode> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_matching(root, locator, &[], &mut results);
        }
        results
    }

    fn collect_matching<'a>(
        node: &'a ContextNode,
        locator: &proto::Locator,
        ancestors: &[&'a ContextNode],
        results: &mut Vec<&'a ContextNode>,
    ) {
        if Self::node_matches(node, locator, ancestors) {
            results.push(node);
        }

        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(node);

        for child in &node.children {
            Self::collect_matching(child, locator, &child_ancestors, results);
        }
    }

    fn node_matches(
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
            if !Self::has_descendant_matching(node, desc_locator) {
                return false;
            }
        }

        true
    }

    fn has_descendant_matching(
        node: &ContextNode,
        desc_locator: &proto::DescendantLocator,
    ) -> bool {
        let loc_ref: &Option<proto::Locator> = &desc_locator.locator;
        if desc_locator.is_child {
            node.children
                .iter()
                .any(|child| matches_node_simple_opt_ref(child, loc_ref))
        } else {
            node.children.iter().any(|child| {
                matches_node_simple_opt_ref(child, loc_ref)
                    || Self::has_descendant_matching(child, desc_locator)
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

impl Drop for ContextTree {
    fn drop(&mut self) {
        eprintln!("[ContextTree::drop] Dropping tree, nodes will release themselves...");
        // ContextNode instances will release themselves via their Drop impl
        // No need to manually release elements here
        eprintln!("[ContextTree::drop] Done.");
    }
}
