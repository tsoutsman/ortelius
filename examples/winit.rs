use std::{thread, time::Duration};

use ortelius::{
    layer::{self, Layer, PointBuffer},
    layout::PlotLayout,
};
use rand_distr::{Distribution, StandardNormal};
use vello::wgpu;

const NUM_WALKS: usize = 10;
const NUM_STARTING_POINTS: usize = 100;
const NEW_POINT_PERIOD: Duration = Duration::from_millis(20);

// How many points to show in the plot.
const _XWINDOW_SIZE: usize = 200;

// Whether to limit the interaction bounds to the lines.
const _INTERACTION_BOUNDS: bool = true;
const _AUTO_SCROLL: bool = true;

// Mathematical Constants for the SDE
const MOMENTUM: f32 = 0.95; // Controls smoothness (mimics H > 0.5 in fBm). Closer to 1.0 = smoother.
const VOLATILITY: f32 = 0.011; // The magnitude of the random shocks.
const MEAN_REVERSION: f32 = 0.007; // The strength of the pull towards 0. Higher = tighter grouping.

pub const COLOURS: [[f32; 4]; 10] = [
    [0.90, 0.10, 0.10, 1.0], // 0: Crimson Red
    [0.10, 0.50, 0.90, 1.0], // 1: Dodger Blue
    [0.10, 0.80, 0.20, 1.0], // 2: Emerald Green
    [0.95, 0.70, 0.00, 1.0], // 3: Golden Yellow
    [0.60, 0.10, 0.80, 1.0], // 4: Deep Purple
    [0.00, 0.80, 0.80, 1.0], // 5: Cyan / Teal
    [0.95, 0.40, 0.00, 1.0], // 6: Vibrant Orange
    [0.90, 0.30, 0.60, 1.0], // 7: Hot Pink
    [0.60, 0.90, 0.10, 1.0], // 8: Lime Green
    [0.85, 0.85, 0.85, 1.0], // 9: Light Silver/Gray
];

fn main() {
    let mut walks_data = vec![];
    for _ in 0..NUM_WALKS {
        walks_data.push(generate_random_walk(NUM_STARTING_POINTS, 0.));
    }

    let mut current_state = walks_data
        .iter()
        .map(|(xs, ys, v)| (*xs.last().unwrap(), *ys.last().unwrap(), *v))
        .collect::<Vec<_>>();

    let walks_for_plot: Vec<(Vec<f32>, Vec<f32>)> =
        walks_data.into_iter().map(|(xs, ys, _)| (xs, ys)).collect();

    ortelius::plot(
        move |device, queue| State::new(walks_for_plot, device, queue),
        PlotLayout::new(),
        |channel| {
            thread::spawn(move || {
                loop {
                    thread::sleep(NEW_POINT_PERIOD);

                    // Update the state (x, y, v)
                    current_state = current_state
                        .into_iter()
                        .map(|(x, y, v)| {
                            let (next_y, next_v) = get_next_step(y, v);
                            (x + 1., next_y, next_v)
                        })
                        .collect();

                    // Map down to just (x, y) for the rendering event
                    let current_points: Vec<(f32, f32)> =
                        current_state.iter().map(|(x, y, _)| (*x, *y)).collect();

                    println!("sending event: {:?}", current_points);
                    channel.send_event(current_points).unwrap();
                }
            });
        },
    );
}

struct State {
    line_buffers: Vec<PointBuffer>,
}

impl State {
    fn new(walks: Vec<(Vec<f32>, Vec<f32>)>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut line_buffers = vec![];
        let mut command_buffers = vec![];

        for i in 0..NUM_WALKS {
            let (xs, ys) = walks[i].clone();
            let mut line_buffer = PointBuffer::new(device);
            let cb = line_buffer.extend(&xs, &ys, device);
            command_buffers.push(cb);
            line_buffers.push(line_buffer);
        }

        queue.submit(command_buffers);
        State { line_buffers }
    }
}

impl ortelius::State for State {
    type Event = Vec<(f32, f32)>;

    fn layers(&self) -> Vec<Layer<'_>> {
        vec![
            Layer::Grid(layer::Grid {
                spacing: 20.,
                thickness: 0.002,
                axis_thickness: 0.004,
            }),
            Layer::Lines(
                self.line_buffers
                    .iter()
                    .enumerate()
                    .map(|(i, buffer)| layer::Line {
                        data: buffer,
                        thickness: 0.004,
                        colour: COLOURS[i],
                    })
                    .collect(),
            ),
            Layer::Scatters(vec![layer::Scatter {
                data: self.line_buffers.first().unwrap(),
                radius: 0.01,
                colour: [0., 0., 0., 1.],
            }]),
        ]
    }

    fn update(&mut self, event: Self::Event, device: &wgpu::Device, queue: &wgpu::Queue) {
        assert_eq!(event.len(), self.line_buffers.len());

        let command_buffers: Vec<_> = self
            .line_buffers
            .iter_mut()
            .zip(event.iter())
            .map(|(buffer, &point)| buffer.append(point.0, point.1, device))
            .collect();
        queue.submit(command_buffers);
    }
}

/// Returns history of X, history of Y, and the final Velocity
fn generate_random_walk(num_steps: usize, initial_value: f32) -> (Vec<f32>, Vec<f32>, f32) {
    let mut xs = Vec::with_capacity(num_steps);
    let mut ys = Vec::with_capacity(num_steps);

    let mut current_y = initial_value;
    let mut current_v = 0.0;

    for i in 0..num_steps {
        xs.push(i as f32);
        ys.push(current_y);

        let (next_y, next_v) = get_next_step(current_y, current_v);
        current_y = next_y;
        current_v = next_v;
    }

    (xs, ys, current_v)
}

/// Applies an AR(1) process to velocity (smoothing) and an OU process to
/// position (mean reversion).
fn get_next_step(current_y: f32, current_v: f32) -> (f32, f32) {
    let mut rng = rand::rng();
    let dist = StandardNormal;
    let random_shock: f32 = dist.sample(&mut rng);

    // 1. Correlate the noise via momentum to achieve fractional-brownian-like
    //    smoothness
    let new_v = (MOMENTUM * current_v) + (VOLATILITY * random_shock);

    // 2. Update position and pull it slightly towards 0 to prevent infinite
    //    divergence
    let new_y = current_y + new_v - (MEAN_REVERSION * current_y);

    (new_y, new_v)
}
