use std::collections::HashMap;
use std::sync::OnceLock;

use super::{NodeHandle, ROOT_HANDLE, node::ContextNode};
use crate::callbacks::CallbackChangeEvent;
use crate::selector::Selector;
use crate::types::{JObject, VmId};
use crate::wrapper::JavaObject;

#[derive(Debug)]
pub struct ContextTree {
    pub nodes: HashMap<NodeHandle, ContextNode>,
    obj_to_handle: HashMap<JObject, NodeHandle>,
    vm_id: VmId,
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
            node.subtree.extend(&child_node.subtree);
            node.subtree.push(handle);

            self.obj_to_handle.insert(child_node.obj.jobject, handle);
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
            .iter()
            .filter_map(|(_, node)| self.node_matches(node, selector).then_some(node))
            .collect()
    }

    pub fn get_node(&self, selector: &Selector) -> Option<&ContextNode> {
        self.nodes
            .iter()
            .find_map(|(_, node)| self.node_matches(node, selector).then_some(node))
    }

    pub(crate) fn apply_event(&mut self, event: CallbackChangeEvent) {
        match event {
            CallbackChangeEvent::Name {
                vm_id,
                source_jobject,
                old_name: _,
                new_name,
            } => {
                if vm_id != self.vm_id {
                    return;
                }
                let Some(node) = self
                    .obj_to_handle
                    .get(&source_jobject)
                    .and_then(|h| self.nodes.get_mut(h))
                else {
                    return;
                };
                node.name = new_name;
            }

            CallbackChangeEvent::Description {
                vm_id,
                source_jobject,
                old_description: _,
                new_description,
            } => {
                if vm_id != self.vm_id {
                    return;
                }
                let Some(node) = self
                    .obj_to_handle
                    .get(&source_jobject)
                    .and_then(|h| self.nodes.get_mut(h))
                else {
                    return;
                };

                node.description = new_description;
            }

            CallbackChangeEvent::State {
                vm_id,
                source_jobject,
                old_state: _,
                new_state,
            } => {
                if vm_id != self.vm_id {
                    return;
                }
                let Some(node) = self
                    .obj_to_handle
                    .get(&source_jobject)
                    .and_then(|h| self.nodes.get_mut(h))
                else {
                    return;
                };

                let new_state: Vec<_> = new_state
                    .split(',')
                    .map(str::to_lowercase)
                    .map(|s| s.replace(' ', "_"))
                    .collect();

                node.states = new_state.clone();
                node.states_cache = OnceLock::new();
                node.actions_cache = OnceLock::new();
                node.action_names_cache = OnceLock::new();
            }

            CallbackChangeEvent::Text {
                vm_id,
                source_jobject,
            } => {
                if vm_id != self.vm_id {
                    return;
                }
                let Some(node) = self
                    .obj_to_handle
                    .get(&source_jobject)
                    .and_then(|h| self.nodes.get_mut(h))
                else {
                    return;
                };

                node.text_cache = OnceLock::new();
            }
        }
    }
}
