use std::thread;
use std::{sync::Arc, time::Duration};

use parking_lot::{Condvar, Mutex, MutexGuard, WaitTimeoutResult};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindow},
    },
    core::BOOL,
};

use crate::{
    context_tree::ContextTree,
    runtime::JabRuntime,
    types::{
        AccessBridgeVersionInfo, AccessibleActions, AccessibleContextInfo, JObject, VmId,
        WindowInfo,
    },
    utils::utf16_to_string,
};

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

#[derive(Debug)]
pub struct JabWrapper {
    runtime: Arc<JabRuntime>,
}

pub struct SharedCtxTree {
    tree: Mutex<Option<ContextTree>>,
    changed: Condvar,
}

impl SharedCtxTree {
    pub fn new() -> Self {
        Self {
            tree: Mutex::new(None),
            changed: Condvar::new(),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Option<ContextTree>> {
        self.tree.lock()
    }

    pub fn wait_change_while_for<F>(
        &self,
        guard: &mut MutexGuard<Option<ContextTree>>,
        condition: F,
        timeout: Duration,
    ) -> WaitTimeoutResult
    where
        F: FnMut(&mut Option<ContextTree>) -> bool,
    {
        self.changed.wait_while_for(guard, condition, timeout)
    }
}

impl Default for SharedCtxTree {
    fn default() -> Self {
        Self::new()
    }
}

impl JabWrapper {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Result<Self, String> {
        let runtime = JabRuntime::new()?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    pub fn spawn_tree_updater(&self, tree: &Arc<SharedCtxTree>) -> thread::JoinHandle<()> {
        let rx = self.runtime.cb_rx.clone();
        let weak = Arc::downgrade(tree);
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                let Some(tree) = weak.upgrade() else {
                    return;
                };
                let mut events = vec![event];
                while let Ok(next) = rx.try_recv() {
                    events.push(next);
                }
                let mut tree_lock = tree.lock();
                if let Some(t) = tree_lock.as_mut() {
                    for e in events {
                        t.apply_event(e);
                    }
                }

                tree.changed.notify_all();
                MutexGuard::unlock_fair(tree_lock);
            }
        })
    }

    pub fn is_java_window(&self, hwnd: HWND) -> bool {
        unsafe { (IsWindow(Some(hwnd)).0 != 0) && (jab_sys::IsJavaWindow(hwnd.0 as *mut _) != 0) }
    }

    pub fn get_hwnd_from_obj(&self, obj: &JavaObject) -> HWND {
        unsafe { HWND(jab_sys::getHWNDFromAccessibleContext(obj.vm_id, obj.jobject) as _) }
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
                .filter(|&hwnd| jab_sys::IsJavaWindow(hwnd.0 as _) != 0)
                .map(|hwnd| WindowInfo {
                    hwnd,
                    title: {
                        let len = GetWindowTextLengthW(hwnd);
                        if len > 0 {
                            let mut buf: Vec<jab_sys::wchar_t> = vec![0; (len + 1) as usize];
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

    /// # Safety
    /// The one calling this must make sure the hwnd exists
    /// and the window it points to is a Java window with
    /// the right bitness
    pub unsafe fn get_root_obj_from_hwnd(&self, hwnd: HWND) -> Result<JavaObject, String> {
        unsafe {
            let mut vm_id: VmId = 0;
            let mut jobject: jab_sys::AccessibleContext = 0;
            let result =
                jab_sys::GetAccessibleContextFromHWND(hwnd.0 as _, &mut vm_id, &mut jobject);
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
}

impl JavaObject {
    pub fn get_version_info(&self) -> Result<AccessBridgeVersionInfo, String> {
        unsafe {
            let mut info: jab_sys::AccessBridgeVersionInfo = std::mem::zeroed();
            if jab_sys::GetVersionInfo(self.vm_id, &mut info) != 0 {
                Ok(info)
            } else {
                Err("Failed to get version info".to_string())
            }
        }
    }

    pub fn get_obj_info(&self) -> Result<AccessibleContextInfo, String> {
        unsafe {
            let mut info: jab_sys::AccessibleContextInfo = std::mem::zeroed();
            if jab_sys::GetAccessibleContextInfo(self.vm_id, self.jobject, &mut info) != 0 {
                Ok(info)
            } else {
                Err("Failed to get accessible context info".to_string())
            }
        }
    }

    /// # Safety
    /// The caller must verify index is within bounds
    pub unsafe fn get_child_from_obj(&self, index: i32) -> JavaObject {
        unsafe {
            let child = jab_sys::GetAccessibleChildFromContext(self.vm_id, self.jobject, index);
            JavaObject {
                vm_id: self.vm_id,
                jobject: child,
                runtime: self.runtime.clone(),
            }
        }
    }

    pub fn get_text(&self) -> Result<String, String> {
        unsafe {
            let mut text_info: jab_sys::AccessibleTextInfo = std::mem::zeroed();
            if jab_sys::GetAccessibleTextInfo(self.vm_id, self.jobject, &mut text_info, 0, 0) == 0 {
                return Err("GetAccessibleTextInfo failed".to_string());
            }

            let total_len = text_info.charCount.max(0) as usize;
            if total_len == 0 {
                return Ok(String::new());
            }

            const CHUNK: usize = 4096;
            let mut text: Vec<jab_sys::wchar_t> = Vec::with_capacity(total_len);
            let mut start = 0;

            while start < total_len {
                let chunk_len = (total_len - start).min(CHUNK);
                let end = start + chunk_len - 1;
                let mut buf: Vec<jab_sys::wchar_t> = vec![0; chunk_len + 1];

                if jab_sys::GetAccessibleTextRange(
                    self.vm_id,
                    self.jobject,
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

    pub fn get_actions(&self) -> Result<AccessibleActions, String> {
        unsafe {
            let mut actions: jab_sys::AccessibleActions = std::mem::zeroed();
            if jab_sys::getAccessibleActions(self.vm_id, self.jobject, &mut actions) == 0 {
                return Err("getAccessibleActions failed".to_string());
            }
            Ok(actions)
        }
    }

    pub fn do_action(&self, action: String) -> Result<(), String> {
        let action = action.to_lowercase();
        let outer_actions: Vec<_>;
        unsafe {
            let Ok(actions) = self.get_actions() else {
                return Err("Failed to get accessible actions".to_string());
            };
            outer_actions = actions
                .actionInfo
                .iter()
                .take(actions.actionsCount as _)
                .map(|act| utf16_to_string(&act.name))
                .collect();

            for i in 0..actions.actionsCount {
                let action_name = utf16_to_string(&actions.actionInfo[i as usize].name);
                if action_name.to_lowercase() == action {
                    let mut actions_to_do: jab_sys::AccessibleActionsToDo = std::mem::zeroed();
                    actions_to_do.actionsCount = 1;
                    actions_to_do.actions[0] = actions.actionInfo[i as usize];

                    let mut failure: i32 = 0;
                    if jab_sys::doAccessibleActions(
                        self.vm_id,
                        self.jobject,
                        &mut actions_to_do,
                        &mut failure,
                    ) != 0
                    {
                        return Ok(());
                    } else {
                        return Err(format!(
                            "Failed to perform {} action, failure index: {}",
                            action, failure
                        ));
                    }
                }
            }
        }

        Err(format!(
            "No {:?} action found. Available actions: {:?}",
            action, outer_actions
        ))
    }
}
