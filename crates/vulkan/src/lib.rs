use std::sync::Arc;

use error_stack::{Result, ResultExt};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::device::{Device, Queue};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::debug::DebugUtilsMessenger;
use vulkano::instance::Instance;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain};

use core::{Size, ThrustlerBackend};
use core::error::ThrustlerError;
use vulkano_tools::{BufferExecutor, VulkanWindow};

use crate::shaders::{simple_fragment_shader, simple_vertex_shader};
use crate::vulkano_tools::{create_command_buffers, create_framebuffers, create_pipeline, create_render_pass, create_surface, create_swapchain, create_vulkan_library, crete_logical_device, pick_physical_device_and_queue_family_index, ThrustlerBackendError, VulkanVertex};

pub mod vulkano_tools;
mod shaders;

pub struct VulkanBackend {
    screen_size: Size,
    vulkano_toolkit: Option<VulkanoToolkit>,
}

#[allow(unused)]
struct VulkanoToolkit {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    logical_device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    buffer_executor: BufferExecutor,
    //have to hold this struct to keep getting debug logs
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

    fn get_toolkit(&self) -> &VulkanoToolkit {
        self.vulkano_toolkit.as_ref().unwrap()
    }
}

impl ThrustlerBackend for VulkanBackend {
    type Window = Arc<dyn VulkanWindow>;

    fn init(&mut self, window: Self::Window) -> Result<(), ThrustlerError> {
        let toolkit = create_vulkano_toolkit(self.screen_size, window)
            .change_context(ThrustlerError::GraphicalBackendError)?;
        self.vulkano_toolkit = Some(toolkit);
        Ok(())
    }

    fn test_draw(&mut self) {
        let toolkit = self.get_toolkit();
        toolkit.buffer_executor.execute_buffer(|buffer_index| toolkit.buffers[buffer_index].clone());
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

    let vertex1 = VulkanVertex {
        position: [0.0, -0.5],
    };
    let vertex2 = VulkanVertex {
        position: [0.5, 0.5],
    };
    let vertex3 = VulkanVertex {
        position: [-0.5, 0.5],
    };

    let buffers = create_command_buffers(
        command_buffer_allocator.clone(),
        memory_allocator.clone(),
        queue.clone(),
        pipeline.clone(),
        &framebuffers,
        vec![vertex1, vertex2, vertex3],
    )?;

    let buffer_executor = BufferExecutor::new(logical_device.clone(), queue.clone(), swapchain.clone());

    Ok(VulkanoToolkit {
        instance,
        physical_device,
        logical_device,
        queue,
        surface,
        swapchain,
        framebuffers,
        render_pass,
        pipeline,
        buffers,
        buffer_executor,
        debug_callback,
    })
}