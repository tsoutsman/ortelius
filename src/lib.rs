mod winit;

use itertools::zip_eq;
use vello::{
    Scene,
    kurbo::{self, Affine, Cap, Join, PathEl},
    peniko::{
        self, Mix,
        color::{AlphaColor, Srgb},
    },
};

pub struct Plot {
    pub padding: Padding,
    pub bounds: Bounds,
    pub padding_layers: Vec<PaddingLayer>,
    pub data_layers: Vec<DataLayer>,
    pub width: usize,
    pub height: usize,
    pub scene: Scene,
}

pub fn launch(mut plot: Plot) {
    let mut app = winit::OrteliusApp::new(&mut plot);
    app.display();
}

impl Plot {
    fn move_bounds(&mut self, x: f64, y: f64) {
        let real_x = self.xscale() * x / 2.;
        let real_y = self.yscale() * y / 2.;

        let (bounds_x0, bounds_x1) = self.bounds.x;
        let (bounds_y0, bounds_y1) = self.bounds.y;

        self.bounds.x = (bounds_x0 + real_x, bounds_x1 + real_x);
        self.bounds.y = (bounds_y0 + real_y, bounds_y1 + real_y);
    }

    fn plot_width(&self) -> f64 {
        self.width as f64 - self.padding.left as f64 - self.padding.right as f64
    }

    fn plot_height(&self) -> f64 {
        self.height as f64 - self.padding.top as f64 - self.padding.bottom as f64
    }

    fn xscale(&self) -> f64 {
        let (bounds_x0, bounds_x1) = self.bounds.x;
        (bounds_x1 - bounds_x0) / self.plot_width()
    }

    fn yscale(&self) -> f64 {
        let (bounds_y0, bounds_y1) = self.bounds.y;
        (bounds_y1 - bounds_y0) / self.plot_height()
    }

    fn transform_x(&self, x: f64) -> f64 {
        (x - self.bounds.x.0) / self.xscale() + self.padding.left as f64
    }

    fn transform_y(&self, y: f64) -> f64 {
        // TODO: top or bottom
        self.height as f64 - self.padding.bottom as f64 - (y - self.bounds.y.0) / self.yscale()
    }

    pub fn zoom(&mut self, position: (f64, f64), factor: f64) {
        let (x, y) = position;

        let (bounds_x0, bounds_x1) = self.bounds.x;
        let (bounds_y0, bounds_y1) = self.bounds.y;

        let real_x = x / 2. - self.padding.left as f64;
        let real_y = self.height as f64 - self.padding.bottom as f64 - y / 2.;

        let percentage_x = real_x / self.plot_width();
        let percentage_y = real_y / self.plot_height();

        if percentage_x < 0. || percentage_y < 0. {
            return;
        }

        let data_x = bounds_x0 + percentage_x * (bounds_x1 - bounds_x0);
        let data_y = bounds_y0 + percentage_y * (bounds_y1 - bounds_y0);

        let mut new_bounds_x0 = data_x - (data_x - bounds_x0) * factor;
        let mut new_bounds_x1 = data_x + (bounds_x1 - data_x) * factor;
        let mut new_bounds_y0 = data_y - (data_y - bounds_y0) * factor;
        let mut new_bounds_y1 = data_y + (bounds_y1 - data_y) * factor;

        let new_range_x = new_bounds_x1 - new_bounds_x0;

        let x_min = 0.;
        let x_max = 1000.;
        let y_min = -100.;
        let y_max = 100.;

        if new_range_x > x_max - x_min {
            new_bounds_x0 = x_min;
            new_bounds_x1 = x_max;
        } else {
            if new_bounds_x0 < x_min {
                let shift = x_min - new_bounds_x0;
                new_bounds_x0 = x_min;
                new_bounds_x1 += shift;
            } else if new_bounds_x1 > x_max {
                let shift = new_bounds_x1 - x_max;
                new_bounds_x1 = x_max;
                new_bounds_x0 -= shift;
            }
        }

        let new_range_y = new_bounds_y1 - new_bounds_y0;
        if new_range_y > y_max - y_min {
            new_bounds_y0 = y_min;
            new_bounds_y1 = y_max;
        } else {
            if new_bounds_y0 < y_min {
                let shift = 0. - new_bounds_y0;
                new_bounds_y0 = 0.;
                new_bounds_y1 += shift;
            } else if new_bounds_y1 > y_max {
                let shift = new_bounds_y1 - y_max;
                new_bounds_y1 = y_max;
                new_bounds_y0 -= shift;
            }
        }

        self.bounds.x = (new_bounds_x0, new_bounds_x1);
        self.bounds.y = (new_bounds_y0, new_bounds_y1);
    }

