mod gpu;
pub mod layer;
pub mod layout;
mod winit;

use vello::wgpu;

pub use crate::winit::Channel;
use crate::{
    layer::Layer,
    layout::{PlotInstanceLayout, PlotLayout},
};

pub trait State {
    type Event: 'static;

    fn layers(&self) -> Vec<Layer<'_>>;

    fn update(&mut self, event: Self::Event, device: &wgpu::Device, queue: &wgpu::Queue) {
        let _ = (event, device, queue);
    }
}

pub fn plot<S, F, G>(state_constructor: F, layout: PlotLayout, channel_storer: G)
where
    S: State,
    F: FnOnce(&wgpu::Device, &wgpu::Queue) -> S + 'static,
    G: FnOnce(Channel<S::Event>),
{
    winit::App::new(state_constructor, layout).plot(channel_storer);
}

pub fn save_png(_layers: Vec<Layer>, _layout: PlotLayout, _path: &str) -> std::io::Result<()> {
    todo!();
}
