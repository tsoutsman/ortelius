mod buffer;
mod line;
mod renderer;
mod scatter;
mod scene_params;

use std::{fmt::Debug, mem::size_of, ops::Range};

use bytemuck::{Pod, Zeroable};
use vello::wgpu::{self, ShaderModuleDescriptor, util::DeviceExt};

pub(crate) use self::{buffer::GpuBuffer, renderer::Renderer, scene_params::SceneParams};

struct Wrapper<R>
where
    R: LayerRenderer,
{
    render_pipeline: wgpu::RenderPipeline,
    group_0_layout: wgpu::BindGroupLayout,
    group_1_layout: wgpu::BindGroupLayout,
    inner: R,
}

impl<R> Wrapper<R>
where
    R: LayerRenderer,
{
    fn new(device: &wgpu::Device) -> Self {
        let group_0_layout = SceneParams::create_group_layout(device, R::NAME);
        let group_1_layout = R::per_layer_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("render {} pipeline layout", R::NAME)),
            bind_group_layouts: &[&group_0_layout, &group_1_layout],
            immediate_size: 0,
        });

        let inner = R::new();

        Self {
            render_pipeline: inner.create_render_pipeline(device, &pipeline_layout),
            group_0_layout,
            group_1_layout,
            inner,
        }
    }
}

pub(crate) trait Layer {
    const HAS_DATA: bool;

    // This will only be called if HAS_DATA is true.
    fn as_entire_binding(&self) -> wgpu::BindingResource<'_>;
}

trait LayerRenderer: Sized {
    type Layer<'a>: Layer;
    type PerLayerParams: Debug + Pod + Zeroable;

    const NAME: &'static str;

    fn new() -> Self;

    fn shader(&self) -> ShaderModuleDescriptor<'static>;

    fn create_render_pipeline(
        &self,
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let sample_count = 4;
        let shader = device.create_shader_module(self.shader());

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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            multiview_mask: None,
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
                    std::num::NonZeroU64::new(size_of::<Self::PerLayerParams>() as u64).unwrap(),
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
                std::num::NonZeroU64::new(size_of::<Self::PerLayerParams>() as u64).unwrap(),
            ),
        },
        count: None,
    }];

    fn per_layer_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} group 1 layout", Self::NAME)),
            entries: if Self::Layer::HAS_DATA {
                &Self::_WITH_POINTS
            } else {
                &Self::_WITHOUT_POINTS
            },
        })
    }

    fn create_per_layer_params(&self, layer: &Self::Layer<'_>) -> Self::PerLayerParams;

    fn create_per_layer_group<'a>(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        layer: &Self::Layer<'a>,
    ) -> wgpu::BindGroup {
        let name = format!("{} bind group 1", Self::NAME);
        let params = self.create_per_layer_params(layer);
        let params_buffer = to_buffer(device, &name, &params);

        if Self::Layer::HAS_DATA {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&name),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: layer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            })
        } else {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&name),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                }],
            })
        }
    }

    fn counts(&self, data: &Self::Layer<'_>) -> (Range<u32>, Range<u32>);
}

fn to_buffer<T>(device: &wgpu::Device, name: &str, value: &T) -> wgpu::Buffer
where
    T: Pod + Zeroable + std::fmt::Debug,
{
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(name),
        contents: bytemuck::bytes_of(value),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}
