use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use super::to_buffer;

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
    pub(crate) fn create_group_layout(
        device: &wgpu::Device,
        layer_name: &str,
    ) -> wgpu::BindGroupLayout {
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

    pub(crate) fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        layer_name: &str,
    ) -> wgpu::BindGroup {
        let name = format!("{layer_name} bind group 0");
        let scene_buffer = to_buffer(device, &name, self);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&name),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        })
    }
}
