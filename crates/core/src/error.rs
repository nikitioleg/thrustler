use std::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub enum ThrustlerError {
    WindowError,
    GraphicalBackendError,
}

impl Display for ThrustlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::WindowError => "Unable error",
            Self::GraphicalBackendError => "Graphical backend error"
        };
        write!(f, "{msg}")
    }
}

impl Context for ThrustlerError {}