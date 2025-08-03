use std::ops::{Add, AddAssign};

use winit::window::Window;

#[derive(Debug, Clone, PartialEq)]
pub struct PlotLayout {
    pub width: f64,
    pub height: f64,
    pub padding: Padding,
    pub initial_bounds: Option<Bounds>,
    pub interaction_bounds: Bounds,
}

impl PlotLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_initial_bounds(mut self, bounds: Bounds) -> Self {
        self.initial_bounds = Some(bounds);
        self
    }

    pub fn with_interaction_bounds(mut self, bounds: Bounds) -> Self {
        self.interaction_bounds = bounds;
        self
    }

    pub(crate) fn instantiate(
        self,
        window: &Window,
        initial_data_bounds: Option<Bounds>,
    ) -> PlotInstanceLayout {
        let data_bounds = if let Some(initial_bounds) = self.initial_bounds {
            initial_bounds
        } else {
            initial_data_bounds.unwrap_or(Bounds::UNIT)
        };

        PlotInstanceLayout {
            logical_width: self.width,
            logical_height: self.height,
            padding: self.padding,
            data_bounds,
            interaction_bounds: self.interaction_bounds,
            scale_factor: window.scale_factor(),
        }
    }
}

impl Default for PlotLayout {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            padding: Padding {
                top: 20.0,
                bottom: 20.0,
                left: 50.0,
                right: 20.0,
            },
            initial_bounds: None,
            interaction_bounds: Bounds::INFINITY,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlotInstanceLayout {
    pub(crate) logical_width: f64,
    pub(crate) logical_height: f64,
    pub(crate) padding: Padding,

    pub(crate) data_bounds: Bounds,
    pub(crate) interaction_bounds: Bounds,

    pub(crate) scale_factor: f64,
}

impl PlotInstanceLayout {
    fn is_on_inner(&self, mouse_position: (f64, f64)) -> bool {
        let (mut x, mut y) = mouse_position;
        x /= self.scale_factor;
        y /= self.scale_factor;

        x >= self.padding.left
            && x <= self.logical_width - self.padding.right
            && y >= self.padding.top
            && y <= self.logical_width - self.padding.bottom
    }

    fn inner_width(&self) -> f64 {
        self.logical_width - self.padding.left - self.padding.right
    }

    fn inner_height(&self) -> f64 {
        self.logical_height - self.padding.top - self.padding.bottom
    }

    fn convert_to_data_position(&self, mouse_position: (f64, f64)) -> Option<(f64, f64)> {
        let logical_position = (
            mouse_position.0 / self.scale_factor,
            self.logical_height - mouse_position.1 / self.scale_factor,
        );
        let logical_plot_position = (
            logical_position.0 - self.padding.left,
            logical_position.1 - self.padding.bottom,
        );
        let percentage_plot_position = (
            logical_plot_position.0 / self.inner_width(),
            logical_plot_position.1 / self.inner_height(),
        );

        if percentage_plot_position.0 >= 0.
            && percentage_plot_position.0 <= 1.
            && percentage_plot_position.1 >= 0.
            && percentage_plot_position.1 <= 1.
        {
            Some((
                self.data_bounds.x.min + percentage_plot_position.0 * self.data_bounds.x.size(),
                self.data_bounds.y.min + percentage_plot_position.1 * self.data_bounds.y.size(),
            ))
        } else {
            None
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        // TODO: use u32 internally as well
        self.logical_width = width as f64 / self.scale_factor;
        self.logical_height = height as f64 / self.scale_factor;
    }

    pub(crate) fn drag(
        &mut self,
        start_drag_mouse_position: (f64, f64),
        pre_position: (f64, f64),
        current_position: (f64, f64),
    ) {
        if !self.is_on_inner(start_drag_mouse_position)
            || !self.is_on_inner(pre_position)
            || !self.is_on_inner(current_position)
        {
            return;
        }

        let change = (
            current_position.0 - pre_position.0,
            current_position.1 - pre_position.1,
        );

        let data_x =
            change.0 * self.data_bounds.x.size() / (self.scale_factor * self.inner_width());
        let data_y =
            change.1 * self.data_bounds.y.size() / (self.scale_factor * self.inner_height());

        self.data_bounds.x += data_x;
        self.data_bounds.y += data_y;

        self.data_bounds = self.interaction_bounds.clamp(self.data_bounds);
    }

    pub(crate) fn zoom(&mut self, mouse_position: (f64, f64), factor: f64) {
        if let Some(data_position) = self.convert_to_data_position(mouse_position) {
            self.data_bounds = Bounds {
                x: Interval {
                    min: data_position.0 - (data_position.0 - self.data_bounds.x.min) * factor,
                    max: data_position.0 + (self.data_bounds.x.max - data_position.0) * factor,
                },
                y: Interval {
                    min: data_position.1 - (data_position.1 - self.data_bounds.y.min) * factor,
                    max: data_position.1 + (self.data_bounds.y.max - data_position.1) * factor,
                },
            };

            self.data_bounds = self.interaction_bounds.bound(self.data_bounds);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: Interval,
    pub y: Interval,
}

impl Bounds {
    pub const UNIT: Self = Self {
        x: Interval::UNIT,
        y: Interval::UNIT,
    };

    pub const INFINITY: Self = Self {
        x: Interval::INFINITY,
        y: Interval::INFINITY,
    };

    #[inline]
    pub fn clamp(self, other: Self) -> Self {
        Self {
            x: self.x.clamp(other.x),
            y: self.y.clamp(other.y),
        }
    }

    #[inline]
    pub fn bound(self, other: Self) -> Self {
        Self {
            x: self.x.bound(other.x),
            y: self.y.bound(other.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub const UNIT: Self = Self { min: 0.0, max: 1.0 };

    pub const INFINITY: Self = Self {
        min: f64::NEG_INFINITY,
        max: f64::INFINITY,
    };

    #[inline]
    pub fn size(self) -> f64 {
        self.max - self.min
    }

    #[inline]
    pub fn clamp(self, other: Self) -> Self {
        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    #[inline]
    pub fn bound(self, other: Self) -> Self {
        if other.size() > self.size() {
            self
        } else if other.min < self.min {
            let shift = self.min - other.min;
            Self {
                min: self.min,
                max: self.max + shift,
            }
        } else if other.max > self.max {
            let shift = other.max - self.max;
            Self {
                min: self.min - shift,
                max: self.max,
            }
        } else {
            Self {
                min: other.min,
                max: other.max,
            }
        }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            min: self.min + other.min,
            max: self.max + other.max,
        }
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl Add<f64> for Interval {
    type Output = Interval;

    fn add(self, other: f64) -> Self::Output {
        Interval {
            min: self.min + other,
            max: self.max + other,
        }
    }
}

impl AddAssign<f64> for Interval {
    fn add_assign(&mut self, other: f64) {
        *self = *self + other;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}
