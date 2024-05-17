use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use error_stack::{Context, Report, Result};
use error_stack::ResultExt;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::{swapchain, sync, Validated, VulkanError, VulkanLibrary};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{CommandBuffer, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions, LayerProperties};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;

use core::Size;

#[derive(Debug)]
pub(crate) enum ThrustlerBackendError {
    BackendUnavailable,
    GraphicalApiError,
    AcquisitionError,
    CreationError,
    AllocationError,
    ShaderError,
}

impl Display for ThrustlerBackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::BackendUnavailable => "Unavailable",
            Self::GraphicalApiError => "Api error",
            Self::CreationError => "Api entity creation error",
            Self::AllocationError => "Allocation error",
            Self::ShaderError => "Shader error",
            Self::AcquisitionError => "Api entity acquisition error"
        };
        write!(f, "{msg}")
    }
}

impl Context for ThrustlerBackendError {}

pub trait VulkanWindow: HasWindowHandle + HasDisplayHandle {}

pub(crate) fn create_vulkan_library(
    window: Arc<dyn VulkanWindow>,
    is_debug: bool,
) -> Result<(Arc<Instance>, Option<DebugUtilsMessenger>), ThrustlerBackendError> {
    let required_validation_layers = ["VK_LAYER_KHRONOS_validation"];

    let required_extensions = InstanceExtensions {
        ext_debug_utils: is_debug,
        ..Surface::required_extensions(&window)
            .attach_printable("Can't get required extensions")
            .change_context(ThrustlerBackendError::BackendUnavailable)?
    };

    let library = VulkanLibrary::new()
        .attach_printable("Can't create vulcan library")
        .change_context(ThrustlerBackendError::BackendUnavailable)?;

    let validation_layers_for_enabling = if is_debug {
        prepare_layers_for_enabling(library.clone(), &required_validation_layers)
    } else {
        vec![]
    };

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_layers: validation_layers_for_enabling,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
        .attach_printable("Can't create instance")
        .change_context(ThrustlerBackendError::CreationError)?;

    let debug_callback = if is_debug {
        let hook_up_callback_result = hook_up_debug_callback(instance.clone());
        match hook_up_callback_result {
            None => println!("Debug callback has been attached"),
            Some(_) => println!("Debug callback hasn't attached")
        }
        hook_up_callback_result
    } else {
        None
    };
    Ok((instance, debug_callback))
}

fn prepare_layers_for_enabling(library: Arc<VulkanLibrary>, required_validation_layers: &[&str]) -> Vec<String> {
    let available_layers = library
        .layer_properties()
        .map(|layer_iter| layer_iter.into_iter().collect::<Vec<LayerProperties>>())
        .unwrap_or(vec![]);

    println!("List of Vulkan debugging layers available to use:");
    let available_layers_names = available_layers.iter()
        .map(|layer_properties| {
            println!("\t{}", layer_properties.name());
            layer_properties.name()
        })
        .collect::<Vec<&str>>();

    required_validation_layers.into_iter()
        .filter_map(|require_layer_name| {
            let is_available = available_layers_names.contains(&require_layer_name);
            if is_available {
                println!("Required validation layer {:?} will be enabled", require_layer_name);
                Some(require_layer_name.to_string())
            } else {
                println!("Required validation layer {:?} is not available", require_layer_name);
                None
            }
        })
        .map(|require_layer_name| require_layer_name)
        .collect::<Vec<String>>()
}

