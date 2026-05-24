pub mod proto {
    tonic::include_proto!("jab");
}

mod service;
mod types;
mod utils;

pub use service::JabService;

use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
