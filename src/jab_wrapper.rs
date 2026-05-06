use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type VmId = i32;
type JObject = i64;

// Global event sender - set by JabWrapper::set_event_sender()
static mut EVENT_TX: *mut mpsc::Sender<crate::JabCallbackEvent> = std::ptr::null_mut();

// Global JObject -> handle mapping for callbacks
static mut JOBJECT_TO_HANDLE: *mut HashMap<JObject, u64> = std::ptr::null_mut();

pub struct JabWrapper {
    pub initialized: AtomicBool,
    vm_id: Mutex<Option<VmId>>,
    root_context: Mutex<Option<JObject>>,
    elements: Mutex<HashMap<u64, (VmId, JObject)>>,
    next_handle: AtomicU64,
    pub context_tree: Mutex<Option<crate::context_tree::ContextTree>>,
    message_pump_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

unsafe impl Send for JabWrapper {}
unsafe impl Sync for JabWrapper {}

impl JabWrapper {
    pub fn new() -> Arc<Self> {
        let wrapper = Arc::new(Self {
            initialized: AtomicBool::new(false),
            vm_id: Mutex::new(None),
            root_context: Mutex::new(None),
            elements: Mutex::new(HashMap::new()),
            next_handle: AtomicU64::new(1),
            context_tree: Mutex::new(None),
            message_pump_handle: Mutex::new(None),
        });

        unsafe {
            let init_result = crate::bindings::initializeAccessBridge();
            if init_result == 0 {
                panic!("Failed to initialize JAB");
            }
        }

        wrapper.initialized.store(true, Ordering::SeqCst);
        wrapper.register_callbacks();

        // Initialize global JObject -> handle mapping
        unsafe {
            JOBJECT_TO_HANDLE = Box::into_raw(Box::new(HashMap::new())) as *mut _;
        }

        // Start Windows message pump in dedicated thread
        let wrapper_clone = wrapper.clone();
        let pump_handle = std::thread::spawn(move || {
            run_message_pump();
        });
        {
            let mut handle = wrapper.message_pump_handle.lock().unwrap();
            *handle = Some(pump_handle);
        }

        wrapper
    }

    fn register_callbacks(&self) {
        unsafe {
            crate::bindings::SetFocusGained(Some(focus_gained_cb));
            crate::bindings::SetFocusLost(Some(focus_lost_cb));
            crate::bindings::SetCaretUpdate(Some(caret_update_cb));
            crate::bindings::SetMouseClicked(Some(mouse_clicked_cb));
            crate::bindings::SetMouseEntered(Some(mouse_entered_cb));
            crate::bindings::SetMouseExited(Some(mouse_exited_cb));
            crate::bindings::SetMousePressed(Some(mouse_pressed_cb));
            crate::bindings::SetMouseReleased(Some(mouse_released_cb));
            crate::bindings::SetPropertyNameChange(Some(property_name_change_cb));
            crate::bindings::SetPropertyDescriptionChange(Some(property_description_change_cb));
            crate::bindings::SetPropertyStateChange(Some(property_state_change_cb));
            crate::bindings::SetPropertyValueChange(Some(property_value_change_cb));
            crate::bindings::SetPropertySelectionChange(Some(property_selection_change_cb));
            crate::bindings::SetPropertyTextChange(Some(property_text_change_cb));
            crate::bindings::SetPropertyCaretChange(Some(property_caret_change_cb));
            crate::bindings::SetPropertyVisibleDataChange(Some(property_visible_data_change_cb));
            crate::bindings::SetPropertyChildChange(Some(property_child_change_cb));
            crate::bindings::SetPropertyActiveDescendentChange(Some(property_active_descendent_change_cb));
            crate::bindings::SetPropertyTableModelChange(Some(property_table_model_change_cb));
            crate::bindings::SetJavaShutdown(Some(java_shutdown_cb));
        }
    }

    pub fn register_element(&self, vm_id: VmId, context: JObject) -> u64 {
        let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
        {
            let mut elements = self.elements.lock().unwrap();
            elements.insert(handle, (vm_id, context));
        }
        // Also update global JObject -> handle mapping
        unsafe {
            if !JOBJECT_TO_HANDLE.is_null() {
                (*JOBJECT_TO_HANDLE).insert(context, handle);
            }
        }
        handle
    }

    pub fn get_element(&self, handle: u64) -> Option<(VmId, JObject)> {
        let elements = self.elements.lock().unwrap();
        elements.get(&handle).copied()
    }

    pub fn release_element(&self, handle: u64) {
        let mut elements = self.elements.lock().unwrap();
        if let Some((vm_id, context)) = elements.remove(&handle) {
            unsafe {
                crate::bindings::ReleaseJavaObject(vm_id, context as i64);
                if !JOBJECT_TO_HANDLE.is_null() {
                    (*JOBJECT_TO_HANDLE).remove(&context);
                }
            }
        }
    }

