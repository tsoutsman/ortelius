use std::{marker::PhantomData, mem, sync::Arc};

pub use ::winit::event_loop::EventLoopProxy as Channel;
use vello::{
    kurbo::Point,
    util::RenderContext,
    wgpu::{Device, Queue},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

use crate::{PlotInstanceLayout, PlotLayout, State, gpu::Renderer};

#[allow(clippy::large_enum_variant)]
pub(crate) enum App<'s, S>
where
    S: State,
{
    Uninitialized {
        // TODO: Explain why we need option.
        state_constructor: Option<Box<dyn FnOnce(&Device, &Queue) -> S>>,
        layout: PlotLayout,
    },
    Initialized {
        state: S,
        window: Arc<Window>,
        input: Input,
        renderer: Renderer<'s>,
        layout: PlotInstanceLayout,
        _phantom: PhantomData<S>,
    },
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Input {
    keyboard_modifiers: ModifiersState,
    is_mouse_down: Option<Point>,
    prior_position: Option<Point>,
}

impl<'s, S> App<'s, S>
where
    S: State,
{
    pub(crate) fn new<F>(state_constructor: F, layout: PlotLayout) -> Self
    where
        F: FnOnce(&Device, &Queue) -> S + 'static,
    {
        Self::Uninitialized {
            state_constructor: Some(Box::new(state_constructor)),
            layout,
        }
    }

    pub(crate) fn plot<F>(&mut self, f: F)
    where
        F: FnOnce(Channel<S::Event>),
    {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        f(event_loop.create_proxy());
        event_loop.run_app(self).unwrap();
    }
}

impl<S> ApplicationHandler<S::Event> for App<'_, S>
where
    S: State,
{
    fn user_event(&mut self, _: &ActiveEventLoop, update: S::Event) {
        match self {
            App::Uninitialized { .. } => todo!(),
            App::Initialized {
                state,
                window,
                renderer,
                ..
            } => {
                state.update(update, renderer.device(), renderer.queue());
                // TODO: only redraw if new data in viewport.
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            App::Uninitialized {
                state_constructor,
                layout,
            } => {
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

                window.request_redraw();

                let device = &context.devices[surface.dev_id].device;
                let queue = &context.devices[surface.dev_id].queue;

                let mut new_state_constructor = None;
                mem::swap(&mut new_state_constructor, state_constructor);

                let mut new_layout = PlotLayout::new();
                mem::swap(&mut new_layout, layout);

                *self = App::Initialized {
                    state: new_state_constructor.unwrap()(device, queue),
                    layout: new_layout.instantiate(&window),
                    window,
                    input: Input::default(),
                    // TODO: don't clone
                    renderer: Renderer::new(device.clone(), queue.clone(), surface.surface),
                    _phantom: PhantomData,
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
                state,
                layout,
                window,
                input,
                renderer,
                _phantom,
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
                                // TODO
                                1.0 + y as f64 / 10.0
                            }
                            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                // TODO
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
                        renderer.render(state.layers(layout).into_iter(), layout);
                    }
                    _ => {}
                }
            }
        }
    }
}
