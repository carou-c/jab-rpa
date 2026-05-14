use crate::context_tree::{ContextNode, ContextTree};

#[derive(Debug)]
pub struct Locator {
    pub selector: String,
}

impl ContextTree {
    pub fn get_nodes(&self, locator: &Locator, relative_to: Option<&ContextNode>) -> Vec<&ContextNode> {
        unimplemented!()
    }
}
