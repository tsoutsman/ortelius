use rand::thread_rng;
use rand_distr::{Distribution, StandardNormal};
use vello::{Scene, peniko::color::AlphaColor};

fn main() {
    const NUM_STEPS: usize = 550;
    const INITIAL_VALUE: f32 = 0.0;

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
        xs.push(i as f32 / 200. - 1.1);
        ys.push(current_value);

        let random_step: f32 = dist.sample(&mut rng);
        current_value += random_step * 0.02;

        ys2.push(current_value2);
        let random_step: f32 = dist.sample(&mut rng);
        current_value2 += random_step * 0.02;
    }

    println!("done");

    ortelius::plot(
        ortelius::PlotLayout::new()
            .with_width(800.0)
            .with_height(600.0)
            .with_padding(ortelius::Padding {
                top: 20.0,
                bottom: 20.0,
                left: 50.0,
                right: 20.0,
            }),
        xs,
        ys,
    );
}
