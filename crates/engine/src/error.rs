use std::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum EngineError {
    CreationError,
}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::CreationError => "Engine creation error",
        };
        write!(f, "{msg}")
    }
}

impl Context for EngineError {}