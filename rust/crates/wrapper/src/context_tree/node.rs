use std::sync::OnceLock;

use super::NodeHandle;
use crate::utils::utf16_to_string;
use crate::wrapper::JavaObject;

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
    pub(crate) text_cache: OnceLock<String>,
    pub(crate) actions_cache: OnceLock<Vec<String>>,
    pub(crate) action_names_cache: OnceLock<String>,
    pub(crate) states_cache: OnceLock<String>,
    pub(crate) states_en_us_cache: OnceLock<String>,
}

impl ContextNode {
    pub(crate) fn from_obj(
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
            actions_cache: OnceLock::new(),
            action_names_cache: OnceLock::new(),
            states_cache: OnceLock::new(),
            states_en_us_cache: OnceLock::new(),
        };

        node.refresh_info();
        node.children.reserve(node.children_count.max(0) as usize);
        node
    }

    pub fn refresh_info(&mut self) {
        if let Ok(info) = self.obj.get_obj_info() {
            self.name = utf16_to_string(&info.name);
            self.role = utf16_to_string(&info.role).to_lowercase().replace(' ', "_");
            self.states = utf16_to_string(&info.states)
                .split(',')
                .map(str::to_lowercase)
                .map(|s| s.replace(' ', "_"))
                .collect();
            self.states_en_us = utf16_to_string(&info.states_en_US)
                .split(',')
                .map(str::to_lowercase)
                .map(|s| s.replace(' ', "_"))
                .collect();
            self.description = utf16_to_string(&info.description);
            self.x = info.x;
            self.y = info.y;
            self.width = info.width;
            self.height = info.height;
            self.accessible_action = info.accessibleAction != 0;
            self.accessible_text = info.accessibleText != 0;
            self.accessible_selection = info.accessibleSelection != 0;
            self.children_count = info.childrenCount;
            self.index_in_parent = info.indexInParent;
        }

        self.text_cache = OnceLock::new();
        self.actions_cache = OnceLock::new();
        self.action_names_cache = OnceLock::new();
        self.states_cache = OnceLock::new();
        self.states_en_us_cache = OnceLock::new();
    }

    pub fn resolve_text(&self) -> &str {
        self.text_cache
            .get_or_init(|| self.obj.get_text().unwrap_or_default())
    }

    pub fn resolve_actions(&self) -> &[String] {
        self.actions_cache.get_or_init(|| {
            let actions = match self.obj.get_actions() {
                Ok(actions) => actions,
                Err(_) => return Vec::new(),
            };

            actions
                .actionInfo
                .iter()
                .take(actions.actionsCount.max(0) as _)
                .map(|action| {
                    utf16_to_string(&action.name)
                        .to_lowercase()
                        .replace(' ', "_")
                })
                .collect::<Vec<_>>()
        })
    }

    pub fn resolve_action_names(&self) -> &str {
        self.action_names_cache
            .get_or_init(|| self.resolve_actions().join(" "))
    }

    pub fn resolve_states(&self) -> &str {
        self.states_cache.get_or_init(|| self.states.join(" "))
    }

    pub fn resolve_states_en_us(&self) -> &str {
        self.states_en_us_cache
            .get_or_init(|| self.states_en_us.join(" "))
    }
}
