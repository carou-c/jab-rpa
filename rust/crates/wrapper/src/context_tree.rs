mod matcher;
mod node;
mod tree;

pub use self::{node::ContextNode, tree::ContextTree};

pub(crate) type NodeHandle = u64;
pub(crate) const ROOT_HANDLE: NodeHandle = 0;
