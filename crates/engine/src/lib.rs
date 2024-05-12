use std::cell::RefCell;
use std::mem::transmute;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use error_stack::{Report, ResultExt};
pub use error_stack::Result;

use core::{Size, ThrustlerBackend, ThrustlerWindow, WindowEvent};
pub use core::error::ThrustlerError;
pub use core::game_objects::{GameObject, Scene, Vertex};
use vulkan::VulkanBackend;
use vulkan::vulkano_tools::VulkanWindow;
use winit_window::{OutputWindow, WinitWindow};

mod error;

pub struct Engine {
    frames_per_second: u32,
    window: Box<dyn ThrustlerWindow>,
    backend: Rc<RefCell<dyn ThrustlerBackend>>,
    scenes: Vec<Box<dyn Scene>>,
}

impl Engine {
    pub fn new_with_settings(engine_settings: EngineSettings) -> Result<Engine, ThrustlerError> {
        let size = engine_settings.window_size;

        let (backend, initializer) = match engine_settings.backend {
            Backend::Vulkan => {
                let backend = Rc::new(RefCell::new(VulkanBackend::new(size)));
                let rc_backend = backend.clone();
                let initializer = Box::new(move |window| {
                    let vulkan_window = unsafe {
                        transmute::<Arc<dyn OutputWindow>, Arc<dyn VulkanWindow>>(window)
                    };
                    rc_backend.borrow_mut().init(vulkan_window)
                });
                (backend, Some(initializer))
            }
        };

        let window = match engine_settings.window {
            Window::Winit => {
                WinitWindow::new(
                    size,
                    initializer
                        .ok_or(Report::new(ThrustlerError::EngineError))
                        .attach_printable("Vulkan init callback is not specified")?,
                )
            }
        }
            .change_context(ThrustlerError::EngineError)
            .attach_printable("Window creation error")?;

        Ok(Self {
            frames_per_second: engine_settings.frames_per_second,
            window: Box::new(window),
            backend,
            scenes: vec![],
        })
    }

    pub fn start(mut self) -> Result<(), ThrustlerError> {
        let mut previous = Instant::now();
        let frame_time = 1.0 / (self.frames_per_second as f32);
        //the time elapsed since last handled frame
        let mut elapsed_time = 0.0;

        let back_clone = self.backend.clone();
        self.window.start(Box::new(move |event| {
            for scene in &mut self.scenes {
                match event {
                    WindowEvent::OnStart => scene.on_start(),
                    WindowEvent::OnDraw => {
                        elapsed_time += previous.elapsed().as_secs_f32();
                        previous = Instant::now();

                        while elapsed_time >= frame_time {
                            scene.on_update();
                            back_clone.clone().borrow_mut().draw_scene(scene);
                            //we could still have some time which wasn't taken into account, and we have to use it in future calculations
                            elapsed_time -= frame_time;
                        }
                    }
                    WindowEvent::OnStop => scene.on_destroy(),
                }
            }
        }))
    }

    pub fn add_scene(mut self, scene: impl Scene + 'static) -> Engine {
        self.scenes.push(Box::new(scene));
        self
    }
}

pub struct EngineSettings {
    pub window_size: Size,
    pub frames_per_second: u32,
    pub window: Window,
    pub backend: Backend,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings {
            window_size: Size::default(),
            frames_per_second: 60,
            window: Window::Winit,
            backend: Backend::Vulkan,
        }
    }
}

enum Window {
    Winit,
}

enum Backend {
    Vulkan,
}

