use crate::context_tree::ContextTree;
use std::sync::OnceLock;

pub(crate) use self::{
    event::ChangeEvent,
    register::{shutdown_event_channel, subscribe_events},
};

mod event;
mod register;

impl ContextTree {
    pub(crate) fn apply_event(&mut self, event: ChangeEvent) {
        println!("Event Received: {:?}", event);

        let meta = event.get_meta();

        if meta.vm_id != self.vm_id {
            return;
        }
        let Some(node) = self
            .obj_to_handle
            .get(&meta.source)
            .and_then(|h| self.nodes.get_mut(h))
        else {
            return;
        };

        match event {
            ChangeEvent::Name(_, data) => {
                node.name = data.new;
            }

            ChangeEvent::Description(_, data) => {
                node.description = data.new;
            }

            ChangeEvent::State(_, data) => {
                let new_state: Vec<_> = data
                    .new
                    .split(',')
                    .map(str::to_lowercase)
                    .map(|s| s.replace(' ', "_"))
                    .collect();

                node.states = new_state.clone();
                node.states_cache = OnceLock::new();
                node.actions_cache = OnceLock::new();
                node.action_names_cache = OnceLock::new();
            }

            ChangeEvent::Text(_) => {
                node.text_cache = OnceLock::new();
            }

            _ => (),
        }
    }
}
