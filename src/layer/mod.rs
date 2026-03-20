mod point_buffer;

pub use point_buffer::PointBuffer;
use vello::wgpu;

#[derive(Debug, Clone)]
pub enum Layer<'a> {
    Title(&'a str),
    XAxis { label: Option<&'a str> },
    YAxis { label: Option<&'a str> },
    Grid(Grid),
    Lines(Vec<Line<'a>>),
    Scatters(Vec<Scatter<'a>>),
}

#[derive(Debug, Clone, Copy)]
pub struct Line<'a> {
    pub data: &'a PointBuffer,
    pub thickness: f32,
    pub colour: [f32; 4],
}

impl crate::gpu::Layer for Line<'_> {
    const HAS_DATA: bool = true;

    fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.data.as_entire_binding()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scatter<'a> {
    pub data: &'a PointBuffer,
    pub radius: f32,
    pub colour: [f32; 4],
}

impl crate::gpu::Layer for Scatter<'_> {
    const HAS_DATA: bool = true;

    fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.data.as_entire_binding()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Grid {
    pub spacing: f32,
    pub thickness: f32,
    pub axis_thickness: f32,
}

impl crate::gpu::Layer for Grid {
    const HAS_DATA: bool = false;

    fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        panic!("grid layer does not have a buffer")
    }
}
