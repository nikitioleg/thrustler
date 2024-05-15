#[derive(Debug)]
pub struct GameObject {
    pub vertices: Vec<Vertex>,
}

impl GameObject {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertices
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub fn x(&self) -> f32 {
        self.position[0]
    }

    pub fn y(&self) -> f32 {
        self.position[1]
    }
}

impl Vertex {
    pub fn new(position: [f32; 2]) -> Self {
        Self { position }
    }
}

pub trait Scene {
    fn on_start(&mut self);
    fn on_update(&mut self);
    fn on_destroy(&mut self);
    fn get_scene_objects(&self) -> &Vec<GameObject>;
}