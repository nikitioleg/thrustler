use engine::{Backend, Engine, EngineSettings, Scene};
use engine::{GameObject, Vertex};
use engine::Result;
use engine::ThrustlerError;

fn main() -> Result<(), ThrustlerError> {
    Ok(
        Engine::new_with_settings(
            EngineSettings {
                frames_per_second: 1,
                backend: Backend::Wgpu,
                ..EngineSettings::default()
            }
        )?
            .add_scene(SierpinskiTriangles::new(6))
            .start()?,
    )
}

struct SierpinskiTriangles {
    game_objects: Vec<GameObject>,
    depth: i32,
    current_depth: i32,
}

impl SierpinskiTriangles {
    fn new(depth: i32) -> Self {
        Self {
            game_objects: vec![],
            depth,
            current_depth: 0,
        }
    }

    fn split_triangle(game_object: &GameObject) -> [GameObject; 3] {
        let [center_x,center_y] = Self::get_center(game_object);

        let intrinsic_triangle_vertices = [
            Vertex::new([
                (game_object.vertices[0].x() + center_x) / 2f32,
                center_y,
            ]),
            Vertex::new([
                center_x,
                game_object.vertices[0].y(),
            ]),
            Vertex::new([
                (game_object.vertices[2].x() + center_x) / 2f32,
                center_y,
            ])
        ];
        [
            GameObject::new(vec![
                game_object.vertices[0], intrinsic_triangle_vertices[0], intrinsic_triangle_vertices[1],
            ]),
            GameObject::new(vec![
                intrinsic_triangle_vertices[0],  game_object.vertices[1], intrinsic_triangle_vertices[2],
            ]),
            GameObject::new(vec![
                intrinsic_triangle_vertices[1], intrinsic_triangle_vertices[2], game_object.vertices[2],
            ]),
        ]
    }

    fn get_center(game_object: &GameObject) -> [f32; 2] {
        let top_left_x = game_object.vertices[0].position[0];
        let top_left_y = game_object.vertices[1].position[1];

        let right_bottom_x = game_object.vertices[2].position[0];
        let right_bottom_y = game_object.vertices[0].position[1];

        [(top_left_x + right_bottom_x) / 2f32, (top_left_y + right_bottom_y) / 2f32]
    }
}

impl Scene for SierpinskiTriangles {
    fn on_start(&mut self) {
        println!("SierpinskiTriangles start")
    }

    fn on_update(&mut self) {
        if self.current_depth >= self.depth {
            return;
        }

        let new_triangles = if self.game_objects.is_empty() {
            vec![
                GameObject::new(vec![
                    Vertex::new([-1.0, 1.0]),
                    Vertex::new([0.0, -1.0]),
                    Vertex::new([1.0, 1.0]),
                ])
            ]
        } else {
            self.game_objects.iter()
                .map(|game_object| Self::split_triangle(game_object))
                .flatten()
                .collect::<Vec<_>>()
        };

        self.current_depth += 1;
        self.game_objects.clear();

        new_triangles.into_iter().for_each(|game_object| {
            self.game_objects.push(game_object)
        });
    }

    fn on_destroy(&mut self) {}

    fn get_scene_objects(&self) -> &Vec<GameObject> {
        self.game_objects.as_ref()
    }
}
