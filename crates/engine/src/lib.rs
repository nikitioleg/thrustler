use std::cell::RefCell;
use std::rc::Rc;

use error_stack::{Result, ResultExt};

use core::{Size, ThrustlerBackend, ThrustlerWindow, WindowEvent};
use core::errors::ThrustlerWindowError;
use error::EngineError;
use vulkan::Vulkan;
use winit_window::WinitWindow;

mod error;

pub struct Engine {
    window: Box<dyn ThrustlerWindow>,
    backend: Rc<RefCell<dyn ThrustlerBackend>>,
}

impl Engine {
    pub fn new() -> Result<Engine, EngineError> {
        let mut vulkan = Rc::new(RefCell::new(Vulkan::new()));
        let mut closure_backend = vulkan.clone();

        let window = WinitWindow::new(
            Size::new(800, 600),
            Box::new(move |window| {
                println!("Window Created");
                closure_backend.borrow_mut().init();
            }),
        )
            .change_context(EngineError::CreationError)?;

        println!("Create engine");

        Ok(Self {
            window: Box::new(window),
            backend: vulkan,
        })
    }

    pub fn start(mut self) -> Result<(), ThrustlerWindowError> {
        let window = self.window;
        let backend = self.backend.clone();

        window.start(Box::new(move |event| {
            let backend = backend.clone();
            match event {
                WindowEvent::OnStart => on_start(backend),
                WindowEvent::OnDraw => on_draw(backend),
                WindowEvent::OnStop => on_stop(backend),
            }
        }))
    }
}

fn on_start(backend: Rc<RefCell<dyn ThrustlerBackend>>) {
    println!("on_start")
}

fn on_draw(backend: Rc<RefCell<dyn ThrustlerBackend>>) {
    println!("on_draw")
}

fn on_stop(backend: Rc<RefCell<dyn ThrustlerBackend>>) {
    println!("on_stop")
}