use std::sync::Arc;

use vello::{
    kurbo::Point,
    util::{RenderContext, RenderSurface},
    wgpu,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

use crate::{
    layer::Line,
    layout::{PlotInstanceLayout, PlotLayout},
};

#[allow(clippy::large_enum_variant)]
pub(crate) enum App<'s> {
    Uninitialized {
        xs: Vec<f32>,
        ys: Vec<f32>,
        layout: PlotLayout,
    },
    Initialized {
        surface: RenderSurface<'s>,
        window: Arc<Window>,
        input: Input,
        context: RenderContext,
        layout: PlotInstanceLayout,
        line: Line,

        line_renderer: crate::layer::LineRenderer,
        msaa_view: wgpu::TextureView,
    },
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Multisample Framebuffer"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Input {
    keyboard_modifiers: ModifiersState,
    is_mouse_down: Option<Point>,
    prior_position: Option<Point>,
}

impl<'s> App<'s> {
    pub(crate) fn new(layout: PlotLayout, xs: Vec<f32>, ys: Vec<f32>) -> Self {
        Self::Uninitialized { layout, xs, ys }
    }

    pub(crate) fn display(&mut self) {
        EventLoop::new().unwrap().run_app(self).unwrap();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            App::Uninitialized { xs, ys, layout } => {
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

                let line = Line::new(&context.devices[surface.dev_id].device, xs, ys);
                *self = App::Initialized {
                    window,
                    msaa_view: create_multisampled_framebuffer(
                        &context.devices[surface.dev_id].device,
                        &surface.config,
                    ),
                    line_renderer: crate::layer::LineRenderer::create(
                        &context.devices[surface.dev_id].device,
                    ),
                    surface,
                    input: Input::default(),
                    context,
                    layout,
                    line,
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
                surface,
                window,
                input,
                context,
                layout,
                msaa_view,
                line,
                line_renderer,
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
                        context.resize_surface(surface, size.width, size.height);
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
                        let handle = &context.devices[surface.dev_id];

                        let mut encoder =
                            handle
                                .device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("Line Render Encoder"),
                                });

                        let output = surface.surface.get_current_texture().unwrap();
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        line_renderer.render(
                            &handle.device,
                            &mut encoder,
                            &view,
                            &msaa_view,
                            [&*line].into_iter(),
                        );

                        handle.queue.submit([encoder.finish()]);
                        output.present();

                        handle.device.poll(wgpu::PollType::Poll).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}