fn hook_up_debug_callback(instance: Arc<Instance>) -> Option<DebugUtilsMessenger> {
    let debug_callback = unsafe {
        DebugUtilsMessengerCallback::new(
            |message_severity, message_type, callback_data| {
                let severity = if message_severity.intersects(DebugUtilsMessageSeverity::ERROR) {
                    "error"
                } else if message_severity.intersects(DebugUtilsMessageSeverity::WARNING) {
                    "warning"
                } else if message_severity.intersects(DebugUtilsMessageSeverity::INFO) {
                    "information"
                } else if message_severity.intersects(DebugUtilsMessageSeverity::VERBOSE) {
                    "verbose"
                } else {
                    panic!("no-impl");
                };

                let message_type = if message_type.intersects(DebugUtilsMessageType::GENERAL) {
                    "general"
                } else if message_type.intersects(DebugUtilsMessageType::VALIDATION) {
                    "validation"
                } else if message_type.intersects(DebugUtilsMessageType::PERFORMANCE) {
                    "performance"
                } else {
                    panic!("no-impl");
                };

                println!(
                    "{} {} {}: {}",
                    callback_data.message_id_name.unwrap_or("unknown"),
                    message_type,
                    severity,
                    callback_data.message.trim()
                );
            },
        )
    };


    DebugUtilsMessenger::new(
        instance,
        DebugUtilsMessengerCreateInfo {
            message_severity: DebugUtilsMessageSeverity::ERROR
                | DebugUtilsMessageSeverity::WARNING
                | DebugUtilsMessageSeverity::INFO
                | DebugUtilsMessageSeverity::VERBOSE,
            message_type: DebugUtilsMessageType::GENERAL
                | DebugUtilsMessageType::VALIDATION
                | DebugUtilsMessageType::PERFORMANCE,
            ..DebugUtilsMessengerCreateInfo::user_callback(debug_callback)
        },
    )
        .ok()
}

pub(crate) fn create_surface(instance: Arc<Instance>,
                             window: Arc<dyn VulkanWindow>,
) -> Result<Arc<Surface>, ThrustlerBackendError> {
    unsafe {
        Surface::from_window_ref(instance, &window)
            .attach_printable("Can't create surface")
            .change_context(ThrustlerBackendError::CreationError)
    }
}

fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    }
}

pub(crate) fn pick_physical_device_and_queue_family_index(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
) -> Result<(Arc<PhysicalDevice>, u32), ThrustlerBackendError> {
    instance
        .enumerate_physical_devices()
        .attach_printable("Enumeration of physical devices failed")
        .change_context(ThrustlerBackendError::AcquisitionError)
        .and_then(|devices| {
            let device_extensions = device_extensions();

            devices
                .filter(|device| device.supported_extensions().contains(&device_extensions))
                .filter_map(|physical_device| {
                    physical_device
                        .queue_family_properties()
                        .iter()
                        .enumerate()
                        .position(|(i, q)| {
                            q.queue_flags.contains(QueueFlags::GRAPHICS)
                                && physical_device.surface_support(i as u32, &surface).unwrap_or(false)
                        })
                        .map(|q| (physical_device, q as u32))
                })
                .min_by_key(|(physical_device, _)| {
                    match physical_device.properties().device_type {
                        // integral gpu is used here deliberately for developing
                        PhysicalDeviceType::IntegratedGpu => 0,
                        /*PhysicalDeviceType::DiscreteGpu => 0,
                        PhysicalDeviceType::IntegratedGpu => 1,
                        PhysicalDeviceType::VirtualGpu => 2,
                        PhysicalDeviceType::Cpu => 3,*/
                        _ => 4,
                    }
                })
                .ok_or(Report::new(ThrustlerBackendError::AcquisitionError)
                    .attach_printable("Fail to find an eligible physical device")
                )
        })
}

pub(crate) fn crete_logical_device(
    physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
) -> Result<(Arc<Device>, Arc<Queue>), ThrustlerBackendError> {
    Device::new(
        physical_device,
        DeviceCreateInfo {
            // here we pass the desired queue family to use by index
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions(),
            ..Default::default()
        },
    )
        .attach_printable("Can't create logical device")
        .change_context(ThrustlerBackendError::CreationError)
        .and_then(|(device, mut queues)| {
            queues.next()
                .ok_or(
                    Report::new(ThrustlerBackendError::AcquisitionError)
                        .attach_printable("Fail to find a queue")
                )
                .map(|queue| (device, queue))
        })
}

