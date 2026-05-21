use crate::context_tree::ContextTree;
use std::sync::OnceLock;

pub(crate) use self::{
    event::CallbackChangeEvent,
    register::{shutdown_event_channel, subscribe_events},
};

mod event;
mod register;

impl ContextTree {
    pub(crate) fn apply_event(&mut self, event: CallbackChangeEvent) {
        if event.get_vm_id() != self.vm_id {
            return;
        }
        let Some(node) = self
            .obj_to_handle
            .get(&event.get_source_jobject())
            .and_then(|h| self.nodes.get_mut(h))
        else {
            return;
        };

        match event {
            CallbackChangeEvent::Name { new_name, .. } => {
                node.name = new_name;
            }

            CallbackChangeEvent::Description {
                new_description, ..
            } => {
                node.description = new_description;
            }

            CallbackChangeEvent::State { new_state, .. } => {
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

            CallbackChangeEvent::Text { .. } => {
                node.text_cache = OnceLock::new();
            }
        }
    }
}
