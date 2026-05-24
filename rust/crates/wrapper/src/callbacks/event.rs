use crate::types::{JObject, VmId};

#[derive(Debug)]
pub(crate) struct EventMeta {
    pub(super) vm_id: VmId,
    pub(super) event: JObject,
    pub(super) source: JObject,
}

impl Drop for EventMeta {
    fn drop(&mut self) {
        unsafe {
            if self.event != 0 {
                jab_sys::ReleaseJavaObject(self.vm_id, self.event);
            }
            if self.source != 0 {
                jab_sys::ReleaseJavaObject(self.vm_id, self.source);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct EventData<T> {
    pub(super) old: T,
    pub(super) new: T,
}

#[derive(Debug)]
pub(crate) enum ChangeEvent {
    Name(EventMeta, EventData<String>),
    Description(EventMeta, EventData<String>),
    #[allow(dead_code)]
    State(EventMeta, EventData<String>),
    Text(EventMeta),
    #[allow(dead_code)]
    // Value(EventMeta, EventData<String>),
    // VisibleData(EventMeta),
    // ActiveDescendent(EventMeta, EventData<JObject>),
    Child(EventMeta, EventData<JObject>),
}

impl ChangeEvent {
    pub(crate) fn meta(&self) -> &EventMeta {
        match self {
            Self::Name(meta, ..)
            | Self::Description(meta, ..)
            | Self::State(meta, ..)
            | Self::Text(meta, ..)
            // | Self::Value(meta, ..)
            // | Self::VisibleData(meta, ..)
            // | Self::ActiveDescendent(meta, ..)
            | Self::Child(meta, ..) => meta,
        }
    }
}

impl Drop for ChangeEvent {
    fn drop(&mut self) {
        #[allow(clippy::single_match)]
        match self {
            // Self::ActiveDescendent(meta, data) |
            Self::Child(meta, data) => unsafe {
                if data.old != 0 {
                    jab_sys::ReleaseJavaObject(meta.vm_id, data.old);
                }
                if data.new != 0 {
                    jab_sys::ReleaseJavaObject(meta.vm_id, data.new);
                }
            },
            _ => (),
        }
    }
}
