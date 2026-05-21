// For safety, the trampolines must:
// 1. Convert the wchar_t* strings to Rust String
// 2. Send the event over the channel
// 3. NOT panic (can't unwind through C)
// 4. Be 'static (no captured references)

use std::{slice, sync::Mutex};

use crossbeam::channel;
use jab_sys::{MAX_STRING_SIZE, wchar_t};

use super::event::CallbackChangeEvent;
use crate::{
    types::{JObject, VmId},
    utils::utf16_to_string,
};

static CALLBACK_TX: Mutex<Option<channel::Sender<CallbackChangeEvent>>> = Mutex::new(None);

extern "C" fn on_property_name_change(
    vm_id: VmId,
    _event: JObject,
    source: JObject,
    old_name: *mut wchar_t,
    new_name: *mut wchar_t,
) {
    let Ok(tx) = CALLBACK_TX.lock() else {
        return;
    };
    let Some(tx) = tx.as_ref() else {
        return;
    };
    let old_name = unsafe { slice::from_raw_parts(old_name, MAX_STRING_SIZE as usize) };
    let new_name = unsafe { slice::from_raw_parts(new_name, MAX_STRING_SIZE as usize) };

    let _old_name = utf16_to_string(old_name);
    let new_name = utf16_to_string(new_name);
    let _ = tx.send(CallbackChangeEvent::Name {
        vm_id,
        source_jobject: source,
        _old_name,
        new_name,
    });
}

extern "C" fn on_property_description_change(
    vm_id: VmId,
    _event: JObject,
    source: JObject,
    old_description: *mut wchar_t,
    new_description: *mut wchar_t,
) {
    let Ok(tx) = CALLBACK_TX.lock() else {
        return;
    };
    let Some(tx) = tx.as_ref() else {
        return;
    };
    let old_description =
        unsafe { slice::from_raw_parts(old_description, MAX_STRING_SIZE as usize) };
    let new_description =
        unsafe { slice::from_raw_parts(new_description, MAX_STRING_SIZE as usize) };

    let _old_description = utf16_to_string(old_description);
    let new_description = utf16_to_string(new_description);
    let _ = tx.send(CallbackChangeEvent::Description {
        vm_id,
        source_jobject: source,
        _old_description,
        new_description,
    });
}

extern "C" fn on_property_state_change(
    vm_id: VmId,
    _event: JObject,
    source: JObject,
    old_state: *mut wchar_t,
    new_state: *mut wchar_t,
) {
    let Ok(tx) = CALLBACK_TX.lock() else {
        return;
    };
    let Some(tx) = tx.as_ref() else {
        return;
    };
    let old_state = unsafe { slice::from_raw_parts(old_state, MAX_STRING_SIZE as usize) };
    let new_state = unsafe { slice::from_raw_parts(new_state, MAX_STRING_SIZE as usize) };

    let _old_state = utf16_to_string(old_state);
    let new_state = utf16_to_string(new_state);
    let _ = tx.send(CallbackChangeEvent::State {
        vm_id,
        source_jobject: source,
        _old_state,
        new_state,
    });
}

extern "C" fn on_property_text_change(vm_id: VmId, _event: JObject, source: JObject) {
    let Ok(tx) = CALLBACK_TX.lock() else {
        return;
    };
    let Some(tx) = tx.as_ref() else {
        return;
    };

    let _ = tx.send(CallbackChangeEvent::Text {
        vm_id,
        source_jobject: source,
    });
}

pub(crate) unsafe fn subscribe_events() -> channel::Receiver<CallbackChangeEvent> {
    let (tx, rx) = channel::unbounded();

    let Ok(mut cb_tx) = CALLBACK_TX.lock() else {
        return rx;
    };
    *cb_tx = Some(tx);

    unsafe {
        jab_sys::SetPropertyNameChange(Some(on_property_name_change));
        jab_sys::SetPropertyDescriptionChange(Some(on_property_description_change));
        jab_sys::SetPropertyStateChange(Some(on_property_state_change));
        jab_sys::SetPropertyTextChange(Some(on_property_text_change));
        // jab_sys::SetPropertyChildChange(Some(on_property_child_change));
        // jab_sys::SetPropertyVisibleDataChange(Some(on_property_visible_data_change));
        // jab_sys::SetPropertyActiveDescendentChange(Some(on_property_active_descendent_change));
        // jab_sys::SetFocusGained(Some(on_focus_gained));
        // jab_sys::SetFocusLost(Some(on_focus_lost));
    }
    rx
}

pub(crate) fn shutdown_event_channel() {
    if let Ok(mut tx) = CALLBACK_TX.lock() {
        (*tx).take();
    };
}
