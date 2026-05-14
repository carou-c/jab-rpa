pub use crate::bindings::{AccessBridgeVersionInfo, AccessibleContextInfo};
pub use crate::wrapper::JavaObject;


#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub(crate) hwnd: u64,
    pub title: String,
}
