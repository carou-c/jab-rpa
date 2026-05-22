// For safety, the trampolines must:
// 1. Convert the wchar_t* strings to Rust String
// 2. Send the event over the channel
// 3. NOT panic (can't unwind through C)
// 4. Be 'static (no captured references)

use std::{slice, sync::Mutex};

use crossbeam::channel;
use jab_sys::{MAX_STRING_SIZE, wchar_t};

use super::event::{ChangeEvent, EventData, EventMeta};
use crate::{
    types::{JObject, VmId},
    utils::utf16_to_string,
};

static CALLBACK_TX: Mutex<Option<channel::Sender<ChangeEvent>>> = Mutex::new(None);

macro_rules! cb {
    (String, $name:ident, $variant:path) => {
        extern "C" fn $name(
            vm_id: VmId,
            event: JObject,
            source: JObject,
            old: *mut wchar_t,
            new: *mut wchar_t,
        ) {
            let Ok(tx) = CALLBACK_TX.lock() else {
                return;
            };
            let Some(tx) = tx.as_ref() else {
                return;
            };

            let old =
                utf16_to_string(unsafe { slice::from_raw_parts(old, MAX_STRING_SIZE as usize) });
            let new =
                utf16_to_string(unsafe { slice::from_raw_parts(new, MAX_STRING_SIZE as usize) });
            let meta = EventMeta {
                vm_id,
                event,
                source,
            };
            let data = EventData { old, new };

            let _ = tx.send($variant(meta, data));
        }
    };
    (JObject, $name:ident, $variant:path) => {
        extern "C" fn $name(
            vm_id: VmId,
            event: JObject,
            source: JObject,
            old: JObject,
            new: JObject,
        ) {
            let Ok(tx) = CALLBACK_TX.lock() else {
                return;
            };
            let Some(tx) = tx.as_ref() else {
                return;
            };

            let meta = EventMeta {
                vm_id,
                event,
                source,
            };
            let data = EventData { old, new };

            let _ = tx.send($variant(meta, data));
        }
    };
    ($name:ident, $variant:path) => {
        extern "C" fn $name(vm_id: VmId, event: JObject, source: JObject) {
            let Ok(tx) = CALLBACK_TX.lock() else {
                return;
            };
            let Some(tx) = tx.as_ref() else {
                return;
            };
            let meta = EventMeta {
                vm_id,
                event,
                source,
            };

            let _ = tx.send($variant(meta));
        }
    };
}

cb!(String, on_name_change, ChangeEvent::Name);
cb!(String, on_description_change, ChangeEvent::Description);
cb!(String, on_state_change, ChangeEvent::State);
cb!(on_text_change, ChangeEvent::Text);
cb!(String, on_value_change, ChangeEvent::Value);
cb!(on_visible_data_change, ChangeEvent::VisibleData);
cb!(JObject, on_child_change, ChangeEvent::Child);
cb!(
    JObject,
    on_active_descendent_change,
    ChangeEvent::ActiveDescendent
);

pub(crate) unsafe fn subscribe_events() -> channel::Receiver<ChangeEvent> {
    let (tx, rx) = channel::unbounded();

    let Ok(mut cb_tx) = CALLBACK_TX.lock() else {
        return rx;
    };
    *cb_tx = Some(tx);

    unsafe {
        jab_sys::SetPropertyNameChange(Some(on_name_change));
        jab_sys::SetPropertyDescriptionChange(Some(on_description_change));
        jab_sys::SetPropertyStateChange(Some(on_state_change));
        jab_sys::SetPropertyTextChange(Some(on_text_change));
        jab_sys::SetPropertyValueChange(Some(on_value_change));
        jab_sys::SetPropertyVisibleDataChange(Some(on_visible_data_change));
        jab_sys::SetPropertyChildChange(Some(on_child_change));
        jab_sys::SetPropertyActiveDescendentChange(Some(on_active_descendent_change));
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
