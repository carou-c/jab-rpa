use std::collections::HashMap;
use std::sync::Arc;

use crate::types::JavaObject;
use crate::utils::utf16_to_string;
use crate::wrapper::JabWrapper;

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
            node.role = utf16_to_string(&info.role)
                .to_lowercase()
                .replace(' ', "_");
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

    pub fn from_root(root_obj: JavaObject, max_depth: Option<i32>, jab: &Arc<JabWrapper>) -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            next_handle: ROOT_HANDLE + 1,
        };

        let mut root = ContextNode::from_obj(root_obj, 0, ROOT_HANDLE, None, jab);
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
                ContextNode::from_obj(child_obj, node.depth + 1, handle, Some(node.handle), jab);

            self.build_subtree(&mut child_node, max_depth, jab);

            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }
    }
}
