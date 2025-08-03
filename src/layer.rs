use bytemuck::{Pod, Zeroable};
use vello::wgpu::{self, BufferUsages, CommandBuffer};
use wgpu::util::DeviceExt;

use crate::GpuBuffer;

pub enum Layer<'a> {
    XAxis,
    YAxis,
    Line(&'a Line),
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct SceneParams {
    scale: [f32; 2],
    offset: [f32; 2],
    padding: [f32; 4],
}

pub struct LineRenderer {
    _cull_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    group0_layout: wgpu::BindGroupLayout,
    group1_layout: wgpu::BindGroupLayout,
}

impl LineRenderer {
    pub fn create(device: &wgpu::Device) -> Self {
        let group0_layout = Line::group0_layout(device);
        let group1_layout = Line::group1_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Line Pipeline Layout"),
            bind_group_layouts: &[&group0_layout, &group1_layout],
            push_constant_ranges: &[],
        });

        LineRenderer {
            _cull_pipeline: Line::cull_pipeline(device, &pipeline_layout),
            render_pipeline: Line::render_pipeline(device, &pipeline_layout),
            group0_layout,
            group1_layout,
        }
    }

    pub fn create_group0(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let scene_params = SceneParams {
            scale: [1.0, 1.0],
            offset: [0., 0.0],
            padding: [0.; 4],
        };
        let scene_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Scene Params Buffer"),
            contents: bytemuck::bytes_of(&scene_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Bind Group 0"),
            layout: &self.group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        })
    }

    pub fn create_group1(&self, device: &wgpu::Device, line: &Line) -> wgpu::BindGroup {
        let thickness_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Thickness Buffer"),
            contents: bytemuck::bytes_of(&line.thickness),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Bind Group 1"),
            layout: &self.group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: line.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: thickness_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn render<'a, I>(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        msaa_view: &wgpu::TextureView,
        lines: I,
    ) where
        I: Iterator<Item = &'a Line>,
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Line Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: msaa_view,
                resolve_target: Some(view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        let bind_group0 = self.create_group0(device);
        render_pass.set_bind_group(0, &bind_group0, &[]);

        for line in lines {
            let bind_group1 = self.create_group1(device, line);
            render_pass.set_bind_group(1, &bind_group1, &[]);
            render_pass.draw(0..(line.buffer.len() * 2) as u32, 0..1);
        }
    }
}

pub struct Line {
    buffer: GpuBuffer<f32>,
    thickness: f32,
}

impl Line {
    pub fn new(device: &wgpu::Device, xs: &[f32], ys: &[f32]) -> Self {
        assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC;

        Line {
            buffer: GpuBuffer::new(device, usage, 2 * xs.len(), |buffer| {
                for i in 0..xs.len() {
                    buffer[i * 2] = xs[i];
                    buffer[i * 2 + 1] = ys[i];
                }
            }),
            thickness: 0.005,
        }
    }

    pub fn append(&mut self, device: &wgpu::Device, x: f32, y: f32) -> CommandBuffer {
        self.buffer.extend(device, 2, |buffer| {
            buffer[0] = x;
            buffer[1] = y;
        })
    }

    pub fn extend(&mut self, device: &wgpu::Device, xs: &[f32], ys: &[f32]) -> CommandBuffer {
        assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");

        self.buffer.extend(device, 2 * xs.len(), |buffer| {
            for i in 0..xs.len() {
                buffer[i * 2] = xs[i];
                buffer[i * 2 + 1] = ys[i];
            }
        })
    }

    #[inline]
    pub(crate) fn group0_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Line Group 0 Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<SceneParams>() as u64)
                            .unwrap(),
                    ),
                },
                count: None,
            }],
        })
    }

    #[inline]
    pub(crate) fn group1_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Line Group 1 Layout"),
            entries: &[
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
                            std::num::NonZeroU64::new(std::mem::size_of::<f32>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                },
            ],
        })
    }

    #[inline]
    pub(crate) fn cull_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::ComputePipeline {
        let cull_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shader/line/cull.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Line Cull Pipeline"),
            // TODO
            layout: Some(pipeline_layout),
            module: &cull_shader,
            entry_point: Some("cs_main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        })
    }

    #[inline]
    pub(crate) fn render_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let vertex_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shader/line/vertex.wgsl"));
        let fragment_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shader/line/fragment.wgsl"));

        let sample_count = 4;
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
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
                // topology: wgpu::PrimitiveTopology::LineStrip,
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
}
