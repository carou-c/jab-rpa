use std::collections::HashMap;
use std::sync::OnceLock;

use crate::types::JavaObject;
use crate::utils::utf16_to_string;

pub(crate) type NodeHandle = u64;
pub(crate) const ROOT_HANDLE: NodeHandle = 0;

#[derive(Debug)]
pub struct ContextNode {
    pub obj: JavaObject,
    pub handle: NodeHandle,
    pub name: String,
    pub role: String,
    pub states: Vec<String>,
    pub states_en_us: Vec<String>,
    pub description: String,
    pub children: Vec<NodeHandle>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub accessible_action: bool,
    pub accessible_text: bool,
    pub accessible_selection: bool,
    pub children_count: i32,
    pub index_in_parent: i32,
    pub parent: Option<NodeHandle>,
    pub depth: i32,
    pub text_cache: OnceLock<String>,
    pub action_names_cache: OnceLock<String>,
    pub states_cache: OnceLock<String>,
    pub states_en_us_cache: OnceLock<String>,
}

#[derive(Debug)]
pub struct ContextTree {
    pub nodes: HashMap<NodeHandle, ContextNode>,
    next_handle: NodeHandle,
}

impl ContextNode {
    fn from_obj(
        obj: JavaObject,
        depth: i32,
        handle: NodeHandle,
        parent: Option<NodeHandle>,
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
            text_cache: OnceLock::new(),
            action_names_cache: OnceLock::new(),
            states_cache: OnceLock::new(),
            states_en_us_cache: OnceLock::new(),
        };

        if let Some(info) = node.obj.get_obj_info() {
            node.name = utf16_to_string(&info.name);
            node.role = utf16_to_string(&info.role).to_lowercase().replace(' ', "_");
            node.states = utf16_to_string(&info.states)
                .split(',')
                .map(str::to_lowercase)
                .map(|s| s.replace(' ', "_"))
                .collect();
            node.states_en_us = utf16_to_string(&info.states_en_US)
                .split(',')
                .map(str::to_lowercase)
                .map(|s| s.replace(' ', "_"))
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
        }
        node
    }

    pub(crate) fn resolve_text(&self) -> &str {
        self.text_cache
            .get_or_init(|| self.obj.get_text().unwrap_or_default())
    }

    pub(crate) fn resolve_action_names(&self) -> &str {
        self.action_names_cache.get_or_init(|| {
            let actions = match self.obj.get_actions() {
                Ok(actions) => actions,
                Err(_) => return String::default(),
            };

            actions.actionInfo[..actions.actionsCount as usize]
                .iter()
                .map(|action| {
                    utf16_to_string(&action.name)
                        .to_lowercase()
                        .replace(' ', "_")
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    pub(crate) fn resolve_states(&self) -> &str {
        self.states_cache
            .get_or_init(|| self.states.join(" "))
    }

    pub(crate) fn resolve_states_en_us(&self) -> &str {
        self.states_cache
            .get_or_init(|| self.states_en_us.join(" "))
    }


}

impl ContextTree {
    pub fn root(&self) -> &ContextNode {
        &self.nodes[&ROOT_HANDLE]
    }

    pub fn into_root(mut self) -> JavaObject {
        self.nodes
            .remove(&ROOT_HANDLE)
            .expect("Root node missing")
            .obj
    }

    pub fn from_root(root_obj: JavaObject, max_depth: Option<i32>) -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            next_handle: ROOT_HANDLE + 1,
        };

        let mut root = ContextNode::from_obj(root_obj, 0, ROOT_HANDLE, None);
        tree.build_subtree(&mut root, max_depth);
        tree.nodes.insert(ROOT_HANDLE, root);

        tree
    }

    fn build_subtree(&mut self, node: &mut ContextNode, max_depth: Option<i32>) {
        if let Some(max) = max_depth
            && node.depth >= max
        {
            return;
        }

        for i in 0..node.children_count {
            let child_obj = unsafe { node.obj.get_child_from_obj(i) };

            let handle = self.next_handle;
            self.next_handle += 1;
            let mut child_node =
                ContextNode::from_obj(child_obj, node.depth + 1, handle, Some(node.handle));

            self.build_subtree(&mut child_node, max_depth);

            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }
    }
}