    pub fn set_vm_id(&self, vm_id: VmId) {
        let mut id = self.vm_id.lock().unwrap();
        *id = Some(vm_id);
    }

    pub fn get_vm_id(&self) -> Option<VmId> {
        let id = self.vm_id.lock().unwrap();
        *id
    }

    pub fn set_root_context(&self, context: JObject) {
        let mut ctx = self.root_context.lock().unwrap();
        *ctx = Some(context);
    }

    pub fn get_root_context(&self) -> Option<JObject> {
        let ctx = self.root_context.lock().unwrap();
        *ctx
    }

    pub fn set_event_sender(&self, tx: mpsc::Sender<crate::JabCallbackEvent>) {
        unsafe {
            EVENT_TX = Box::into_raw(Box::new(tx)) as *mut _;
        }
    }

    pub fn list_java_windows(&self) -> Vec<crate::jab_service::WindowInfo> {
        let mut windows = Vec::new();
        unsafe {
            use winapi::um::winuser::{EnumWindows, IsWindow};
            use winapi::shared::windef::HWND;

            let mut hwnds: Vec<HWND> = Vec::new();

            extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> i32 {
                let hwnds = unsafe { &mut *(lparam as *mut Vec<HWND>) };
                if IsWindow(hwnd) != 0 {
                    hwnds.push(hwnd);
                }
                1
            }

            EnumWindows(Some(enum_proc), &mut hwnds as *mut _ as isize);

            for &hwnd in &hwnds {
                let mut vm_id: i32 = 0;
                let mut context: i64 = 0;
                let result = crate::bindings::GetAccessibleContextFromHWND(
                    hwnd as *mut _,
                    &mut vm_id,
                    &mut context,
                );
                if result != 0 && vm_id != 0 {
                    let mut info: crate::bindings::AccessibleContextInfo = std::mem::zeroed();
                    if crate::bindings::GetAccessibleContextInfo(vm_id, context, &mut info) != 0 {
                        windows.push(crate::jab_service::WindowInfo {
                            vm_id,
                            hwnd: hwnd as u64,
                            title: String::from_utf16_lossy(&info.name)
                                .trim_end_matches('\0')
                                .to_string(),
                            role: String::from_utf16_lossy(&info.role)
                                .trim_end_matches('\0')
                                .to_string(),
                        });
                    }
                }
            }
        }
        windows
    }

    pub fn select_window_by_title(&self, title: &str, _partial_match: bool) -> Result<(), String> {
        unsafe {
            use winapi::um::winuser::EnumWindows;
            use winapi::shared::windef::HWND;

            let title_owned = title.to_string();
            let mut found: Option<(i32, i64)> = None;

            extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> i32 {
                let (title, found) = unsafe {
                    &mut *(lparam as *mut (String, Option<(i32, i64)>))
                };
                let mut vm_id: i32 = 0;
                let mut context: i64 = 0;
                let result = crate::bindings::GetAccessibleContextFromHWND(
                    hwnd as *mut _,
                    &mut vm_id,
                    &mut context,
                );
                if result != 0 && vm_id != 0 {
                    let mut info: crate::bindings::AccessibleContextInfo = std::mem::zeroed();
                    if crate::bindings::GetAccessibleContextInfo(vm_id, context, &mut info) != 0 {
                        let window_title = String::from_utf16_lossy(&info.name);
                        let window_title = window_title.trim_end_matches('\0');
                        if window_title == *title {
                            *found = Some((vm_id, context));
                            return 0;
                        }
                    }
                }
                1
            }

            let mut data = (title_owned, None);
            EnumWindows(Some(enum_proc), &mut data as *mut _ as isize);

            if let Some((vm_id, context)) = data.1 {
                self.set_vm_id(vm_id);
                self.set_root_context(context);
                Ok(())
            } else {
                Err(format!("Window with title '{}' not found", title))
            }
        }
    }

    pub fn get_accessible_context_info(
        &self,
        vm_id: VmId,
        context: JObject,
    ) -> Option<crate::bindings::AccessibleContextInfo> {
        unsafe {
            let mut info: crate::bindings::AccessibleContextInfo = std::mem::zeroed();
            if crate::bindings::GetAccessibleContextInfo(vm_id, context, &mut info) != 0 {
                Some(info)
            } else {
                None
            }
        }
    }

