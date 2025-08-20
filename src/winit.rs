use std::{mem, sync::Arc};

use vello::{kurbo::Point, util::RenderContext, wgpu};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

use crate::{
    layer::{Layer, LayerSpecification},
    layout::{PlotInstanceLayout, PlotLayout},
    render::Renderer,
};

#[allow(clippy::large_enum_variant)]
pub(crate) enum App<'s> {
    Uninitialized {
        layout: PlotLayout,
        layers: Vec<LayerSpecification>,
    },
    Initialized {
        layout: PlotInstanceLayout,
        layers: Vec<Layer>,

        window: Arc<Window>,
        input: Input,
        renderer: Renderer<'s>,
    },
}

pub struct Update {
    layer: usize,
    line: Option<usize>,
    kind: UpdateKind,
}

pub enum UpdateKind {
    Append { x: f32, y: f32 },
    Extend { xs: Vec<f32>, ys: Vec<f32> },
    Set { xs: Vec<f32>, ys: Vec<f32> },
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Input {
    keyboard_modifiers: ModifiersState,
    is_mouse_down: Option<Point>,
    prior_position: Option<Point>,
}

impl<'s> App<'s> {
    pub(crate) fn new(layout: PlotLayout, layers: Vec<LayerSpecification>) -> Self {
        Self::Uninitialized { layout, layers }
    }

    pub(crate) fn display<F>(&mut self, f: F)
    where
        F: FnOnce(crate::Channel),
    {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        f(event_loop.create_proxy());
        event_loop.run_app(self).unwrap();
    }
}

impl ApplicationHandler<Update> for App<'_> {
    fn user_event(&mut self, _: &ActiveEventLoop, update: Update) {
        match self {
            App::Uninitialized { .. } => todo!(),
            App::Initialized {
                layers,
                window,
                renderer,
                ..
            } => {
                let layer = layers.get_mut(update.layer).unwrap();
                if let Some(line) = update.line {
                    if let Layer::Lines(lines) = layer {
                        let line = lines.get_mut(line).unwrap();
                        let device = renderer.device();

                        let command_buffer = match update.kind {
                            UpdateKind::Append { x, y } => line.append(device, x, y),
                            UpdateKind::Extend { xs, ys } => line.extend(device, &xs, &ys),
                            UpdateKind::Set { .. } => todo!(),
                        };

                        renderer.queue().submit([command_buffer]);
                        // TODO: do we have to do this?
                        device.poll(wgpu::PollType::Poll).unwrap();
                    } else {
                        panic!("specified line index for non-line layer");
                    }
                } else {
                    todo!();
                }

                // TODO: only redraw if new data in viewport.
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            App::Uninitialized { layout, layers } => {
                let window = Arc::new(
                    event_loop
                        .create_window(
                            Window::default_attributes()
                                .with_inner_size(LogicalSize::new(
                                    layout.width as u32,
                                    layout.height as u32,
                                ))
                                .with_resizable(true)
                                .with_title("Ortelius"),
                        )
                        .unwrap(),
                );
                let size = window.inner_size();
                let present_mode = vello::wgpu::PresentMode::AutoVsync;

                let mut context = RenderContext::new();

                let surface_future =
                    context.create_surface(window.clone(), size.width, size.height, present_mode);
                let surface = pollster::block_on(surface_future).expect("Error creating surface");

                // TODO
                let initial_data_bounds = None;
                // TODO: don't clone
                let layout = layout.clone().instantiate(&window, initial_data_bounds);

                window.request_redraw();

                let mut layers_data = Vec::new();
                mem::swap(&mut layers_data, layers);

                let device = &context.devices[surface.dev_id].device;
                let queue = &context.devices[surface.dev_id].queue;
                *self = App::Initialized {
                    window,
                    input: Input::default(),
                    layout,
                    // TODO: don't clone
                    renderer: Renderer::new(device.clone(), queue.clone(), surface.surface),
                    layers: layers_data
                        .into_iter()
                        .map(|spec| spec.init(device))
                        .collect(),
                };
            }
            App::Initialized { .. } => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match self {
            App::Uninitialized { .. } => {}
            App::Initialized {
                window,
                input,
                layout,
                renderer,
                layers,
            } => {
                if window.id() != window_id {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::ModifiersChanged(m) => input.keyboard_modifiers = m.state(),
                    WindowEvent::KeyboardInput { event, .. }
                        if event.state == ElementState::Pressed =>
                    {
                        #[allow(clippy::single_match)]
                        match event.logical_key.as_ref() {
                            Key::Named(NamedKey::Escape) => event_loop.exit(),
                            _ => {}
                        }
                    }
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                        layout.resize(size.width, size.height);
                        window.request_redraw();
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            input.is_mouse_down = if state == ElementState::Pressed {
                                input.prior_position
                            } else {
                                None
                            };
                        }
                    }
                    WindowEvent::CursorLeft { .. } => {
                        input.prior_position = None;
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let factor = match delta {
                            winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                1.0 + y as f64 / 10.0
                            }
                            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                1.0 + delta.y / 500.0
                            }
                        };

                        if let Some(prior) = input.prior_position {
                            layout.zoom(prior.into(), factor);
                            window.request_redraw();
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let position = Point {
                            x: position.x,
                            y: position.y,
                        };

                        if let Some(start_drag_mouse_position) = input.is_mouse_down
                            && let Some(prior) = input.prior_position
                        {
                            layout.drag(
                                start_drag_mouse_position.into(),
                                prior.into(),
                                position.into(),
                            );
                            window.request_redraw();
                        }

                        input.prior_position = Some(position);
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.render(layers.iter(), layout);
                    }
                    _ => {}
                }
            }
        }
    }
}
