use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use crate::layer::Grid;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(super) struct Params {
    spacing: [f32; 2],
    thickness: f32,
    axis_thickness: f32,
}

pub(super) struct Renderer {}

impl super::LayerRenderer for Renderer {
    type Layer<'a> = Grid;
    type PerLayerParams = Params;

    const NAME: &'static str = "grid";

    fn new() -> Self {
        Self {}
    }

    fn shader(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("render.wgsl")
    }

    fn counts(&self, _: &Self::Layer<'_>) -> (std::ops::Range<u32>, std::ops::Range<u32>) {
        (0..6, 0..1)
    }

    fn create_per_layer_params<'a>(&self, data: &Self::Layer<'a>) -> Self::PerLayerParams {
        Params {
            spacing: data.spacing,
            thickness: data.thickness,
            axis_thickness: data.axis_thickness,
        }
    }
}