    pub fn click_element(&self, handle: u64) -> Result<(), String> {
        let (vm_id, context) = self.get_element(handle).ok_or("Element not found")?;

        unsafe {
            let mut actions: crate::bindings::AccessibleActions = std::mem::zeroed();
            if crate::bindings::getAccessibleActions(vm_id, context, &mut actions) == 0 {
                return Err("Failed to get accessible actions".to_string());
            }

            for i in 0..actions.actionsCount {
                let action_name = String::from_utf16_lossy(&actions.actionInfo[i as usize].name);
                if action_name.to_lowercase().contains("click") {
                    let mut actions_to_do: crate::bindings::AccessibleActionsToDo = std::mem::zeroed();
                    actions_to_do.actionsCount = 1;
                    actions_to_do.actions[0] = actions.actionInfo[i as usize];

                    let mut failure: i32 = 0;
                    if crate::bindings::doAccessibleActions(vm_id, context, &mut actions_to_do, &mut failure) != 0 {
                        return Ok(());
                    } else {
                        return Err(format!("Failed to perform click action, failure index: {}", failure));
                    }
                }
            }
        }

        Err("No click action found for element".to_string())
    }

    pub fn type_text(&self, handle: u64, text: &str) -> Result<(), String> {
        let (vm_id, context) = self.get_element(handle).ok_or("Element not found")?;

        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            if crate::bindings::setTextContents(vm_id, context as i64, text_wide.as_ptr()) != 0 {
                Ok(())
            } else {
                Err("Failed to set text contents".to_string())
            }
        }
    }

    pub fn get_version_info(&self) -> Result<crate::bindings::AccessBridgeVersionInfo, String> {
        let vm_id = self.get_vm_id().ok_or("No VM ID set")?;
        unsafe {
            let mut info: crate::bindings::AccessBridgeVersionInfo = std::mem::zeroed();
            if crate::bindings::GetVersionInfo(vm_id, &mut info) != 0 {
                Ok(info)
            } else {
                Err("Failed to get version info".to_string())
            }
        }
    }
}

// Standalone function to send callback events via the global sender
fn send_callback_event(mut event: crate::JabCallbackEvent) {
    unsafe {
        if !EVENT_TX.is_null() {
            let tx = &*EVENT_TX;
            // Try to convert the context_handle (JObject) to a proper handle
            let context = event.context_handle as JObject;
            if !JOBJECT_TO_HANDLE.is_null() {
                if let Some(handle) = (*JOBJECT_TO_HANDLE).get(&context) {
                    event.context_handle = *handle;
                }
            }
            let _ = tx.try_send(event);
        }
    }
}

fn run_message_pump() {
    unsafe {
        use winapi::um::winuser::{GetMessageW, TranslateMessage, DispatchMessageW};
        use winapi::shared::windef::HWND;

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

extern "C" fn focus_gained_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "FocusGained".to_string(),
        vm_id,
        context_handle: source as u64, // Will be converted to handle in send_callback_event
        message: format!("source={}", source),
        event_time: 0, // TODO: get actual time
    });
}

extern "C" fn focus_lost_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "FocusLost".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn caret_update_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "CaretUpdate".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn mouse_clicked_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "MouseClicked".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn mouse_entered_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "MouseEntered".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn mouse_exited_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "MouseExited".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn mouse_pressed_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "MousePressed".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn mouse_released_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "MouseReleased".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_name_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    _old_name: *mut u16,
    _new_name: *mut u16,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyNameChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_description_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    _old_desc: *mut u16,
    _new_desc: *mut u16,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyDescriptionChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_state_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    _old_state: *mut u16,
    _new_state: *mut u16,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyStateChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_value_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    _old_value: *mut u16,
    _new_value: *mut u16,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyValueChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_selection_change_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertySelectionChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_text_change_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyTextChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_caret_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    old_pos: i32,
    new_pos: i32,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyCaretChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}, old_pos={}, new_pos={}", source, old_pos, new_pos),
        event_time: 0,
    });
}

extern "C" fn property_visible_data_change_cb(vm_id: i32, event: i64, source: i64) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyVisibleDataChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn property_child_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    old_child: i64,
    new_child: i64,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyChildChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}, old_child={}, new_child={}", source, old_child, new_child),
        event_time: 0,
    });
}

extern "C" fn property_active_descendent_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    old_active: i64,
    new_active: i64,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyActiveDescendentChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}, old_active={}, new_active={}", source, old_active, new_active),
        event_time: 0,
    });
}

extern "C" fn property_table_model_change_cb(
    vm_id: i32,
    event: i64,
    source: i64,
    _old_model: *mut u16,
    _new_model: *mut u16,
) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "PropertyTableModelChange".to_string(),
        vm_id,
        context_handle: source as u64,
        message: format!("source={}", source),
        event_time: 0,
    });
}

extern "C" fn java_shutdown_cb(vm_id: i32) {
    send_callback_event(crate::JabCallbackEvent {
        event_type: "JavaShutdown".to_string(),
        vm_id,
        context_handle: 0,
        message: format!("vm_id={}", vm_id),
        event_time: 0,
    });
}
