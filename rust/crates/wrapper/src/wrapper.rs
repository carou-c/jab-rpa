use std::sync::Arc;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindow},
    },
    core::BOOL,
};

use crate::{
    bindings,
    runtime::JabRuntime,
    types::{AccessBridgeVersionInfo, AccessibleContextInfo, WindowInfo},
    utils::utf16_to_string,
};

type VmId = std::os::raw::c_long;
type JObject = bindings::Java_Object;

#[derive(Debug)]
pub struct JavaObject {
    vm_id: VmId,
    jobject: JObject,
    runtime: Arc<JabRuntime>,
}

impl Drop for JavaObject {
    fn drop(&mut self) {
        unsafe {
            bindings::ReleaseJavaObject(self.vm_id, self.jobject);
        }
    }
}

#[derive(Debug)]
pub struct JabWrapper {
    runtime: Arc<JabRuntime>,
}

impl JabWrapper {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(JabRuntime::new()),
        }
    }

    pub fn list_java_windows(&self) -> Vec<WindowInfo> {
        unsafe {
            let mut hwnds: Vec<HWND> = Vec::new();

            extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
                let hwnds: &mut Vec<HWND> = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
                if unsafe { IsWindow(Some(hwnd)).0 } != 0 {
                    hwnds.push(hwnd);
                }
                BOOL(1)
            }

            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut hwnds as *mut _ as isize));

            hwnds
                .into_iter()
                .filter(|&hwnd| bindings::IsJavaWindow(hwnd.0 as _) != 0)
                .map(|hwnd| WindowInfo {
                    hwnd: hwnd.0 as u64,
                    title: {
                        let len = GetWindowTextLengthW(hwnd);
                        if len > 0 {
                            let mut buf: Vec<bindings::wchar_t> = vec![0; (len + 1) as usize];
                            GetWindowTextW(hwnd, &mut buf);
                            utf16_to_string(&buf)
                        } else {
                            String::new()
                        }
                    },
                })
                .collect()
        }
    }

    pub fn get_root_obj_from_window(&self, window: WindowInfo) -> Result<JavaObject, String> {
        unsafe {
            let mut vm_id: VmId = 0;
            let mut jobject: bindings::AccessibleContext = 0;
            let result =
                bindings::GetAccessibleContextFromHWND(window.hwnd as _, &mut vm_id, &mut jobject);
            if result != 0 {
                Ok(JavaObject {
                    vm_id,
                    jobject,
                    runtime: self.runtime.clone(),
                })
            } else {
                Err(format!("GetAccessibleContextFromHWND returned {}", result))
            }
        }
    }

    pub fn get_obj_info(&self, obj: &JavaObject) -> Option<AccessibleContextInfo> {
        unsafe {
            let mut info: bindings::AccessibleContextInfo = std::mem::zeroed();
            if bindings::GetAccessibleContextInfo(obj.vm_id, obj.jobject, &mut info) != 0 {
                Some(info)
            } else {
                None
            }
        }
    }

    /// # Safety
    /// The caller must verify index is within bounds
    pub unsafe fn get_child_from_obj(&self, obj: &JavaObject, index: i32) -> JavaObject {
        unsafe {
            let child = bindings::GetAccessibleChildFromContext(obj.vm_id, obj.jobject, index);
            JavaObject {
                vm_id: obj.vm_id,
                jobject: child,
                runtime: self.runtime.clone(),
            }
        }
    }

    pub fn click_element(&self, obj: &JavaObject) -> Result<(), String> {
        unsafe {
            let mut actions: bindings::AccessibleActions = std::mem::zeroed();
            if bindings::getAccessibleActions(obj.vm_id, obj.jobject, &mut actions) == 0 {
                return Err("Failed to get accessible actions".to_string());
            }

            for i in 0..actions.actionsCount {
                let action_name = utf16_to_string(&actions.actionInfo[i as usize].name);
                if action_name.to_lowercase().contains("click") {
                    let mut actions_to_do: bindings::AccessibleActionsToDo = std::mem::zeroed();
                    actions_to_do.actionsCount = 1;
                    actions_to_do.actions[0] = actions.actionInfo[i as usize];

                    let mut failure: i32 = 0;
                    if bindings::doAccessibleActions(
                        obj.vm_id,
                        obj.jobject,
                        &mut actions_to_do,
                        &mut failure,
                    ) != 0
                    {
                        return Ok(());
                    } else {
                        return Err(format!(
                            "Failed to perform click action, failure index: {}",
                            failure
                        ));
                    }
                }
            }
        }

        Err("No click action found for element".to_string())
    }

    pub fn get_text(&self, obj: &JavaObject) -> Result<String, String> {
        unsafe {
            let mut text_info: bindings::AccessibleTextInfo = std::mem::zeroed();
            if bindings::GetAccessibleTextInfo(obj.vm_id, obj.jobject, &mut text_info, 0, 0) == 0 {
                return Err("GetAccessibleTextInfo failed".to_string());
            }

            let total_len = text_info.charCount.max(0) as usize;
            if total_len == 0 {
                return Ok(String::new());
            }

            const CHUNK: usize = 4096;
            let mut text: Vec<bindings::wchar_t> = Vec::with_capacity(total_len);
            let mut start = 0;

            while start < total_len {
                let chunk_len = (total_len - start).min(CHUNK);
                let end = start + chunk_len - 1;
                let mut buf: Vec<bindings::wchar_t> = vec![0; chunk_len + 1];

                if bindings::GetAccessibleTextRange(
                    obj.vm_id,
                    obj.jobject,
                    start as i32,
                    end as i32,
                    buf.as_mut_ptr(),
                    (chunk_len + 1) as i16,
                ) == 0
                {
                    return Err(format!(
                        "GetAccessibleTextRange failed at start={}, end={}, chunk_len={}",
                        start, end, chunk_len
                    ));
                }
                let actual_len = buf.iter().position(|&c| c == 0).unwrap_or(chunk_len);

                text.extend_from_slice(&buf[..actual_len]);
                start += chunk_len;
            }
            Ok(utf16_to_string(&text))
        }
    }

    pub fn get_version_info(&self, obj: &JavaObject) -> Result<AccessBridgeVersionInfo, String> {
        unsafe {
            let mut info: bindings::AccessBridgeVersionInfo = std::mem::zeroed();
            if bindings::GetVersionInfo(obj.vm_id, &mut info) != 0 {
                Ok(info)
            } else {
                Err("Failed to get version info".to_string())
            }
        }
    }
}
