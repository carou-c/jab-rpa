use windows::Win32::Foundation::HWND;

pub use jab_sys::{
    AccessBridgeVersionInfo, AccessibleActionInfo, AccessibleActions, AccessibleContextInfo,
};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: String,
}

pub(crate) type VmId = std::os::raw::c_long;
pub(crate) type JObject = jab_sys::Java_Object;
