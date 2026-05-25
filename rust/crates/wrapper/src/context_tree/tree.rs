use std::collections::HashMap;

use super::{NodeHandle, ROOT_HANDLE, node::ContextNode};
use crate::selector::Selector;
use crate::types::{JObject, VmId};
use crate::wrapper::JavaObject;

#[derive(Debug)]
pub struct ContextTree {
    pub nodes: HashMap<NodeHandle, ContextNode>,
    pub(crate) obj_to_handle: HashMap<JObject, NodeHandle>,
    pub(crate) vm_id: VmId,
    next_handle: NodeHandle,
}

impl ContextTree {
    pub fn root(&self) -> Result<&ContextNode, String> {
        self.nodes
            .get(&ROOT_HANDLE)
            .ok_or("Root node missing".to_string())
    }

    pub fn into_root(mut self) -> Result<JavaObject, String> {
        self.nodes
            .remove(&ROOT_HANDLE)
            .map(|root| root.obj)
            .ok_or("Root node missing".to_string())
    }

    pub fn from_root(root_obj: JavaObject, max_depth: Option<i32>) -> Self {
        let mut tree = Self {
            nodes: HashMap::new(),
            obj_to_handle: HashMap::new(),
            vm_id: root_obj.vm_id,
            next_handle: ROOT_HANDLE + 1,
        };

        let mut root = ContextNode::from_obj(root_obj, 0, ROOT_HANDLE, None);
        tree.build_subtree(&mut root, max_depth);
        tree.obj_to_handle.insert(root.obj.jobject, ROOT_HANDLE);
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

            self.obj_to_handle.insert(child_node.obj.jobject, handle);
            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }
    }

    pub(crate) fn rebuild_subtree(&mut self, handle: &NodeHandle) {
        let Some(mut node) = self.nodes.remove(handle) else {
            return;
        };
        for h in self.subtree(&node) {
            let Some(dropped) = self.nodes.remove(&h) else {
                continue;
            };
            self.obj_to_handle.remove(&dropped.obj.jobject);
        }

        node.children.clear();
        node.refresh_info();
        node.children.reserve(node.children_count.max(0) as usize);

        for i in 0..node.children_count {
            let child_obj = unsafe { node.obj.get_child_from_obj(i) };

            let handle = self.next_handle;
            self.next_handle += 1;
            let mut child_node =
                ContextNode::from_obj(child_obj, node.depth + 1, handle, Some(node.handle));

            self.build_subtree(&mut child_node, None);

            self.obj_to_handle.insert(child_node.obj.jobject, handle);
            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }

        self.nodes.insert(*handle, node);
    }

    pub fn subtree(&self, node: &ContextNode) -> Vec<NodeHandle> {
        let children = node.children.iter().filter_map(|h| self.nodes.get(h));

        node.children
            .clone()
            .into_iter()
            .chain(children.flat_map(|child| self.subtree(child)))
            .collect()
    }

    pub fn node_matches(&self, node: &ContextNode, selector: &Selector) -> bool {
        use super::matcher::matches_selector;
        matches_selector(node, selector, None, self)
    }

    pub fn get_nodes(&self, selector: &Selector) -> Vec<&ContextNode> {
        self.nodes
            .iter()
            .filter_map(|(_, node)| self.node_matches(node, selector).then_some(node))
            .collect()
    }

    pub fn get_node(&self, selector: &Selector) -> Option<&ContextNode> {
        self.nodes
            .iter()
            .find_map(|(_, node)| self.node_matches(node, selector).then_some(node))
    }
}
