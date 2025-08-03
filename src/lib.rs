mod buffer;
mod layer;
mod layout;
mod winit;

pub use crate::{
    buffer::GpuBuffer,
    layer::Layer,
    layout::{Bounds, Padding, PlotLayout},
};

pub enum NewData {
    Point { x: f32, y: f32 },
    Points { xs: Vec<f32>, ys: Vec<f32> },
}

pub fn plot(layout: PlotLayout, xs: Vec<f32>, ys: Vec<f32>) {
    winit::App::new(layout, xs, ys).display();
}
