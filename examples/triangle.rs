use engine::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(Engine::new()?.start()?)
}