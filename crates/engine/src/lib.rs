use std::cell::RefCell;
use std::mem::transmute;
use std::rc::Rc;
use std::sync::Arc;

use error_stack::{Result, ResultExt};

use core::{Size, ThrustlerBackend, ThrustlerWindow, WindowEvent};
use core::error::ThrustlerError;
use error::EngineError;
use vulkan::VulkanBackend;
use vulkan::vulkano_tools::VulkanWindow;
use winit_window::{OutputWindow, WinitWindow};

mod error;

pub struct Engine {
    window: Box<dyn ThrustlerWindow>,
    backend: Rc<RefCell<dyn ThrustlerBackend<Window=Arc<dyn VulkanWindow>>>>,
}

impl Engine {
    pub fn new() -> Result<Engine, EngineError> {
        let size = Size::new(800, 600);
        let mut vulkan = Rc::new(RefCell::new(VulkanBackend::new(size)));
        let mut closure_backend = vulkan.clone();

        let window = WinitWindow::new(
            size,
            Box::new(move |window| {
                let vulkan_window = unsafe { transmute::<Arc<dyn OutputWindow>, Arc<dyn VulkanWindow>>(window) };
                closure_backend.borrow_mut().init(vulkan_window.clone())
            }),
        )
            .change_context(EngineError::CreationError)?;

        Ok(Self {
            window: Box::new(window),
            backend: vulkan,
        })
    }

    pub fn start(mut self) -> Result<(), ThrustlerError> {
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

fn on_start(_backend: Rc<RefCell<dyn ThrustlerBackend<Window=Arc<dyn VulkanWindow>>>>) {}

fn on_draw(backend: Rc<RefCell<dyn ThrustlerBackend<Window=Arc<dyn VulkanWindow>>>>) {
    backend.borrow_mut().test_draw();
}

fn on_stop(_backend: Rc<RefCell<dyn ThrustlerBackend<Window=Arc<dyn VulkanWindow>>>>) {}