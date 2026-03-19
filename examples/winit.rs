use std::{thread, time::Duration};

use ortelius::{
    layer::{self, Layer, LineBuffer},
    layout::PlotLayout,
};
use rand_distr::{Distribution, StandardNormal};
use vello::wgpu;

const NUM_WALKS: usize = 3;
const NUM_STARTING_POINTS: usize = 1000;
const NEW_POINT_PERIOD: Duration = Duration::from_millis(20);

// How many points to show in the plot.
const _XWINDOW_SIZE: usize = 200;

// Whether to limit the interaction bounds to the lines.
const _INTERACTION_BOUNDS: bool = true;
const _AUTO_SCROLL: bool = true;

fn main() {
    let mut walks = vec![];
    for _ in 0..NUM_WALKS {
        walks.push(generate_random_walk(NUM_STARTING_POINTS, 0.));
    }
    let mut current = walks
        .iter()
        .map(|walk| (*walk.0.last().unwrap(), *walk.1.last().unwrap()))
        .collect::<Vec<_>>();

    ortelius::plot(
        move |device, queue| State::new(walks, device, queue),
        PlotLayout::new(),
        |channel| {
            thread::spawn(move || {
                loop {
                    thread::sleep(NEW_POINT_PERIOD);
                    current = current
                        .into_iter()
                        .map(|(x, y)| (x + 1., get_next_step(y)))
                        .collect();
                    channel.send_event(current.clone()).unwrap();
                }
            });
        },
    );
}

struct State {
    line_buffers: Vec<LineBuffer>,
}

impl State {
    fn new(walks: Vec<(Vec<f32>, Vec<f32>)>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut line_buffers = vec![];
        let mut command_buffers = vec![];

        for i in 0..NUM_WALKS {
            let (xs, ys) = walks[i].clone();
            let mut line_buffer = LineBuffer::new(device);
            let cb = line_buffer.extend(&xs, &ys, device);
            command_buffers.push(cb);
            line_buffers.push(line_buffer);
        }

        queue.submit(command_buffers);

        State { line_buffers }
    }
}

impl ortelius::State for State {
    // Each event contains a new point for each line buffer.
    type Event = Vec<(f32, f32)>;

    fn layers(&self) -> Vec<Layer> {
        vec![Layer::Lines(
            self.line_buffers
                .iter()
                .map(|buffer| layer::Line {
                    data: buffer,
                    thickness: 2.,
                    colour: 1.,
                })
                .collect(),
        )]
    }

    fn update(&mut self, event: Self::Event, device: &wgpu::Device, queue: &wgpu::Queue) {
        assert_eq!(event.len(), self.line_buffers.len(),);

        let command_buffers: Vec<_> = self
            .line_buffers
            .iter_mut()
            .zip(event.iter())
            .map(|(buffer, &point)| buffer.append(point.0, point.1, device))
            .collect();
        queue.submit(command_buffers);
    }
}

fn generate_random_walk(num_steps: usize, initial_value: f32) -> (Vec<f32>, Vec<f32>) {
    let mut xs = vec![];
    let mut ys = vec![];

    let mut current_value = initial_value;

    for i in 0..num_steps {
        xs.push(i as f32);
        ys.push(current_value);
        current_value = get_next_step(current_value);
    }

    (xs, ys)
}

fn get_next_step(current_value: f32) -> f32 {
    let mut rng = rand::rng();
    let dist = StandardNormal;
    let random_step: f32 = dist.sample(&mut rng);
    current_value + random_step * 0.02
}
