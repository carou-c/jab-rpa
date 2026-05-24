pub mod proto {
    tonic::include_proto!("jab");
}

mod service;
mod types;
pub use service::JabService;
