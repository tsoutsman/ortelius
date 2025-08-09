mod line;

use bytemuck::{Pod, Zeroable};
use line::LineRenderer;
use vello::wgpu::{self, CommandEncoder, Device, Queue, Surface, TextureFormat, TextureView};

use crate::Layer;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct SceneParams {
    scale: [f32; 2],
    offset: [f32; 2],
    padding: [f32; 4],
}

pub struct Renderer<'a> {
    line: LineRenderer,
    surface: Surface<'a>,
    device: Device,
    msaa_view: TextureView,
    queue: Queue,
}

impl<'a> Renderer<'a> {
    pub(crate) fn new(device: Device, surface: Surface<'a>) -> Self {
        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisample Framebuffer"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            // TODO
            format: TextureFormat::R16Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Self {
            line: LineRenderer::new(&device),
            device,
            msaa_view: msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            surface,
            queue: todo!(),
        }
    }

    pub(crate) fn render<'b, I>(&self, layers: I)
    where
        I: Iterator<Item = Layer<'b>>,
    {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Line Render Encoder"),
            });

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        layers.for_each(|layer| self.render_layer(layer, &mut encoder, &view));

        self.queue.submit([encoder.finish()]);
        output.present();

        self.device.poll(wgpu::PollType::Poll).unwrap();
    }

    fn render_layer(&self, layer: Layer, encoder: &mut CommandEncoder, view: &TextureView) {
        match layer {
            Layer::XAxis => todo!(),
            Layer::YAxis => todo!(),
            Layer::Lines(lines) => self.line.render(
                &self.device,
                encoder,
                view,
                &self.msaa_view,
                lines.into_iter(),
            ),
            Layer::Scatter => todo!(),
        }
    }
}
