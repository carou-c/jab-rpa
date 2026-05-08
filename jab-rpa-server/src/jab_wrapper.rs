use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::mpsc;

use crate::bindings;
use crate::types::WindowInfo;
use crate::utils::utf16_to_string;

type VmId = std::os::raw::c_long;
type JObject = bindings::Java_Object;

#[derive(Debug)]
pub struct JavaObject {
    vm_id: VmId,
    jobject: JObject,
}

impl Drop for JavaObject {
    fn drop(&mut self) {
        eprintln!("Releasing vm_id={}, context={}", self.vm_id, self.jobject);
        unsafe {
            bindings::ReleaseJavaObject(self.vm_id, self.jobject);
        }
    }
}

// Global weak reference to JabWrapper for callbacks
static mut JAB_WRAPPER: *mut Weak<JabWrapper> = std::ptr::null_mut();

pub struct JabWrapper {
    pub initialized: AtomicBool,
    message_pump_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    message_pump_thread_id: Mutex<Option<u32>>,
    event_tx: Mutex<Option<mpsc::Sender<crate::JabCallbackEvent>>>,
}

unsafe impl Send for JabWrapper {}
unsafe impl Sync for JabWrapper {}

impl JabWrapper {
    pub fn new() -> Arc<Self> {
        let wrapper = Arc::new(Self {
            initialized: AtomicBool::new(false),
            message_pump_handle: Mutex::new(None),
            message_pump_thread_id: Mutex::new(None),
            event_tx: Mutex::new(None),
        });

        // Set global weak reference to wrapper
        unsafe {
            JAB_WRAPPER = Box::into_raw(Box::new(Arc::downgrade(&wrapper))) as *mut _;
        }

        // Channel to synchronize initialization
        let (init_tx, init_rx) = std::sync::mpsc::channel::<bool>();

        // Start Windows message pump in dedicated thread (same thread will call initializeAccessBridge)
        let wrapper_clone = wrapper.clone();
        let pump_handle = std::thread::spawn(move || {
            // Store thread ID for later shutdown
            let thread_id = unsafe { winapi::um::processthreadsapi::GetCurrentThreadId() };
            {
                let mut tid = wrapper_clone.message_pump_thread_id.lock().unwrap();
                *tid = Some(thread_id);
            }

            // Initialize JAB on this thread (required for callbacks to work)
            let init_result = unsafe { bindings::initializeAccessBridge() };
            let success = init_result != 0;
            let _ = init_tx.send(success);

            if success {
                // Run message pump loop
                run_message_pump();
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
                // wrapper.register_callbacks();
            }
            Ok(false) => panic!("Failed to initialize JAB"),
            Err(_) => panic!("Message pump thread crashed during initialization"),
        }

        wrapper
    }

    // fn register_callbacks(&self) {
    //     unsafe {
    //         bindings::SetFocusGained(Some(focus_gained_cb));
    //         bindings::SetFocusLost(Some(focus_lost_cb));
    //         bindings::SetCaretUpdate(Some(caret_update_cb));
    //         bindings::SetMouseClicked(Some(mouse_clicked_cb));
    //         bindings::SetMouseEntered(Some(mouse_entered_cb));
    //         bindings::SetMouseExited(Some(mouse_exited_cb));
    //         bindings::SetMousePressed(Some(mouse_pressed_cb));
    //         bindings::SetMouseReleased(Some(mouse_released_cb));
    //         bindings::SetPropertyNameChange(Some(property_name_change_cb));
    //         bindings::SetPropertyDescriptionChange(Some(property_description_change_cb));
    //         bindings::SetPropertyStateChange(Some(property_state_change_cb));
    //         bindings::SetPropertyValueChange(Some(property_value_change_cb));
    //         bindings::SetPropertySelectionChange(Some(property_selection_change_cb));
    //         bindings::SetPropertyTextChange(Some(property_text_change_cb));
    //         bindings::SetPropertyCaretChange(Some(property_caret_change_cb));
    //         bindings::SetPropertyVisibleDataChange(Some(property_visible_data_change_cb));
    //         bindings::SetPropertyChildChange(Some(property_child_change_cb));
    //         bindings::SetPropertyActiveDescendentChange(Some(
    //             property_active_descendent_change_cb,
    //         ));
    //         bindings::SetPropertyTableModelChange(Some(property_table_model_change_cb));
    //         bindings::SetJavaShutdown(Some(java_shutdown_cb));
    //     }
    // }

    pub fn set_event_sender(&self, tx: mpsc::Sender<crate::JabCallbackEvent>) {
        let mut event_tx = self.event_tx.lock().unwrap();
        *event_tx = Some(tx);
    }

    pub fn list_java_windows(&self) -> Vec<WindowInfo> {
        unsafe {
            use winapi::ctypes::wchar_t;
            use winapi::shared::minwindef::LPARAM;
            use winapi::shared::windef::HWND;
            use winapi::um::winuser::{
                EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindow,
            };

            let mut hwnds: Vec<HWND> = Vec::new();

            extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
                let hwnds = unsafe { &mut *(lparam as *mut Vec<HWND>) };
                if unsafe { IsWindow(hwnd) } != 0 {
                    hwnds.push(hwnd);
                }
                1
            }

            EnumWindows(Some(enum_proc), &mut hwnds as *mut _ as isize);

            hwnds
                .into_iter()
                .filter(|&hwnd| bindings::IsJavaWindow(hwnd as _) != 0)
                .map(|hwnd| WindowInfo {
                    hwnd: hwnd as u64,
                    title: {
                        let len = GetWindowTextLengthW(hwnd);
                        if len > 0 {
                            let mut buf: Vec<wchar_t> = vec![0; (len + 1) as usize];
                            GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
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

    pub fn get_obj_info(&self, obj: &JavaObject) -> Option<bindings::AccessibleContextInfo> {
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
                let action_name = String::from_utf16_lossy(&actions.actionInfo[i as usize].name);
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

    pub fn type_text(&self, obj: &JavaObject, text: &str) -> Result<(), String> {
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            if bindings::setTextContents(obj.vm_id, obj.jobject, text_wide.as_ptr()) != 0 {
                Ok(())
            } else {
                Err("Failed to set text contents".to_string())
            }
        }
    }

    pub fn get_version_info(
        &self,
        obj: &JavaObject,
    ) -> Result<bindings::AccessBridgeVersionInfo, String> {
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
                winapi::um::winuser::PostThreadMessageW(tid, winapi::um::winuser::WM_QUIT, 0, 0);
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

        // Clean up global weak reference
        unsafe {
            if !JAB_WRAPPER.is_null() {
                let _ = Box::from_raw(JAB_WRAPPER);
                JAB_WRAPPER = std::ptr::null_mut();
            }
        }

        // Call shutdownAccessBridge after the message pump has exited
        unsafe {
            bindings::shutdownAccessBridge();
        }
    }
}

// Standalone function to send callback events via the global weak reference
// fn send_callback_event(mut event: crate::JabCallbackEvent) {
//     unsafe {
//         if !JAB_WRAPPER.is_null()
//             && let Some(wrapper) = (*JAB_WRAPPER).upgrade()
//         {
//             // Try to convert the context_handle (JObject) to a proper handle
//             let context = event.context_handle as JObject;
//             let handle_map = wrapper.jobject_to_handle.lock().unwrap();
//             if let Some(handle) = handle_map.get(&context) {
//                 event.context_handle = *handle;
//             }
//
//             // Send the event
//             let event_tx = wrapper.event_tx.lock().unwrap();
//             if let Some(tx) = &*event_tx {
//                 let _ = tx.try_send(event);
//             }
//         }
//     }
// }

fn run_message_pump() {
    unsafe {
        use winapi::um::winuser::{DispatchMessageW, GetMessageW, TranslateMessage};

        let mut msg = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if result == 0 {
                break; // WM_QUIT
            } else if result == -1 {
                eprintln!("Message pump error");
                break;
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

// extern "C" fn focus_gained_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "FocusGained".to_string(),
//         vm_id,
//         context_handle: source as u64, // Will be converted to handle in send_callback_event
//         message: format!("source={}", source),
//         event_time: 0, // TODO: get actual time
//     });
// }
//
// extern "C" fn focus_lost_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "FocusLost".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn caret_update_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "CaretUpdate".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn mouse_clicked_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "MouseClicked".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn mouse_entered_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "MouseEntered".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn mouse_exited_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "MouseExited".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn mouse_pressed_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "MousePressed".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn mouse_released_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "MouseReleased".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_name_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     _old_name: *mut u16,
//     _new_name: *mut u16,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyNameChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_description_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     _old_desc: *mut u16,
//     _new_desc: *mut u16,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyDescriptionChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_state_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     _old_state: *mut u16,
//     _new_state: *mut u16,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyStateChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_value_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     _old_value: *mut u16,
//     _new_value: *mut u16,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyValueChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_selection_change_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertySelectionChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_text_change_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyTextChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_caret_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     old_pos: i32,
//     new_pos: i32,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyCaretChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!(
//             "source={}, old_pos={}, new_pos={}",
//             source, old_pos, new_pos
//         ),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_visible_data_change_cb(vm_id: i32, _event: i64, source: i64) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyVisibleDataChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_child_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     old_child: i64,
//     new_child: i64,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyChildChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!(
//             "source={}, old_child={}, new_child={}",
//             source, old_child, new_child
//         ),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_active_descendent_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     old_active: i64,
//     new_active: i64,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyActiveDescendentChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!(
//             "source={}, old_active={}, new_active={}",
//             source, old_active, new_active
//         ),
//         event_time: 0,
//     });
// }
//
// extern "C" fn property_table_model_change_cb(
//     vm_id: i32,
//     _event: i64,
//     source: i64,
//     _old_model: *mut u16,
//     _new_model: *mut u16,
// ) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "PropertyTableModelChange".to_string(),
//         vm_id,
//         context_handle: source as u64,
//         message: format!("source={}", source),
//         event_time: 0,
//     });
// }
//
// extern "C" fn java_shutdown_cb(vm_id: i32) {
//     send_callback_event(crate::JabCallbackEvent {
//         event_type: "JavaShutdown".to_string(),
//         vm_id,
//         context_handle: 0,
//         message: format!("vm_id={}", vm_id),
//         event_time: 0,
//     });
// }
