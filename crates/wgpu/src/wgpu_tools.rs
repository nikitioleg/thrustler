use std::sync::Arc;
use error_stack::ResultExt;
use wgpu::{Adapter, Backends, BlendState, Color, ColorTargetState, ColorWrites, CommandBuffer, CommandEncoderDescriptor, CompositeAlphaMode, Device, Face, FragmentState, FrontFace, include_wgsl, Instance, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, VertexState};
use core::error::ThrustlerError;
use error_stack::Result;
use pollster::FutureExt;
use core::Size;
use crate::WgpuWindow;

pub(crate) fn create_surface(instance: &Instance, window: Arc<dyn WgpuWindow>) -> Result<Surface<'static>, ThrustlerError> {
    instance.create_surface(window)
        .attach_printable("Can't create wgpu surface")
        .change_context(ThrustlerError::GraphicalBackendError)
}

pub(crate) fn create_adapter(instance: &Instance, surface: &Surface<'static>) -> Result<Adapter, ThrustlerError> {
    instance
        .enumerate_adapters(Backends::all())
        .into_iter()
        .filter(|adapter| adapter.is_surface_supported(surface))
        .next()
        .ok_or(ThrustlerError::GraphicalBackendError)
        .attach_printable("Can't create wgpu surface")
}

pub(crate) fn pick_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue), ThrustlerError> {
    adapter.request_device(
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
        .change_context(ThrustlerError::GraphicalBackendError)
}

pub(crate) fn create_surface_config(screen_size: Size, surface: &Surface<'static>, adapter: &Adapter) -> Result<SurfaceConfiguration, ThrustlerError> {
    let surface_caps = surface.get_capabilities(&adapter);

    surface_caps.alpha_modes.iter().for_each(|e| {
        println!("alpha mode {:?}", e)
    });

    let surface_format = surface_caps.formats.iter()
        .find(|f| f.is_srgb())
        .copied()
        .ok_or(ThrustlerError::GraphicalBackendError)
        .attach_printable("Can't find appropriate format")?;

    Ok(SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: screen_size.width,
        height: screen_size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    })
}

pub(crate) fn create_render_pipeline(device: &Device, config: &SurfaceConfiguration) -> RenderPipeline {
    let shader_module = device.create_shader_module(include_wgsl!(
            "../../../assets/shaders/wgsl/simple_shader.wgsl"
        ));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Thruster pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[
                Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })
            ],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub struct CommandBufferExecutor {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl CommandBufferExecutor {
    pub fn new(surface: Surface<'static>, device: Device, queue: Queue, render_pipeline: RenderPipeline) -> Self {
        Self {
            surface,
            device,
            queue,
            render_pipeline,
        }
    }

    pub fn execute_buffer(&self) {
        //todo remove unwrap
        let (current_texture, texture_view) = self.acquire_next_surface().unwrap();
        let command_buffer = self.fill_render_pass(texture_view).unwrap();
        self.queue.submit(std::iter::once(command_buffer));
        current_texture.present();
    }

    fn acquire_next_surface(&self) -> Result<(SurfaceTexture, TextureView), ThrustlerError> {
        let current_texture = self.surface.get_current_texture()
            .attach_printable("Can't get current texture")
            .change_context(ThrustlerError::GraphicalBackendError)?;

        let texture_view = current_texture.texture.create_view(&TextureViewDescriptor::default());
        Ok((current_texture, texture_view))
    }

    fn fill_render_pass(&self, texture_view: TextureView) -> Result<CommandBuffer, ThrustlerError> {
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Thrustler encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(
                &RenderPassDescriptor {
                    label: Some("Thrustler encoder"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(
                                Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                }
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        };

        Ok(encoder.finish())
    }
}