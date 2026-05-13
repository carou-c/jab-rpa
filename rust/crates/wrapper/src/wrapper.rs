use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            DispatchMessageW, EnumWindows, GetMessageW, GetWindowTextLengthW, GetWindowTextW,
            IsWindow, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, TranslateMessage, WM_QUIT,
        },
    },
    core::BOOL,
};

use crate::{
    bindings,
    types::{AccessBridgeVersionInfo, AccessibleContextInfo, JavaObject, VmId, WindowInfo},
    utils::utf16_to_string,
};

pub struct JabWrapper {
    pub(crate) initialized: AtomicBool,
    message_pump_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    message_pump_thread_id: Mutex<Option<u32>>,
}

impl JabWrapper {
    pub fn new() -> Arc<Self> {
        let wrapper = Arc::new(Self {
            initialized: AtomicBool::new(false),
            message_pump_handle: Mutex::new(None),
            message_pump_thread_id: Mutex::new(None),
        });

        // Channel to synchronize initialization
        let (init_tx, init_rx) = mpsc::channel::<bool>();

        // Start Windows message pump in dedicated thread (same thread will call initializeAccessBridge)
        let wrapper_clone = wrapper.clone();
        let pump_handle = std::thread::spawn(move || {
            // Store thread ID for later shutdown
            let thread_id = unsafe { GetCurrentThreadId() };
            {
                let mut tid = wrapper_clone.message_pump_thread_id.lock().unwrap();
                *tid = Some(thread_id);
            }

            unsafe {
                let _ = PeekMessageW(&mut std::mem::zeroed(), None, 0, 0, PM_NOREMOVE);
            }

            // Initialize JAB on this thread
            let init_result = unsafe { bindings::initializeAccessBridge() };
            let success = init_result != 0;
            let _ = init_tx.send(success);

            if success {
                // Run message pump loop
                run_message_pump();
            }

            // Shutdown JAB
            unsafe {
                bindings::shutdownAccessBridge();
            }
        });

        {
            let mut handle = wrapper.message_pump_handle.lock().unwrap();
            *handle = Some(pump_handle);
        }

        // Wait for initialization to complete
        match init_rx.recv() {
            Ok(true) => {
                wrapper.initialized.store(true, Ordering::SeqCst);
            }
            Ok(false) => panic!("Failed to initialize JAB"),
            Err(_) => panic!("Message pump thread crashed during initialization"),
        }

        wrapper
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
                Ok(JavaObject { vm_id, jobject })
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

impl Drop for JabWrapper {
    fn drop(&mut self) {
        // Post WM_QUIT to the message pump thread to exit the loop
        let thread_id = {
            let tid = self.message_pump_thread_id.lock().unwrap();
            *tid
        };

        if let Some(tid) = thread_id {
            unsafe {
                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        // Wait for the message pump thread to finish
        let handle = {
            let mut h = self.message_pump_handle.lock().unwrap();
            h.take()
        };

        if let Some(h) = handle {
            let _ = h.join();
        }
    }
}

fn run_message_pump() {
    unsafe {
        let mut msg = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, None, 0, 0);
            if result.0 <= 0 {
                if result.0 < 0 {
                    eprintln!("Message pump error: {}", result.0);
                }
                break; // WM_QUIT
            } else {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
