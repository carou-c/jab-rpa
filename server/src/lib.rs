#[allow(warnings)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod context_tree;
pub mod jab_service;
pub mod jab_wrapper;
pub mod types;
mod utils;

pub mod proto {
    tonic::include_proto!("jab");
}

#[derive(Debug, Clone)]
pub struct JabCallbackEvent {
    pub event_type: String,
    pub vm_id: i32,
    pub context_handle: u64,
    pub message: String,
    pub event_time: i64,
}
