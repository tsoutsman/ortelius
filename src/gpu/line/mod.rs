use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use crate::layer::Line;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(super) struct PerLineParams {
    colour: [f32; 4],
    thickness: f32,
    _padding: [f32; 3],
}

pub(super) struct Renderer {
    is_miter: bool,
}

impl super::LayerRenderer for Renderer {
    type Layer<'a> = Line<'a>;

    type PerLayerParams = PerLineParams;

    const NAME: &'static str = "line";

    fn new() -> Self {
        Self { is_miter: false }
    }

    fn shader(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        #[allow(unreachable_code)]
        if self.is_miter {
            todo!("miter shader needs to be updated");
            wgpu::include_wgsl!("miter.wgsl")
        } else {
            wgpu::include_wgsl!("round.wgsl")
        }
    }

    fn counts(&self, data: &Self::Layer<'_>) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
        if self.is_miter {
            (0..(data.data.len() * 2) as u32, 0..1)
        } else {
            (0..6, 0..(data.data.len() as u32).saturating_sub(1))
        }
    }

    fn create_per_layer_params<'a>(&self, data: &Self::Layer<'a>) -> Self::PerLayerParams {
        PerLineParams {
            colour: data.colour,
            thickness: data.thickness,
            _padding: [0., 0., 0.],
        }
    }
}
