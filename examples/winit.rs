use rand::thread_rng;
use rand_distr::{Distribution, StandardNormal};
use vello::{Scene, peniko::color::AlphaColor};

fn main() {
    const NUM_STEPS: usize = 50_000;
    const INITIAL_VALUE: f64 = 0.0;

    // 2. Set up the random number generator and the distribution.
    // `thread_rng` is a fast, cryptographically secure RNG provided by the OS.
    // `StandardNormal` is the standard normal distribution (mean=0, stddev=1).
    let mut rng = thread_rng();
    let dist = StandardNormal;

    let mut xs = vec![];
    let mut ys = vec![];
    let mut ys2 = vec![];

    let mut current_value = INITIAL_VALUE;
    let mut current_value2 = INITIAL_VALUE;

    for i in 0..NUM_STEPS {
        xs.push(i as f64);

        ys.push(current_value);
        let random_step: f64 = dist.sample(&mut rng);
        current_value += random_step;

        ys2.push(current_value2);
        let random_step: f64 = dist.sample(&mut rng);
        current_value2 += random_step;
    }

    println!("done");

    let plot = ortelius::Plot {
        bounds: ortelius::Bounds {
            //x: (-1./50. * NUM_STEPS as f64, NUM_STEPS as f64 + 1./50. * NUM_STEPS as f64),
            x: (0., 400.),
            y: (-100., 100.),
            // x: (0., 1.),
            // y: (0., 1.)
        },
        padding_layers: vec![
            ortelius::PaddingLayer::XAxis {},
            ortelius::PaddingLayer::YAxis {},
        ],
        data_layers: vec![
            ortelius::DataLayer::Line {
                xs: xs.clone(),
                ys,
                color: AlphaColor::new([0.0, 0.0, 1.0, 1.0]),
                width: 3.0,
            },
            ortelius::DataLayer::Line {
                xs,
                ys: ys2,
                color: AlphaColor::new([1.0, 0.0, 1.0, 1.0]),
                width: 3.0,
            },
        ],
        width: 700,
        height: 400,
        scene: Scene::new(),
        padding: ortelius::Padding {
            left: 48,
            right: 0,
            top: 0,
            bottom: 48,
        },
    };

    ortelius::launch(plot);
}
