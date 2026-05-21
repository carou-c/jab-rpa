use crate::types::{JObject, VmId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct EventMeta {
    pub(crate) vm_id: VmId,
    pub(crate) source: JObject,
}

#[derive(Debug, Clone)]
pub(crate) struct EventData<T> {
    pub(crate) old: T,
    pub(crate) new: T,
}

#[derive(Debug, Clone)]
pub(crate) enum ChangeEvent {
    Name(EventMeta, EventData<String>),
    Description(EventMeta, EventData<String>),
    State(EventMeta, EventData<String>),
    Text(EventMeta),
    Value(EventMeta, EventData<String>),
    VisibleData(EventMeta),
    Child(EventMeta, EventData<JObject>),
    ActiveDescendent(EventMeta, EventData<JObject>),
}

impl ChangeEvent {
    pub(crate) fn get_meta(&self) -> EventMeta {
        match *self {
            Self::Name(meta, ..)
            | Self::Description(meta, ..)
            | Self::State(meta, ..)
            | Self::Text(meta, ..)
            | Self::Value(meta, ..)
            | Self::VisibleData(meta, ..)
            | Self::Child(meta, ..)
            | Self::ActiveDescendent(meta, ..) => meta,
        }
    }
}
