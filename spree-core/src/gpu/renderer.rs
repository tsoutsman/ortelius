use vello::wgpu::{
    self, CommandEncoder, Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureView,
};

use super::{LayerRenderer, SceneParams, Wrapper};
use crate::layout::PlotInstanceLayout;

pub struct Renderer<'a> {
    line: Wrapper<super::line::Renderer>,
    scatter: Wrapper<super::scatter::Renderer>,
    grid: Wrapper<super::grid::Renderer>,
    surface: Surface<'a>,
    device: Device,
    msaa_view: TextureView,
    queue: Queue,
    config: SurfaceConfiguration,
}

fn create_msaa_texture(device: &Device, width: u32, height: u32) -> TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("multisample framebuffer"),
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
    fn usee<'b, R, I>(
        &self,
        stuff: &Wrapper<R>,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        scene_params: SceneParams,
        clear: Option<wgpu::Color>,
        datas: I,
    ) where
        R: LayerRenderer,
        I: Iterator<Item = R::Layer<'b>>,
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{} render pass", R::NAME)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.msaa_view,
                resolve_target: Some(view),
                ops: wgpu::Operations {
                    load: match clear {
                        Some(colour) => wgpu::LoadOp::Clear(colour),
                        None => wgpu::LoadOp::Load,
                    },
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&stuff.render_pipeline);

        let bind_group0 =
            scene_params.create_bind_group(&self.device, &stuff.group_0_layout, "line");
        render_pass.set_bind_group(0, &bind_group0, &[]);

        for data in datas {
            let (vertices, instances) = stuff.inner.counts(&data);

            let bind_group1 =
                stuff
                    .inner
                    .create_per_layer_group(&self.device, &stuff.group_1_layout, &data);
            render_pass.set_bind_group(1, &bind_group1, &[]);

            render_pass.draw(vertices, instances);
        }
    }

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
            line: Wrapper::new(&device),
            scatter: Wrapper::new(&device),
            grid: Wrapper::new(&device),
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
        I: Iterator<Item = crate::Layer<'b>>,
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
        let background = wgpu::Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        };
        layers.enumerate().for_each(|(i, layer)| {
            let clear = if i == 0 { Some(background) } else { None };
            self.render_layer(layer, &mut encoder, &view, clear, scene_params)
        });

        self.queue.submit([encoder.finish()]);
        output.present();

        self.device.poll(wgpu::PollType::Poll).unwrap();
    }

    fn render_layer(
        &self,
        layer: crate::Layer,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        clear: Option<wgpu::Color>,
        scene_params: SceneParams,
    ) {
        match layer {
            crate::Layer::Title(_) => todo!(),
            crate::Layer::XAxis { .. } => todo!(),
            crate::Layer::YAxis { .. } => todo!(),
            crate::Layer::Lines(lines) => self.usee(
                &self.line,
                encoder,
                view,
                scene_params,
                clear,
                lines.into_iter(),
            ),
            crate::Layer::Scatters(scatters) => self.usee(
                &self.scatter,
                encoder,
                view,
                scene_params,
                clear,
                scatters.into_iter(),
            ),
            crate::Layer::Grid(grid) => self.usee(
                &self.grid,
                encoder,
                view,
                scene_params,
                clear,
                std::iter::once(grid),
            ),
        };
    }
}
