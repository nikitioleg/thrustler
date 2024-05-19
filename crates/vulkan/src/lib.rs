use std::sync::Arc;

use error_stack::{Result, ResultExt};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::instance::debug::DebugUtilsMessenger;
use vulkano::memory::allocator::StandardMemoryAllocator;

use core::{Size, ThrustlerBackend};
use core::error::ThrustlerError;
use core::game_objects::Scene;
use vulkano_tools::VulkanWindow;

use crate::shaders::{simple_fragment_shader, simple_vertex_shader};
use crate::vulkano_tools::*;

pub mod vulkano_tools;
mod shaders;

pub struct VulkanBackend {
    screen_size: Size,
    vulkano_toolkit: Option<VulkanoToolkit>,
}

struct VulkanoToolkit {
    command_buffer_executor: CommandBufferExecutor,
    //have to hold this struct to keep getting debug logs
    #[allow(unused)]
    debug_callback: Option<DebugUtilsMessenger>,
}

impl VulkanBackend {
    pub fn new(
        size: Size,
    ) -> VulkanBackend {
        Self {
            screen_size: size,
            vulkano_toolkit: None,
        }
    }

    fn get_toolkit(&mut self) -> &mut VulkanoToolkit {
        self.vulkano_toolkit.as_mut().unwrap()
    }

    pub fn init(&mut self, window: Arc<dyn VulkanWindow>) -> Result<(), ThrustlerError> {
        let toolkit = create_vulkano_toolkit(self.screen_size, window)
            .change_context(ThrustlerError::GraphicalBackendError)
            .attach_printable("Vulkan toolkit initialization error")?;
        self.vulkano_toolkit = Some(toolkit);
        Ok(())
    }
}

impl ThrustlerBackend for VulkanBackend {
    fn draw_scene(&mut self, scene: &Box<dyn Scene>) {
        let toolkit = self.get_toolkit();
        let game_objects = scene.get_scene_objects();

        toolkit.command_buffer_executor.execute_buffer(game_objects);
    }
}

fn create_vulkano_toolkit(
    size: Size,
    window: Arc<dyn VulkanWindow>,
) -> Result<VulkanoToolkit, ThrustlerBackendError> {
    let (instance, debug_callback) = create_vulkan_library(
        window.clone(),
        true,
    )?;

    let surface = create_surface(instance.clone(), window.clone())?;

    let (physical_device, queue_family_index) = pick_physical_device_and_queue_family_index(
        instance.clone(), surface.clone())?;
    let (logical_device, queue) = crete_logical_device(
        physical_device.clone(),
        queue_family_index,
    )?;

    let (swapchain, swapchain_images) = create_swapchain(
        physical_device.clone(),
        logical_device.clone(),
        surface.clone(),
        size,
    )?;

    let render_pass = create_render_pass(
        logical_device.clone(),
        swapchain.clone(),
    )?;

    let framebuffers = create_framebuffers(
        &swapchain_images,
        render_pass.clone(),
    )?;

    let vertex_shader = simple_vertex_shader::load(
        logical_device.clone()
    )
        .attach_printable("Vertex shader loading error")
        .change_context(ThrustlerBackendError::ShaderError)?;

    let fragment_shader = simple_fragment_shader::load(
        logical_device.clone()
    )
        .attach_printable("Fragment shader loading error")
        .change_context(ThrustlerBackendError::ShaderError)?;

    let pipeline = create_pipeline(
        logical_device.clone(),
        vertex_shader,
        fragment_shader,
        render_pass.clone(),
        size,
    )?;

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(logical_device.clone()));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        logical_device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default()),
    );

    let command_buffer_executor = CommandBufferExecutor::new(
        command_buffer_allocator.clone(),
        memory_allocator.clone(),
        logical_device.clone(),
        queue.clone(),
        pipeline.clone(),
        swapchain.clone(),
        framebuffers.clone(),
    );

    Ok(VulkanoToolkit {
        command_buffer_executor,
        debug_callback,
    })
}