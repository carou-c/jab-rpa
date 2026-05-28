use std::collections::HashMap;
use std::sync::OnceLock;

use rayon::prelude::*;

use crate::context_tree::ContextTree;

pub(crate) use self::{
    event::ChangeEvent,
    register::{shutdown_event_channel, subscribe_events},
};

mod event;
mod register;

impl ContextTree {
    pub(crate) fn apply_events(&mut self, events: Vec<ChangeEvent>) {
        let mut dedup = HashMap::new();
        for (idx, event_type, handle, data) in
            events.into_iter().enumerate().filter_map(|(idx, event)| {
                let meta = event.meta();

                if meta.vm_id != self.vm_id {
                    return None;
                }

                let handle = self
                    .obj_to_handle
                    .par_iter()
                    .find_any(|&(&obj, _)| unsafe {
                        jab_sys::IsSameObject(self.vm_id, obj, meta.source) != 0
                    })
                    .map(|(_, h)| *h)?;

                match &event {
                    ChangeEvent::Name(_, data) => {
                        Some((idx, "name", handle, Some(data.new.clone())))
                    }
                    ChangeEvent::Description(_, data) => {
                        Some((idx, "description", handle, Some(data.new.clone())))
                    }
                    ChangeEvent::State(..) => Some((idx, "state", handle, None)),
                    ChangeEvent::Text(_) => Some((idx, "text", handle, None)),
                    ChangeEvent::Child(..) => Some((idx, "child", handle, None)),
                }
            })
        {
            dedup.insert((event_type, handle), (idx, data));
        }

        let mut dedup = dedup.into_iter().collect::<Vec<_>>();
        dedup.sort_by_key(|((_, _), (idx, _))| *idx);

        for ((event_type, handle), (_, new)) in dedup {
            let Some(node) = self.nodes.get_mut(&handle) else {
                continue;
            };

            match event_type {
                "name" => {
                    node.name = new.unwrap_or_else(String::new);
                }

                "description" => {
                    node.description = new.unwrap_or_else(String::new);
                }

                "state" => {
                    node.states_cache = OnceLock::new();
                    node.states_en_us_cache = OnceLock::new();
                    node.actions_cache = OnceLock::new();
                    node.action_names_cache = OnceLock::new();

                    node.refresh_info();
                }

                "text" => {
                    node.text_cache = OnceLock::new();
                }

                "child" => {
                    let handle = node.handle;
                    self.rebuild_subtree(&handle);
                }

                _ => (),
            }
        }
    }
}
