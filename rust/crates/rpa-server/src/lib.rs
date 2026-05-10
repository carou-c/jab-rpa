pub mod proto {
    tonic::include_proto!("jab");
}

mod context_tree;
pub mod jab_service;
mod utils;
