mod line;

use vello::wgpu::{self, BufferUsages, CommandBuffer, Device};

use crate::GpuBuffer;

pub enum Layer<'a> {
    Title(&'a str),
    XAxis { label: Option<&'a str> },
    YAxis { label: Option<&'a str> },
    Lines(Vec<Line>),
    Scatter,
}

pub struct Line {
    pub(crate) thickness: f32,
    pub(crate) colour: f32,
}

impl Line {
    pub fn new(device: &wgpu::Device, xs: &[f32], ys: &[f32]) -> Self {
        assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC;

        Line {
            buffer: GpuBuffer::new(device, usage, 2 * xs.len(), |buffer| {
                for i in 0..xs.len() {
                    buffer[i * 2] = xs[i];
                    buffer[i * 2 + 1] = ys[i];
                }
            }),
            thickness: 0.005,
            colour: 0.,
        }
    }

    #[must_use]
    pub fn append(&mut self, device: &wgpu::Device, x: f32, y: f32) -> CommandBuffer {
        self.buffer.extend(device, 2, |buffer| {
            buffer[0] = x;
            buffer[1] = y;
        })
    }

    #[must_use]
    pub fn extend(&mut self, device: &wgpu::Device, xs: &[f32], ys: &[f32]) -> CommandBuffer {
        assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");

        self.buffer.extend(device, 2 * xs.len(), |buffer| {
            for i in 0..xs.len() {
                buffer[i * 2] = xs[i];
                buffer[i * 2 + 1] = ys[i];
            }
        })
    }
}