    pub fn redraw(&mut self, scale_factor: f64) {
        // vello moment.
        assert!(scale_factor == 1.0 || scale_factor == 2.0);

        assert!(self.padding.left * scale_factor as u16 % 16 == 0);
        assert!(self.padding.right * scale_factor as u16 % 16 == 0);
        assert!(self.padding.top * scale_factor as u16 % 16 == 0);
        assert!(self.padding.bottom * scale_factor as u16 % 16 == 0);

        // TODO: don't redraw text layers
        // self.scene.reset();
        let mut scene = Scene::new();


        let rect = kurbo::Rect {
            x0: scale_factor * self.padding.left as f64,
            y0: scale_factor * self.padding.top as f64,
            // TODO y1 needs to be fixed
            x1: scale_factor * (self.width as f64 - self.padding.right as f64) + 16.,
            y1: scale_factor * (self.height as f64 - self.padding.bottom as f64),
        };
        scene.push_layer(Mix::Clip, 1.0, Affine::IDENTITY, &rect);

        for layer in &self.data_layers {
            match layer {
                DataLayer::Bar { xs, ys, colors } => {
                    self.draw_bar(
                        xs.iter().cloned(),
                        ys.iter().cloned(),
                        colors.iter().cloned(),
                        &mut scene,
                        scale_factor,
                    );
                }
                DataLayer::Scatter {
                    xs,
                    ys,
                    color,
                    size,
                } => {
                    self.draw_scatter(
                        xs.iter().cloned(),
                        ys.iter().cloned(),
                        color.iter().cloned(),
                        size.iter().cloned(),
                        &mut scene,
                        scale_factor,
                    );
                }
                DataLayer::Line {
                    xs,
                    ys,
                    color,
                    width,
                } => {
                    self.draw_line(
                        xs.iter().cloned(),
                        ys.iter().cloned(),
                        *color,
                        *width,
                        &mut scene,
                        scale_factor,
                    );
                }
                _ => panic!(),
            }
        }

        scene.pop_layer();

        for layer in &self.padding_layers {
            match layer {
                PaddingLayer::XAxis => {
                    let path = [
                        PathEl::MoveTo(
                            (
                                2. * self.padding.left as f64,
                                2. * (self.height as f64 - self.padding.bottom as f64),
                            )
                                .into(),
                        ),
                        PathEl::LineTo(
                            (
                                2. * (self.width as f64 - self.padding.right as f64),
                                2. * (self.height as f64 - self.padding.bottom as f64),
                            )
                                .into(),
                        ),
                    ];
                    let style = kurbo::Stroke::new(3.).with_caps(Cap::Square);

                    scene.stroke(
                        &style,
                        kurbo::Affine::IDENTITY,
                        peniko::BrushRef::Solid(AlphaColor::BLACK),
                        None,
                        &path.as_slice(),
                    );

                    let y0 = 2. * (self.height as f64 - self.padding.bottom as f64) + 10.;
                    let y1 = 2. * (self.height as f64 - self.padding.bottom as f64);

                    for i in 1..=9 {
                        let x = 2. * self.padding.left as f64
                            + i as f64 * 2. * self.plot_width() / 10.;
                        let path = [
                            PathEl::MoveTo((x, y0).into()),
                            PathEl::LineTo((x, y1).into()),
                        ];
                        scene.stroke(
                            &kurbo::Stroke::new(3.),
                            kurbo::Affine::IDENTITY,
                            peniko::BrushRef::Solid(AlphaColor::BLACK),
                            None,
                            &path.as_slice(),
                        );
                    }
                }
                PaddingLayer::YAxis => {
                    let path = [
                        PathEl::MoveTo(
                            (
                                2. * self.padding.left as f64,
                                2. * (self.height as f64 - self.padding.bottom as f64),
                            )
                                .into(),
                        ),
                        PathEl::LineTo(
                            (2. * self.padding.left as f64, 2. * self.padding.top as f64).into(),
                        ),
                    ];
                    let style = kurbo::Stroke::new(3.).with_caps(Cap::Square);
                    scene.stroke(
                        &style,
                        kurbo::Affine::IDENTITY,
                        peniko::BrushRef::Solid(AlphaColor::BLACK),
                        None,
                        &path.as_slice(),
                    );

                    let x0 = 2. * self.padding.left as f64 - 10.;
                    let x1 = 2. * self.padding.left as f64;

                    for i in 1..=9 {
                        let y = 2. * (self.height as f64 - self.padding.bottom as f64)
                            - i as f64 * 2. * self.plot_height() / 10.;
                        let path = [
                            PathEl::MoveTo((x0, y).into()),
                            PathEl::LineTo((x1, y).into()),
                        ];
                        scene.stroke(
                            &kurbo::Stroke::new(3.),
                            kurbo::Affine::IDENTITY,
                            peniko::BrushRef::Solid(AlphaColor::BLACK),
                            None,
                            &path.as_slice(),
                        );
                    }
                }
                _ => todo!(),
            }
        }

        self.scene = scene;
    }

