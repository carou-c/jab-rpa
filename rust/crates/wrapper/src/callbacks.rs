use std::sync::OnceLock;

use crate::context_tree::ContextTree;

pub(crate) use self::{
    event::ChangeEvent,
    register::{shutdown_event_channel, subscribe_events},
};

mod event;
mod register;

impl ContextTree {
    pub(crate) fn apply_event(&mut self, event: ChangeEvent) {
        let meta = event.meta();

        if meta.vm_id != self.vm_id {
            return;
        }

        let Some(node) = self
            .obj_to_handle
            .iter()
            .find(|&(&obj, _)| unsafe { jab_sys::IsSameObject(self.vm_id, obj, meta.source) != 0 })
            .and_then(|(_, h)| self.nodes.get_mut(h))
        else {
            return;
        };

        match &event {
            ChangeEvent::Name(_, data) => {
                node.name = data.new.clone();
            }

            ChangeEvent::Description(_, data) => {
                node.description = data.new.clone();
            }

            ChangeEvent::State(..) => {
                node.states_cache = OnceLock::new();
                node.states_en_us_cache = OnceLock::new();
                node.actions_cache = OnceLock::new();
                node.action_names_cache = OnceLock::new();

                node.refresh_info();
            }

            ChangeEvent::Text(_) => {
                node.text_cache = OnceLock::new();
            }

            ChangeEvent::Child(..) => {
                let handle = node.handle;
                self.rebuild_subtree(&handle);
            }
        }
    }
}
