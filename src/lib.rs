mod buffer;
mod layer;
mod layout;
mod render;
mod winit;

use crate::buffer::GpuBuffer;
pub use crate::{
    layer::{Layer, LayerSpecification, LineSpecification},
    layout::{Bounds, Padding, PlotLayout},
    winit::{Update, UpdateKind},
};

type Channel = ::winit::event_loop::EventLoopProxy<Update>;

trait State {
    type Event;

    fn update(&mut self, event: Self::Event);

    fn plot(&self) -> PlotLayout;
}

pub fn plot<F>(layout: PlotLayout, layers: Vec<LayerSpecification>, f: F)
where
    F: FnOnce(Channel),
{
    winit::App::new(layout, layers).display(f);
}
