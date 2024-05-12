use engine::{Engine, EngineSettings, Scene};
use engine::{GameObject, Vertex};
use engine::Result;
use engine::ThrustlerError;

fn main() -> Result<(), ThrustlerError> {
    Ok(
        Engine::new_with_settings(
            EngineSettings{
                frames_per_second: 5,
                ..EngineSettings::default()
            }
        )?
            .add_scene(SierpinskiTriangles::new())
            .start()?,
    )
}

struct SierpinskiTriangles {
    game_objects: Vec<GameObject>,
}

impl SierpinskiTriangles {
    fn new() -> Self {
        Self {
            game_objects: vec![
                GameObject::new(vec![Vertex::new([-1.0, 1.0]), Vertex::new([0.0, -1.0]), Vertex::new([1.0, 1.0])]),
            ]
        }
    }
}

impl Scene for SierpinskiTriangles {
    fn on_start(&mut self) {
        println!("SierpinskiTriangles start")
    }

    fn on_update(&mut self) {
        println!("SierpinskiTriangles on_update")
    }

    fn on_destroy(&mut self) {
        println!("SierpinskiTriangles on destroy")
    }

    fn get_scene_objects(&self) -> &Vec<GameObject> {
        self.game_objects.as_ref()
    }
}
