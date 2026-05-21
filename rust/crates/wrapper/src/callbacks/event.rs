use crate::types::{JObject, VmId};

#[derive(Debug, Clone)]
pub(crate) enum CallbackChangeEvent {
    Name {
        vm_id: VmId,
        source_jobject: JObject,
        _old_name: String,
        new_name: String,
    },
    Description {
        vm_id: VmId,
        source_jobject: JObject,
        _old_description: String,
        new_description: String,
    },
    State {
        vm_id: VmId,
        source_jobject: JObject,
        _old_state: String,
        new_state: String,
    },
    Text {
        vm_id: VmId,
        source_jobject: JObject,
    },
}

impl CallbackChangeEvent {
    pub(crate) fn get_vm_id(&self) -> VmId {
        match *self {
            Self::Name { vm_id, .. } => vm_id,
            Self::Description { vm_id, .. } => vm_id,
            Self::State { vm_id, .. } => vm_id,
            Self::Text { vm_id, .. } => vm_id,
        }
    }

    pub(crate) fn get_source_jobject(&self) -> JObject {
        match *self {
            Self::Name { source_jobject, .. } => source_jobject,
            Self::Description { source_jobject, .. } => source_jobject,
            Self::State { source_jobject, .. } => source_jobject,
            Self::Text { source_jobject, .. } => source_jobject,
        }
    }
}
