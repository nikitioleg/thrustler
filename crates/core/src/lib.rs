use error_stack::Result;
use crate::error::ThrustlerError;

pub mod error;

pub trait ThrustlerWindow {
    fn start(&self, dispatcher: Box<dyn FnMut(WindowEvent) -> ()>) -> Result<(), ThrustlerError>;
}

pub enum WindowEvent {
    OnStart,
    OnDraw,
    OnStop,
}

pub trait ThrustlerBackend {
    fn test_draw(&mut self);
}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size { width, height }
    }
}

impl Into<[u32; 2]> for Size {
    fn into(self) -> [u32; 2] {
        [self.width, self.height]
    }
}

impl Into<[f32; 2]> for Size {
    fn into(self) -> [f32; 2] {
        [self.width as f32, self.height as f32]
    }
}
