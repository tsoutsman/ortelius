mod line;

use bytemuck::{Pod, Zeroable};
use line::LineRenderer;
use vello::wgpu::{
    self, CommandEncoder, Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureView,
};

use crate::{Layer, layout::PlotInstanceLayout};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct SceneParams {
    pub(crate) projection_matrix: [[f32; 4]; 4],
    pub(crate) xclip_bounds: [f32; 2],
    pub(crate) yclip_bounds: [f32; 2],
    pub(crate) viewport_size: [f32; 2],
    pub(crate) _padding: [f32; 2],
}

pub struct Renderer<'a> {
    line: LineRenderer,
    surface: Surface<'a>,
    device: Device,
    msaa_view: TextureView,
    queue: Queue,
    config: SurfaceConfiguration,
}

fn create_msaa_texture(device: &Device, width: u32, height: u32) -> TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisample Framebuffer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            // TODO
            format: TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

impl<'a> Renderer<'a> {
    pub(crate) fn new(device: Device, queue: Queue, surface: Surface<'a>) -> Self {
        let msaa_texture = create_msaa_texture(&device, 1600, 1200);

        // let surface_caps = surface.get_capabilities(&adapter);
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .find(|f| f.is_srgb())
        //     .copied()
        //     .unwrap_or(surface_caps.formats[0]);

        Self {
            line: LineRenderer::new(&device),
            device,
            msaa_view: msaa_texture,
            surface,
            queue,
            config: SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8Unorm,
                width: 1600,
                height: 1200,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);
            self.msaa_view = create_msaa_texture(&self.device, width, height);

            // TODO: reconfigure line renderer?
            // self.is_surface_configured = true;
        }
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn render<'b, I>(&self, layers: I, layout: &PlotInstanceLayout)
    where
        I: Iterator<Item = &'b Layer>,
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

        let scene_params = layout.scene_params();
        layers.for_each(|layer| self.render_layer(layer, &mut encoder, &view, scene_params));

        self.queue.submit([encoder.finish()]);
        output.present();

        self.device.poll(wgpu::PollType::Poll).unwrap();
    }

    fn render_layer(
        &self,
        layer: &Layer,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        scene_params: SceneParams,
    ) {
        match layer {
            Layer::Title(_) => todo!(),
            Layer::XAxis { .. } => todo!(),
            Layer::YAxis { .. } => todo!(),
            Layer::Lines(lines) => self.line.render(
                &self.device,
                encoder,
                view,
                &self.msaa_view,
                scene_params,
                lines.iter(),
            ),
            Layer::Scatter => todo!(),
        }
    }
}
