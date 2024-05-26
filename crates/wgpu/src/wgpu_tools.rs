use error_stack::ResultExt;
use wgpu::{Color, CommandBuffer, CommandEncoderDescriptor, Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceTexture, TextureView, TextureViewDescriptor};
use core::error::ThrustlerError;
use error_stack::Result;

pub struct CommandBufferExecutor {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
}

impl CommandBufferExecutor {
    pub fn new(surface: Surface<'static>, device: Device, queue: Queue) -> Self {
        Self {
            surface,
            device,
            queue,
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
        let command_buffer = {
            encoder.begin_render_pass(
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
            encoder.finish()
        };

        Ok(command_buffer)
    }
}