use std::fmt::{Display, Formatter};
use error_stack::Context;

#[derive(Debug)]
pub enum ThrustlerWindowError {
    UnableToCreateWindow,
    WindowLoopError,
}

impl Display for ThrustlerWindowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::UnableToCreateWindow => "Unable to create window",
            Self::WindowLoopError => "Window loop error"
        };
        write!(f, "{msg}")
    }
}

impl Context for ThrustlerWindowError {}