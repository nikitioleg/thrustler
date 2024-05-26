mod wgpu_tools;

use std::sync::Arc;
use error_stack::{Result, ResultExt};
use wgpu::{Adapter, Backend, Backends, CompositeAlphaMode, Device, Instance, InstanceDescriptor, PowerPreference, PresentMode, Queue, Surface, SurfaceTargetUnsafe, WindowHandle};
use core::{Size, ThrustlerBackend};
use core::game_objects::Scene;
use core::error::ThrustlerError;
use std::future;
use std::os::unix::raw::mode_t;
use pollster::FutureExt;
use crate::wgpu_tools::CommandBufferExecutor;

pub struct WgpuBackend {
    instance: Instance,
    toolkit: Option<WgpuToolkit>,
    screen_size: Size,
}

struct WgpuToolkit {
    adapter: Adapter,
    command_buffer_executor: CommandBufferExecutor,
}

impl WgpuBackend {
    pub fn new(screen_size: Size) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            #[cfg(target_arch = "macos")]
            backends: Backends::METAL,
            ..Default::default()
        });

        Self {
            instance,
            toolkit: None,
            screen_size,
        }
    }

    fn get_toolkit(&mut self) -> &mut WgpuToolkit {
        self.toolkit.as_mut().unwrap()
    }

    pub fn init(&mut self,
                window: Arc<dyn WgpuWindow>,
    ) -> Result<(), ThrustlerError> {
        let surface = self.instance.create_surface(window)
            .attach_printable("Can't create wgpu surface")
            .change_context(ThrustlerError::GraphicalBackendError)?;

        let adapter = self.instance
            .enumerate_adapters(Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .ok_or(ThrustlerError::GraphicalBackendError)
            .attach_printable("Can't create wgpu surface")?;


        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        )
            .block_on()
            .attach_printable("Can't create wgpu logical device")
            .change_context(ThrustlerError::GraphicalBackendError)?;

        let surface_caps = surface.get_capabilities(&adapter);

        surface_caps.alpha_modes.iter().for_each(|e| {
            println!("alpha mode {:?}", e)
        });


        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .ok_or(ThrustlerError::GraphicalBackendError)
            .attach_printable("Can't find appropriate format")?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.screen_size.width,
            height: self.screen_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let command_buffer_executor = CommandBufferExecutor::new(surface, device, queue);
        let toolkit = WgpuToolkit {
            adapter,
            command_buffer_executor,
        };

        self.toolkit = Some(toolkit);

        Ok(())
    }
}

pub trait WgpuWindow: WindowHandle {}

impl ThrustlerBackend for WgpuBackend {
    fn draw_scene(&mut self, scene: &Box<dyn Scene>) {
        let toolkit = self.get_toolkit();
        toolkit.command_buffer_executor.execute_buffer();
    }
}
