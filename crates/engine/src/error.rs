use std::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum EngineError {
    InitialisationError,
}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::InitialisationError => "Engine creation error",
        };
        write!(f, "{msg}")
    }
}

impl Context for EngineError {}