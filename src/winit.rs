use std::{num::NonZeroUsize, sync::Arc};

use vello::{
    AaConfig, Renderer, RendererOptions,
    kurbo::{Affine, Point},
    peniko::{Color, Fill},
    util::{RenderContext, RenderSurface},
    wgpu,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowAttributes},
};

use crate::Plot;

pub(crate) struct OrteliusApp<'p, 's> {
    plot: &'p mut Plot,
    is_plot_outdated: bool,

    context: RenderContext,
    render_state: Option<RenderState<'s>>,
    renderers: Vec<Option<Renderer>>,

    keyboard_modifiers: ModifiersState,
    is_mouse_down: bool,

    i: u64,

    prior_position: Option<Point>,
}

impl<'p, 's> OrteliusApp<'p, 's> {
    pub(crate) fn new(plot: &'p mut Plot) -> Self {
        Self {
            context: RenderContext::new(),
            plot,
            is_plot_outdated: true,
            render_state: None,
            renderers: Vec::new(),
            keyboard_modifiers: ModifiersState::default(),
            is_mouse_down: false,
            i: 0,
            prior_position: None,
        }
    }

    // TODO: somehow allow updating plot

    /// this blocks
    pub(crate) fn display(&mut self) {
        EventLoop::new().unwrap().run_app(self).unwrap();
    }
}

impl ApplicationHandler for OrteliusApp<'_, '_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.render_state.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(LogicalSize::new(
                            self.plot.width as u32,
                            self.plot.height as u32,
                        ))
                        .with_resizable(true)
                        .with_title("Ortelius"),
                )
                .unwrap(),
        );
        let size = window.inner_size();
        let present_mode = vello::wgpu::PresentMode::AutoVsync;
        //let present_mode = vello::wgpu::PresentMode::AutoNoVsync;
        let surface_future =
            self.context
                .create_surface(window.clone(), size.width, size.height, present_mode);
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        let render_state = RenderState { surface, window };

        self.renderers
            .resize_with(self.context.devices.len(), || None);

        let id = render_state.surface.dev_id;
        self.renderers[id].get_or_insert_with(|| {
            let device_handle = &self.context.devices[id];

            let renderer = Renderer::new(
                &device_handle.device,
                RendererOptions {
                    use_cpu: false,
                    antialiasing_support: [AaConfig::Area].iter().copied().collect(),
                    num_init_threads: NonZeroUsize::new(1),
                    pipeline_cache: None,
                },
            )
            .unwrap();
            renderer
        });

        self.render_state = Some(render_state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(render_state) = &mut self.render_state else {
            return;
        };
        if render_state.window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(m) => self.keyboard_modifiers = m.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key.as_ref() {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(RenderState { surface, window }) = &mut self.render_state {
                    self.context
                        .resize_surface(surface, size.width, size.height);

                    self.plot.width = (size.width as usize) / 2;
                    self.plot.height = (size.height as usize) / 2;

                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.is_mouse_down = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.prior_position = None;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(RenderState { window, .. }) = &mut self.render_state {
                    let factor = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => 1.0 + y as f64 / 10.0,
                        winit::event::MouseScrollDelta::PixelDelta(delta) => {
                            1.0 + delta.y as f64 / 500.0
                        }
                    };

                    if let Some(prior) = self.prior_position {
                        self.plot.zoom(prior.into(), factor);
                        self.is_plot_outdated = true;
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = Point {
                    x: position.x,
                    y: position.y,
                };

                if self.is_mouse_down {
                    if let Some(prior) = self.prior_position {
                        self.plot
                            .move_bounds(prior.x - position.x, position.y - prior.y);
                    }
                    self.is_plot_outdated = true;

                    if let Some(RenderState { window, .. }) = &mut self.render_state {
                        window.request_redraw();
                    }
                }

                self.prior_position = Some(position);
            }
            WindowEvent::RedrawRequested => {
                // TODO
                //render_state.window.request_redraw();
                self.i += 1;

                let Some(RenderState { surface, window }) = &self.render_state else {
                    return;
                };
                let width = surface.config.width;
                let height = surface.config.height;

                // println!("redrawing: {}x{}", width, height);
                // println!("scaling factor: {:?}", window.scale_factor());

                let device_handle = &self.context.devices[surface.dev_id];

                let render_params = vello::RenderParams {
                    base_color: Color::WHITE,
                    width,
                    height,
                    antialiasing_method: vello::AaConfig::Area,
                };

                //println!("rendering: {:?}", self.i);

                if self.is_plot_outdated {
                    self.plot.redraw(window.scale_factor());
                    self.is_plot_outdated = false;
                }

                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.plot.scene,
                        &surface.target_view,
                        &render_params,
                    )
                    .expect("failed to render to texture");

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
                device_handle.queue.submit([encoder.finish()]);
                surface_texture.present();

                device_handle.device.poll(wgpu::PollType::Poll).unwrap();
            }
            _ => {}
        }
    }
}

struct RenderState<'s> {
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}
