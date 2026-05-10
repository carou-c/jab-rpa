use crate::bindings;

pub use crate::bindings::{AccessBridgeVersionInfo, AccessibleContextInfo};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
}

pub(crate) type VmId = std::os::raw::c_long;
pub(crate) type JObject = bindings::Java_Object;

#[derive(Debug)]
pub struct JavaObject {
    pub(crate) vm_id: VmId,
    pub(crate) jobject: JObject,
}

impl Drop for JavaObject {
    fn drop(&mut self) {
        unsafe {
            bindings::ReleaseJavaObject(self.vm_id, self.jobject);
        }
    }
}
