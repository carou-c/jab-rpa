use std::collections::HashMap;

use rayon::prelude::*;

use super::{NodeHandle, ROOT_HANDLE, node::ContextNode};
use crate::selector::Selector;
use crate::types::JavaObject;

#[derive(Debug)]
pub struct ContextTree {
    pub nodes: HashMap<NodeHandle, ContextNode>,
    next_handle: NodeHandle,
}

impl ContextTree {
    pub fn root(&self) -> &ContextNode {
        self.nodes.get(&ROOT_HANDLE).expect("Root node missing")
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

    pub fn node_matches(&self, node: &ContextNode, selector: &Selector) -> bool {
        use super::matcher::matches_selector;

        matches_selector(node, selector, None, self)
    }

    pub fn get_nodes(&self, selector: &Selector) -> Vec<&ContextNode> {
        self.nodes
            .par_iter()
            .filter_map(|(_, node)| self.node_matches(node, selector).then_some(node))
            .collect()
    }

    pub fn get_node(&self, selector: &Selector) -> Option<&ContextNode> {
        self.nodes
            .par_iter()
            .find_map_any(|(_, node)| self.node_matches(node, selector).then_some(node))
    }
}
