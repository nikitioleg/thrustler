use engine::{Engine, EngineSettings};
use engine::Result;
use engine::ThrustlerError;

fn main() -> Result<(), ThrustlerError> {
    Ok(
        Engine::new_with_settings(
            EngineSettings::default()
        )?
            .start()?,
    )
}