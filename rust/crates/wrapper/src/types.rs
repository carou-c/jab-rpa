pub use jab_sys::{
    AccessBridgeVersionInfo, AccessibleActionInfo, AccessibleActions, AccessibleContextInfo,
};
pub use crate::wrapper::JavaObject;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub(crate) hwnd: u64,
    pub title: String,
}

impl WindowInfo {
    /// # Safety
    /// The one calling this must make sure the hwnd exists
    /// and the window it points to is a Java window with
    /// the right bitness
    pub unsafe fn new(hwnd: u64, title: String) -> Self {
        Self {
            hwnd, title
        }
    }

    pub fn get_hwnd(&self) -> u64 {
        self.hwnd
    }
}
