use std::sync::Arc;

use crate::runtime::JabRuntime;

pub use jab_sys::{
    AccessBridgeVersionInfo, AccessibleActionInfo, AccessibleActions, AccessibleContextInfo,
};

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
        Self { hwnd, title }
    }

    pub fn get_hwnd(&self) -> u64 {
        self.hwnd
    }
}

pub(crate) type VmId = std::os::raw::c_long;
pub(crate) type JObject = jab_sys::Java_Object;

#[derive(Debug)]
pub struct JavaObject {
    pub(crate) vm_id: VmId,
    pub(crate) jobject: JObject,
    pub(crate) runtime: Arc<JabRuntime>,
}

impl Drop for JavaObject {
    fn drop(&mut self) {
        unsafe {
            jab_sys::ReleaseJavaObject(self.vm_id, self.jobject);
        }
    }
}
