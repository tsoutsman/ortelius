use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use crate::layer::Scatter;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(super) struct PerScatterParams {
    colour: [f32; 4],
    radius: f32,
    _padding: [f32; 3],
}

pub(super) struct Renderer {}

impl super::LayerRenderer for Renderer {
    type Layer<'a> = Scatter<'a>;
    type PerLayerParams = PerScatterParams;

    const NAME: &'static str = "scatter";

    fn new() -> Self {
        Self {}
    }

    fn shader(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("render.wgsl")
    }

    fn counts(&self, data: &Self::Layer<'_>) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
        (0..6, 0..data.data.len() as u32)
    }

    fn create_per_layer_params<'a>(&self, data: &Self::Layer<'a>) -> Self::PerLayerParams {
        PerScatterParams {
            colour: data.colour,
            radius: data.radius,
            _padding: [0., 0., 0.],
        }
    }
}
