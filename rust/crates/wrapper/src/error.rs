use thiserror::Error;

#[derive(Debug)]
pub struct ContextSuffix(pub(crate) Option<String>);

impl std::fmt::Display for ContextSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ctx) = &self.0 {
            write!(f, " ({ctx})")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
pub enum JabWrapperError {
    #[error("Error during JabRuntime initialization")]
    RuntimeInitError(#[from] JabRuntimeInitError),

    #[error("JAB call {jab_call} failed{context}")]
    JabCallFailed {
        jab_call: &'static str,
        context: ContextSuffix,
    },

    #[error("No {action:?} action found. Available actions: {available_actions:?}")]
    ActionNotFound {
        action: String,
        available_actions: Vec<String>,
    }
}

#[derive(Debug, Error)]
pub enum ContextTreeError {
    #[error("Root node missing")]
    RootNodeMissing
}


#[derive(Debug, Error)]
pub enum JabRuntimeInitError {
    #[error("Failed to initialize Java Access Bridge")]
    InitializeAccessBridgeFailed,

    #[error("Message pump thread crashed during initialization: {failed_call} failed")]
    MessagePumpThreadCrashed { failed_call: &'static str },
}
