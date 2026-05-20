use crate::types::{VmId, JObject};

#[derive(Debug, Clone)]
pub enum CallbackChangeEvent {
    Name {
        vm_id: VmId,
        source_jobject: JObject,
        #[allow(dead_code)]
        old_name: String,
        new_name: String,
    },
    Description {
        vm_id: VmId,
        source_jobject: JObject,
        #[allow(dead_code)]
        old_description: String,
        new_description: String,
    },
    State {
        vm_id: VmId,
        source_jobject: JObject,
        #[allow(dead_code)]
        old_state: String,
        new_state: String,
    },
    Text {
        vm_id: VmId,
        source_jobject: JObject,
    },
}