    pub fn draw_bar<I, J, K>(&self, xs: I, ys: J, colors: K, scene: &mut Scene, scale_factor: f64)
    where
        I: Iterator<Item = (f64, f64)>,
        J: Iterator<Item = f64>,
        K: Iterator<Item = AlphaColor<Srgb>>,
    {
        zip_eq(zip_eq(xs, ys), colors).for_each(|((x, y), color)| {
            let (x, y) = (
                (self.transform_x(x.0), self.transform_x(x.1)),
                self.transform_y(y),
            );

            scene.fill(
                // TODO
                peniko::Fill::NonZero,
                kurbo::Affine::IDENTITY,
                peniko::BrushRef::Solid(color),
                None,
                &kurbo::Rect {
                    x0: x.0,
                    y0: 0.,
                    x1: x.1,
                    y1: y,
                },
            );
        });
    }

    pub fn draw_scatter<I, J, K, L>(
        &self,
        x: I,
        y: J,
        color: K,
        size: L,
        scene: &mut Scene,
        scale_factor: f64,
    ) where
        I: Iterator<Item = f64>,
        J: Iterator<Item = f64>,
        K: Iterator<Item = AlphaColor<Srgb>>,
        L: Iterator<Item = f64>,
    {
        zip_eq(zip_eq(zip_eq(x, y), color), size).for_each(|(((xi, yi), colour), size)| {
            let (xi, yi) = (
                scale_factor * self.transform_x(xi),
                scale_factor * self.transform_y(yi),
            );

            scene.fill(
                // TODO: what is this
                peniko::Fill::NonZero,
                kurbo::Affine::IDENTITY,
                peniko::BrushRef::Solid(colour),
                None,
                &kurbo::Circle::new((xi, yi), size),
            );
        });
    }

    pub fn draw_line<I, J>(
        &self,
        x: I,
        y: J,
        colour: AlphaColor<Srgb>,
        width: f64,
        scene: &mut Scene,
        scale_factor: f64,
    ) where
        I: Iterator<Item = f64>,
        J: Iterator<Item = f64>,
    {
        let path = zip_eq(x, y)
            .map(|(xi, yi)| {
                (
                    scale_factor * self.transform_x(xi),
                    scale_factor * self.transform_y(yi),
                )
            })
            .map(|(xi, yi)| PathEl::LineTo((xi, yi).into()))
            // TODO: upstream impl Shape for Iterator<Item = PathEl>
            .collect::<Vec<_>>();

        let style = kurbo::Stroke::new(width);
        scene.stroke(
            &style,
            kurbo::Affine::IDENTITY,
            peniko::BrushRef::Solid(colour),
            None,
            &path.as_slice(),
        );
    }
}

pub enum PaddingLayer {
    XAxis,
    YAxis,
    Title,
}

pub enum DataLayer {
    Line {
        xs: Vec<f64>,
        ys: Vec<f64>,
        color: AlphaColor<Srgb>,
        width: f64,
    },
    Scatter {
        xs: Vec<f64>,
        ys: Vec<f64>,
        color: Vec<AlphaColor<Srgb>>,
        size: Vec<f64>,
    },
    Bar {
        xs: Vec<(f64, f64)>,
        ys: Vec<f64>,
        colors: Vec<AlphaColor<Srgb>>,
    },
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: (f64, f64),
    pub y: (f64, f64),
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            x: (0.0, 1.0),
            y: (0.0, 1.0),
        }
    }
}

pub struct Padding {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}
