#[allow(warnings)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod jab_wrapper;
pub mod jab_service;
pub mod context_tree;

#[derive(Debug, Clone)]
pub struct JabCallbackEvent {
    pub event_type: String,
    pub vm_id: i32,
    pub context_handle: u64,
    pub message: String,
    pub event_time: i64,
}
