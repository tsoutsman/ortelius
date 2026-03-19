use vello::wgpu;

use crate::render::SceneParams;

pub(super) fn group0_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Line Group 0 Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(
                    std::num::NonZeroU64::new(std::mem::size_of::<SceneParams>() as u64).unwrap(),
                ),
            },
            count: None,
        }],
    })
}

pub(super) fn group1_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
                    min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<[f32; 4]>() as u64).unwrap(),
                    ),
                },
                count: None,
            },
        ],
    })
}

#[inline]
pub(super) fn cull_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::ComputePipeline {
    let cull_shader =
        device.create_shader_module(wgpu::include_wgsl!("../../../shader/line/cull.wgsl"));

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
pub(super) fn render_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    is_miter: bool,
) -> wgpu::RenderPipeline {
    // TODO
    let is_miter = true;
    let (vertex_shader, fragment_shader) = if is_miter {
        (
            device.create_shader_module(wgpu::include_wgsl!(
                "../../../shader/line/miter/vertex.wgsl"
            )),
            device.create_shader_module(wgpu::include_wgsl!(
                "../../../shader/line/miter/fragment.wgsl"
            )),
        )
    } else {
        panic!();
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../../shader/line/round.wgsl"));
    };

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
