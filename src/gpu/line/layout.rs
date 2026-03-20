use vello::wgpu;

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
    let cull_shader = device.create_shader_module(wgpu::include_wgsl!("cull.wgsl"));

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
    let shader = if is_miter {
        device.create_shader_module(wgpu::include_wgsl!("miter.wgsl"))
    } else {
        device.create_shader_module(wgpu::include_wgsl!("round.wgsl"))
    };

    todo!()
}
