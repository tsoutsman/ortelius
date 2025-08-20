use ortelius::{LayerSpecification, LineSpecification};
use rand::thread_rng;
use rand_distr::{Distribution, StandardNormal};

fn main() {
    const NUM_STEPS: usize = 550;
    const INITIAL_VALUE: f32 = 0.0;

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

    let layers = vec![LayerSpecification::Lines(vec![LineSpecification {
        xs,
        ys,
    }])];

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
        layers,
        |_| {},
    );
}
