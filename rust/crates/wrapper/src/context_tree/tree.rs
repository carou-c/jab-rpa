use std::collections::{HashMap, HashSet};

use rayon::prelude::*;

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
    role_to_handle: HashMap<String, HashSet<NodeHandle>>,
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
            role_to_handle: HashMap::new(),
        };

        let mut root = ContextNode::from_obj(root_obj, 0, ROOT_HANDLE, None);
        tree.build_subtree(&mut root, max_depth);
        tree.role_to_handle
            .entry(root.role.clone())
            .or_default()
            .insert(ROOT_HANDLE);
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

        let count = node.children_count.max(0);
        let start = self.next_handle;
        self.next_handle += count as NodeHandle;

        let mut child_nodes: Vec<ContextNode> = (0..count)
            .into_par_iter()
            .map(|i| {
                let child_obj = unsafe { node.obj.get_child_from_obj(i) };
                ContextNode::from_obj(
                    child_obj,
                    node.depth + 1,
                    start + i as NodeHandle,
                    Some(node.handle),
                )
            })
            .collect();

        for mut child_node in child_nodes.drain(..) {
            let handle = child_node.handle;
            self.build_subtree(&mut child_node, max_depth);
            self.role_to_handle
                .entry(child_node.role.clone())
                .or_default()
                .insert(child_node.handle);
            self.obj_to_handle.insert(child_node.obj.jobject, handle);
            self.nodes.insert(handle, child_node);
            node.children.push(handle);
        }
    }

    pub(crate) fn rebuild_subtree(&mut self, handle: &NodeHandle) {
        let Some(mut node) = self.nodes.remove(handle) else {
            return;
        };

        let subtree = self.subtree(&node).collect::<Vec<_>>();
        for h in &subtree {
            if let Some(dropped) = self.nodes.remove(h) {
                self.obj_to_handle.remove(&dropped.obj.jobject);
                self.role_to_handle
                    .entry(dropped.role)
                    .and_modify(|handles| {
                        handles.remove(&dropped.handle);
                    });
            };
        }

        node.children.clear();
        node.refresh_info();
        node.children.reserve(node.children_count.max(0) as usize);

        self.build_subtree(&mut node, None);

        self.nodes.insert(*handle, node);
    }

    pub fn subtree<'a>(
        &'a self,
        node: &'a ContextNode,
    ) -> Box<dyn Iterator<Item = NodeHandle> + 'a> {
        let children_subtrees = node
            .children
            .iter()
            .filter_map(|h| self.nodes.get(h))
            .flat_map(|child| self.subtree(child));

        Box::new(node.children.iter().cloned().chain(children_subtrees))
    }

    pub fn node_matches(&self, node: &ContextNode, selector: &Selector) -> bool {
        use super::matcher::matches_selector;
        matches_selector(node, selector, None, self)
    }

    fn candidates(&self, selector: &Selector) -> Option<Vec<&ContextNode>> {
        let roles = selector
            .alternatives
            .iter()
            .map(|complex| &complex.last.role)
            .collect::<HashSet<_>>();

        if roles.contains(&None) {
            None
        } else {
            let roles = roles.into_iter().filter_map(|role| role.as_ref());

            let candidates = roles
                .filter_map(|role| self.role_to_handle.get(role))
                .flat_map(|hs| hs.iter().filter_map(|h| self.nodes.get(h)));

            Some(candidates.collect())
        }
    }

    pub fn get_nodes(&self, selector: &Selector) -> Vec<&ContextNode> {
        match self.candidates(selector) {
            Some(candidates) => candidates
                .into_par_iter()
                .filter_map(|node| self.node_matches(node, selector).then_some(node))
                .collect(),
            None => self
                .nodes
                .par_iter()
                .filter_map(|(_, node)| self.node_matches(node, selector).then_some(node))
                .collect(),
        }
    }

    pub fn get_node(&self, selector: &Selector) -> Option<&ContextNode> {
        match self.candidates(selector) {
            Some(candidates) => candidates
                .into_par_iter()
                .find_any(|node| self.node_matches(node, selector)),
            None => self
                .nodes
                .par_iter()
                .find_map_any(|(_, node)| self.node_matches(node, selector).then_some(node)),
        }
    }
}
