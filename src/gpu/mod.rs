mod buffer;
mod line;
mod scatter;

use std::{mem::size_of, ops::Range};

pub(crate) use buffer::GpuBuffer;
use bytemuck::{Pod, Zeroable};
use scatter::ScatterRenderer;
use vello::wgpu::{
    self, BindGroup, CommandEncoder, Device, Queue, ShaderModuleDescriptor, Surface,
    SurfaceConfiguration, TextureFormat, TextureView, util::DeviceExt,
};

use crate::{Layer, layout::PlotInstanceLayout};

struct Stuff<R>
where
    R: Rendererr,
{
    render_pipeline: wgpu::RenderPipeline,
    group_0_layout: wgpu::BindGroupLayout,
    group_1_layout: wgpu::BindGroupLayout,
    rr: R,
}

trait Rendererr: Sized {
    type Data<'a>;
    type PerLayerBinding: Pod + Zeroable;

    const NAME: &'static str;
    const SHADER: ShaderModuleDescriptor<'static>;

    const USES_POINTS: bool;

    fn new() -> Self;

    fn init(device: &wgpu::Device) -> Stuff<Self> {
        let group_0_layout = SceneParams::group_layout(device, "line");
        let group_1_layout = Self::per_layer_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render line pipeline layout"),
            bind_group_layouts: &[&group_0_layout, &group_1_layout],
            push_constant_ranges: &[],
        });

        Stuff {
            render_pipeline: Self::create_render_pipeline(device, &pipeline_layout),
            group_0_layout,
            group_1_layout,
            rr: Self::new(),
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let sample_count = 4;
        let shader = device.create_shader_module(Self::SHADER);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} render pipeline", Self::NAME)),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // format: config.format,
                    // TODO
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            cache: None,
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
        })
    }

    #[doc(hidden)]
    const _WITH_POINTS: [wgpu::BindGroupLayoutEntry; 2] = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(
                    std::num::NonZeroU64::new(size_of::<Self::PerLayerBinding>() as u64).unwrap(),
                ),
            },
            count: None,
        },
    ];
    #[doc(hidden)]
    const _WITHOUT_POINTS: [wgpu::BindGroupLayoutEntry; 1] = [wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
                std::num::NonZeroU64::new(size_of::<Self::PerLayerBinding>() as u64).unwrap(),
            ),
        },
        count: None,
    }];

    fn per_layer_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} group 1 layout", Self::NAME)),
            entries: if Self::USES_POINTS {
                &Self::_WITH_POINTS
            } else {
                &Self::_WITHOUT_POINTS
            },
        })
    }

    fn counts(&self, data: &Self::Data<'_>) -> (Range<u32>, Range<u32>);

    fn create_per_layer_group<'a>(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        data: &Self::Data<'a>,
    ) -> BindGroup;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct SceneParams {
    pub(crate) projection_matrix: [[f32; 4]; 4],
    pub(crate) xclip_bounds: [f32; 2],
    pub(crate) yclip_bounds: [f32; 2],
    pub(crate) viewport_size: [f32; 2],
    pub(crate) _padding: [f32; 2],
}

impl SceneParams {
    fn group_layout(device: &wgpu::Device, layer_name: &str) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{layer_name} group 0 layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<Self>() as u64).unwrap(),
                    ),
                },
                count: None,
            }],
        })
    }

    fn create_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        layer_name: &str,
    ) -> wgpu::BindGroup {
        let scene_buffer = to_buffer(device, self);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{layer_name} bind group 0")),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        })
    }
}

fn to_buffer<T>(device: &wgpu::Device, value: &T) -> wgpu::Buffer
where
    T: Pod + Zeroable + std::fmt::Debug,
{
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Line Thickness Buffer"),
        contents: bytemuck::bytes_of(value),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub struct Renderer<'a> {
    line: Stuff<line::Temp>,
    scatter: ScatterRenderer,
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
    fn usee<'b, R, I>(
        &self,
        stuff: &Stuff<R>,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        scene_params: SceneParams,
        clear: Option<wgpu::Color>,
        datas: I,
    ) where
        R: Rendererr,
        I: Iterator<Item = R::Data<'b>>,
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
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&stuff.render_pipeline);

        let bind_group0 = scene_params.create_group(&self.device, &stuff.group_0_layout, "line");
        render_pass.set_bind_group(0, &bind_group0, &[]);

        for data in datas {
            let (a, b) = stuff.rr.counts(&data);

            let bind_group1 =
                stuff
                    .rr
                    .create_per_layer_group(&self.device, &stuff.group_1_layout, &data);
            render_pass.set_bind_group(1, &bind_group1, &[]);

            render_pass.draw(a, b);
            // render_pass.draw(0..(line.data.len() * 2) as u32, 0..1);

            // if self.is_miter {
            //     render_pass.draw(0..(line.data.len() * 2) as u32, 0..1);
            // } else {
            //     render_pass.draw(0..6, 0..(line.data.len() as
            // u32).saturating_sub(1)); }
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
            line: line::Temp::init(&device),
            scatter: ScatterRenderer::new(&device),
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

        let scene_params = layout.scene_params();
        layers.for_each(|layer| self.render_layer(layer, &mut encoder, &view, scene_params));

        self.queue.submit([encoder.finish()]);
        output.present();

        self.device.poll(wgpu::PollType::Poll).unwrap();
    }

    fn render_layer(
        &self,
        layer: Layer,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        scene_params: SceneParams,
    ) {
        match layer {
            Layer::Title(_) => todo!(),
            Layer::XAxis { .. } => todo!(),
            Layer::YAxis { .. } => todo!(),
            Layer::Lines(lines) => self.usee(
                &self.line,
                encoder,
                view,
                scene_params,
                Some(wgpu::Color {
                    r: 1.,
                    g: 1.,
                    b: 1.,
                    a: 1.,
                }),
                lines.into_iter(),
            ),
            Layer::Scatter(scatter) => self.scatter.render(
                &self.device,
                encoder,
                view,
                &self.msaa_view,
                scene_params,
                std::iter::once(scatter),
            ),
        }
    }
}
