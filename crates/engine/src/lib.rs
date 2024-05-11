use std::cell::RefCell;
use std::mem::transmute;
use std::rc::Rc;
use std::sync::Arc;

use error_stack::{Report, ResultExt};
pub use error_stack::Result;

use core::{Size, ThrustlerBackend, ThrustlerWindow, WindowEvent};
pub use core::error::ThrustlerError;
pub use core::game_objects::{Vertex, GameObject, Scene};
use vulkan::VulkanBackend;
use vulkan::vulkano_tools::VulkanWindow;
use winit_window::{OutputWindow, WinitWindow};

mod error;

pub struct Engine {
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
            window: Box::new(window),
            backend,
            scenes: vec![],
        })
    }

    pub fn start(mut self) -> Result<(), ThrustlerError> {
        let back_clone = self.backend.clone();
        self.window.start(Box::new(move |event| {
            for scene in &mut self.scenes {
                match event {
                    WindowEvent::OnStart => scene.on_start(),
                    WindowEvent::OnDraw => {
                        scene.on_update();
                        back_clone.clone().borrow_mut().draw_scene(scene)
                    },
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
    window_size: Size,
    window: Window,
    backend: Backend,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings::builder()
            .size(Size::default())
            .window(Window::Winit)
            .backend(Backend::Vulkan)
            .build()
    }
}

impl EngineSettings {
    fn builder() -> EngineSettingsBuilder {
        EngineSettingsBuilder::new()
    }
}

struct EngineSettingsBuilder {
    window_size: Option<Size>,
    window: Option<Window>,
    backend: Option<Backend>,
}

impl EngineSettingsBuilder {
    fn new() -> Self {
        Self {
            window_size: None,
            window: None,
            backend: None,
        }
    }
    pub fn size(mut self, size: Size) -> EngineSettingsBuilder {
        self.window_size = Some(size);
        self
    }
    pub fn window(mut self, window: Window) -> EngineSettingsBuilder {
        self.window = Some(window);
        self
    }
    pub fn backend(mut self, backend: Backend) -> EngineSettingsBuilder {
        self.backend = Some(backend);
        self
    }

    pub fn build(self) -> EngineSettings {
        let window_size = self.window_size.unwrap_or(Size::default());
        let window = self.window.unwrap_or(Window::Winit);
        let backend = self.backend.unwrap_or(Backend::Vulkan);

        EngineSettings {
            window_size,
            window,
            backend,
        }
    }
}

enum Window {
    Winit,
}

enum Backend {
    Vulkan,
}

