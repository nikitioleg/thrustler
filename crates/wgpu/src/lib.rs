use std::sync::Arc;

use error_stack::{Result, ResultExt};
use pollster::FutureExt;
use wgpu::{Adapter, Instance, InstanceDescriptor, WindowHandle};

use core::{Size, ThrustlerBackend};
use core::error::ThrustlerError;
use core::game_objects::Scene;

use crate::wgpu_tools::{CommandBufferExecutor, create_adapter, create_render_pipeline, create_surface, create_surface_config, pick_device_and_queue};

mod wgpu_tools;

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
        let surface = create_surface(&self.instance, window.clone())?;
        let adapter = create_adapter(&self.instance, &surface)?;
        let (device, queue) = pick_device_and_queue(&adapter)?;
        let config = create_surface_config(self.screen_size, &surface, &adapter)?;
        let render_pipeline = create_render_pipeline(&device, &config);

        surface.configure(&device, &config);

        let command_buffer_executor = CommandBufferExecutor::new(surface, device, queue, render_pipeline);
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