pub(crate) fn create_swapchain(
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    surface: Arc<Surface>,
    size: Size,
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), ThrustlerBackendError> {
    let (composite_alpha, min_image_count) = physical_device
        .surface_capabilities(&surface, Default::default())
        .attach_printable("Unable to acquire surface capabilities")
        .change_context(ThrustlerBackendError::AcquisitionError)
        .and_then(|capabilities| {
            let composite_alpha = capabilities.supported_composite_alpha
                .into_iter()
                .next()
                .ok_or(
                    Report::new(ThrustlerBackendError::AcquisitionError)
                        .attach_printable("Unable to acquire composite alpha")
                )?;
            Ok((composite_alpha, capabilities.min_image_count + 1))
        })?;

    let image_format = physical_device
        .surface_formats(&surface, Default::default())
        .attach_printable("Unable to acquire image format")
        .change_context(ThrustlerBackendError::AcquisitionError)?[0].0;

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count, // How many buffers to use in the swapchain
            image_format,
            image_extent: size.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
            composite_alpha,
            ..Default::default()
        },
    )
        .attach_printable("Can't create swapchain")
        .change_context(ThrustlerBackendError::CreationError)
}

pub(crate) fn create_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, ThrustlerBackendError> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
                .attach_printable_lazy(|| "Can't create framebuffer")
                .change_context(ThrustlerBackendError::CreationError)
        })
        .collect()
}

pub(crate) fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Arc<RenderPass>, ThrustlerBackendError> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
        .attach_printable("Can't create pipeline")
        .change_context(ThrustlerBackendError::CreationError)
}

fn fill_render_pass(
    mut builder: RecordingCommandBuffer,
    framebuffer: Arc<Framebuffer>,
    pipeline: Arc<GraphicsPipeline>,
    vertices: Subbuffer<[VulkanVertex]>,
    vertices_count: u32,
) -> Result<RecordingCommandBuffer, ThrustlerBackendError> {
    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )
        .attach_printable("Begin render pass is failed")
        .change_context(ThrustlerBackendError::GraphicalApiError)?
        .bind_pipeline_graphics(pipeline.clone())
        .attach_printable("Bind pipeline is failed")
        .change_context(ThrustlerBackendError::GraphicalApiError)?
        .bind_vertex_buffers(0, vertices.clone())
        .attach_printable("Bind vertex buffer is failed")
        .change_context(ThrustlerBackendError::GraphicalApiError)?;

    unsafe { builder.draw(vertices_count, 1, 0, 0) }
        .attach_printable("Draw is failed")
        .change_context(ThrustlerBackendError::GraphicalApiError)?;

    builder.end_render_pass(SubpassEndInfo::default())
        .attach_printable("End render pass is failed")
        .change_context(ThrustlerBackendError::GraphicalApiError)?;
    Ok(builder)
}

pub(crate) fn create_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    size: Size,
) -> Result<Arc<GraphicsPipeline>, ThrustlerBackendError> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs.clone()),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let vertex_input_state = VulkanVertex::per_vertex()
        .definition(&vs)
        .attach_printable("Can't get vertex definition")
        .change_context(ThrustlerBackendError::GraphicalApiError)?;

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .attach_printable("Can't create pipeline layout creation info")
            .change_context(ThrustlerBackendError::CreationError)?,
    )
        .attach_printable("Can't create pipeline layout")
        .change_context(ThrustlerBackendError::CreationError)?;

    let subpass = Subpass::from(render_pass.clone(), 0).ok_or(
        Report::new(ThrustlerBackendError::AcquisitionError)
            .attach_printable("Can't get subpass from render pass")
    )?;

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: size.into(),
        depth_range: 0.0..=1.0,
    };

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
        .attach_printable("Fail to create graphical pipeline")
        .change_context(ThrustlerBackendError::CreationError)
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub(crate) struct VulkanVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

pub(crate) struct CommandBufferExecutor {
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    standard_memory_allocator: Arc<StandardMemoryAllocator>,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    logical_device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    last_frame_fence: RefCell<Option<Box<dyn GpuFuture>>>,
}

