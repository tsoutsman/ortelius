mod line;

use vello::wgpu::{self, CommandBuffer};

use crate::gpu::GpuBuffer;

#[derive(Debug, Clone)]
pub enum Layer<'a> {
    Title(&'a str),
    XAxis { label: Option<&'a str> },
    YAxis { label: Option<&'a str> },
    Lines(Vec<Line<'a>>),
    Scatter(Scatter<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct Line<'a> {
    pub data: &'a PointBuffer,
    pub thickness: f32,
    pub colour: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct Scatter<'a> {
    pub data: &'a PointBuffer,
    pub radius: f32,
}

#[derive(Debug)]
pub struct PointBuffer {
    inner: GpuBuffer<f32>,
}

impl PointBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            inner: GpuBuffer::new(
                device,
                // TODO
                wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                0,
                |_| {},
            ),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len() / 2
    }

    #[must_use]
    pub fn append(&mut self, x: f32, y: f32, device: &wgpu::Device) -> CommandBuffer {
        self.extend(&[x], &[y], device)
    }

    #[must_use]
    pub fn extend(&mut self, xs: &[f32], ys: &[f32], device: &wgpu::Device) -> CommandBuffer {
        assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");

        let len = xs.len();
        self.inner.extend(device, len * 2, |buffer| {
            for i in 0..len {
                buffer[i * 2] = xs[i];
                buffer[i * 2 + 1] = ys[i];
            }
        })
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource<'_> {
        self.inner.as_entire_binding()
    }
}
