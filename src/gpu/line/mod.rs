use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use super::to_buffer;
use crate::layer::Line;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(super) struct PerLineParams {
    colour: [f32; 4],
    thickness: f32,
    _padding: [f32; 3],
}

pub(super) struct Temp {
    is_miter: bool,
}

impl super::Rendererr for Temp {
    type Data<'a> = Line<'a>;

    type PerLayerBinding = PerLineParams;

    const NAME: &'static str = "line";

    const SHADER: wgpu::ShaderModuleDescriptor<'static> = wgpu::include_wgsl!("round.wgsl");

    const USES_POINTS: bool = true;

    fn new() -> Self {
        Self { is_miter: false }
    }

    fn counts(&self, data: &Self::Data<'_>) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
        if self.is_miter {
            (0..(data.data.len() * 2) as u32, 0..1)
        } else {
            (0..6, 0..(data.data.len() as u32).saturating_sub(1))
        }
    }

    fn create_per_layer_group<'a>(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        data: &Self::Data<'a>,
    ) -> wgpu::BindGroup {
        let params = PerLineParams {
            colour: data.colour,
            thickness: data.thickness,
            _padding: [0., 0., 0.],
        };
        let params_buffer = to_buffer(device, &params);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("line bind group 1"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: data.data.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        })
    }
}