pub enum BufferExecutorResult {
    Done,
    Recreate,
    Fail,
}

impl CommandBufferExecutor {
    pub fn new(
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        standard_memory_allocator: Arc<StandardMemoryAllocator>,
        logical_device: Arc<Device>,
        queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline>,
        swapchain: Arc<Swapchain>,
        framebuffers: Vec<Arc<Framebuffer>>,
    ) -> Self {
        let last_frame_fence = RefCell::new(Some(sync::now(logical_device.clone()).boxed()));
        Self {
            command_buffer_allocator,
            standard_memory_allocator,
            queue,
            pipeline,
            logical_device,
            swapchain,
            framebuffers,
            last_frame_fence,
        }
    }

    pub fn execute_buffer(&self, vertices: Vec<VulkanVertex>) -> BufferExecutorResult {
        swapchain::acquire_next_image(self.swapchain.clone(), None)
            .map_err(|_| {
                BufferExecutorResult::Fail
            })
            .and_then(|(image_index, suboptimal, swapchain_future)| {
                if suboptimal {
                    {
                        let mut mut_last_frame_fence = self.last_frame_fence.borrow_mut();
                        mut_last_frame_fence.as_mut().unwrap().cleanup_finished();
                    }
                    Ok(BufferExecutorResult::Recreate)
                } else {
                    self.create_command_buffer(self.framebuffers[image_index as usize].clone(), vertices)
                        .map_err(|_| BufferExecutorResult::Fail)
                        .and_then(|command_buffer| {
                            self.last_frame_fence
                                .take()
                                .unwrap_or(sync::now(self.logical_device.clone()).boxed())
                                .join(swapchain_future)
                                .then_execute(self.queue.clone(), command_buffer)
                                .map_err(|_| BufferExecutorResult::Fail)
                                .and_then(|exec_future| {
                                    exec_future
                                        .then_swapchain_present(
                                            self.queue.clone(),
                                            SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
                                        )
                                        .then_signal_fence_and_flush()
                                        .map(|_future| {
                                            {
                                                let mut mut_last_frame_fence = self.last_frame_fence.borrow_mut();
                                                mut_last_frame_fence.replace(sync::now(self.logical_device.clone()).boxed());
                                            }
                                            BufferExecutorResult::Done
                                        })
                                        .map_err(Validated::unwrap)
                                        .map_err(|err| match err {
                                            VulkanError::OutOfDate => {
                                                {
                                                    let mut mut_last_frame_fence = self.last_frame_fence.borrow_mut();
                                                    mut_last_frame_fence.as_mut().unwrap().cleanup_finished();
                                                }
                                                BufferExecutorResult::Recreate
                                            }
                                            _ => BufferExecutorResult::Fail
                                        })
                                })
                        })
                }
            })
            .unwrap_or_else(|err| err)
    }
    fn create_command_buffer(&self, framebuffer: Arc<Framebuffer>, vertices: Vec<VulkanVertex>) -> Result<Arc<CommandBuffer>, ThrustlerBackendError> {
        let vertices_count = vertices.len() as u32;
        let vertices = self.create_vertex_buffer(vertices)?;

        let builder = RecordingCommandBuffer::new(
            self.command_buffer_allocator.clone(),
            self.queue.clone().queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
            .attach_printable("Can't create primary command buffer")
            .change_context(ThrustlerBackendError::CreationError)?;

        fill_render_pass(
            builder,
            framebuffer.clone(),
            self.pipeline.clone(),
            vertices.clone(),
            vertices_count,
        )
            ?.end()
            .attach_printable("Render pass stuffing is failed")
            .change_context(ThrustlerBackendError::GraphicalApiError)
    }

    fn create_vertex_buffer(&self, vertices: Vec<VulkanVertex>) -> Result<Subbuffer<[VulkanVertex]>, ThrustlerBackendError> {
        Buffer::from_iter(
            self.standard_memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
            .attach_printable("Unable to allocate vertex buffer")
            .change_context(ThrustlerBackendError::AllocationError)
    }
}