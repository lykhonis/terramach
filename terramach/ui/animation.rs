/*
 * Terra Mach
 * Copyright [2020] Terra Mach Authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::time::{Duration, Instant};

use terramach_graphics::{Rect, Size};

pub trait Interpolation {
    fn interpolate(&self, t: f32) -> f32;
}

impl<F: Fn(f32) -> f32> Interpolation for F {
    fn interpolate(&self, t: f32) -> f32 {
        (self)(t)
    }
}

pub trait Driver {
    fn is_driving(&self) -> bool;

    fn start(&mut self);

    fn stop(&mut self) {}

    fn advance(&mut self);

    fn value(&self) -> f32;
}

pub trait Animator<T> {
    fn animate<A: 'static + Animated<T>>(self, value: A) -> Animation<T>;
}

impl<T, D: 'static + Driver> Animator<T> for D {
    fn animate<A: 'static + Animated<T>>(self, value: A) -> Animation<T> {
        Animation::new(self, value)
    }
}

pub struct Animation<T> {
    driver: Box<dyn Driver>,
    animated: Box<dyn Animated<T>>,
    interpolation: Option<Box<dyn Interpolation>>,
}

impl<T> Animation<T> {
    pub fn new<D, A>(driver: D, animated: A) -> Self
        where
            D: 'static + Driver,
            A: 'static + Animated<T>,
    {
        Animation {
            driver: Box::new(driver),
            animated: Box::new(animated),
            interpolation: None,
        }
    }

    pub fn with<I: 'static + Interpolation>(mut self, interpolation: I) -> Self {
        self.interpolation = Some(if let Some(other) = self.interpolation.take() {
            Box::new(move |t| other.interpolate(t))
        } else {
            Box::new(interpolation)
        });
        self
    }

    pub fn is_animating(&self) -> bool {
        self.driver.is_driving()
    }

    pub fn start(&mut self) {
        self.driver.start();
    }

    pub fn stop(&mut self) {
        self.driver.stop();
    }

    pub fn advance(&mut self) -> bool {
        self.driver.advance();
        self.driver.is_driving()
    }

    pub fn value(&self) -> T {
        let fraction = if let Some(interpolation) = &self.interpolation {
            interpolation.interpolate(self.driver.value())
        } else {
            self.driver.value()
        };
        self.animated.animate(fraction)
    }
}

pub struct DurationDriver {
    timestamp: Option<Instant>,
    value: f32,
    duration: Duration,
}

impl DurationDriver {
    pub fn new(duration: Duration) -> Self {
        DurationDriver {
            timestamp: None,
            value: 0.0,
            duration,
        }
    }
}

impl Driver for DurationDriver {
    fn is_driving(&self) -> bool {
        self.timestamp.is_some()
    }

    fn start(&mut self) {
        self.timestamp = Some(Instant::now());
    }

    fn stop(&mut self) {
        self.timestamp = None;
    }

    fn advance(&mut self) {
        if self.timestamp.is_none() {
            self.timestamp = Some(Instant::now());
        }
        if let Some(timestamp) = &self.timestamp {
            if self.value == 1.0 {
                self.timestamp = None;
            } else {
                self.value = (timestamp.elapsed().as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
            }
        }
    }

    fn value(&self) -> f32 {
        self.value
    }
}

impl<T> Animator<T> for Duration {
    fn animate<A: 'static + Animated<T>>(self, value: A) -> Animation<T> {
        DurationDriver::new(self).animate(value)
    }
}

pub struct Tween<T: Animated<T>> {
    begin: T,
    end: T,
}

impl<T: Animated<T>> Tween<T> {
    pub fn new(begin: impl Into<T>, end: impl Into<T>) -> Self {
        Tween {
            begin: begin.into(),
            end: end.into(),
        }
    }
}

impl<T: Animated<T>> Animated<T> for Tween<T> {
    fn animate(&self, fraction: f32) -> T {
        let begin = self.begin.animate(1.0);
        let end = self.end.animate(1.0);
        begin.lerp(fraction, end)
    }

    fn lerp(&self, t: f32, other: T) -> T {
        self.begin.lerp(t, self.end.lerp(t, other))
    }
}

impl<T: Animated<T>> From<(T, T)> for Tween<T> {
    fn from(values: (T, T)) -> Self {
        Tween::new(values.0, values.1)
    }
}

pub struct AnimationOptions {}

impl AnimationOptions {
    pub fn reverse(reverse: impl Into<Option<bool>>) -> impl Interpolation {
        let reverse = reverse.into().unwrap_or_default();
        move |t| if reverse { 1.0 - t } else { t }
    }
}

pub struct AnimationCurves {}

impl AnimationCurves {
    pub fn linear() -> impl Interpolation {
        |t| t
    }

    pub fn ease_in() -> impl Interpolation {
        |t| t * t
    }

    pub fn ease_out() -> impl Interpolation {
        |t| t * (2.0 - t)
    }
}

pub trait Animated<T> {
    fn animate(&self, fraction: f32) -> T;

    fn lerp(&self, t: f32, other: T) -> T;
}

impl Animated<f32> for f32 {
    fn animate(&self, fraction: f32) -> f32 {
        self * fraction
    }

    fn lerp(&self, t: f32, other: f32) -> f32 {
        self * (1.0 - t) + other * t
    }
}

impl Animated<usize> for usize {
    fn animate(&self, fraction: f32) -> usize {
        (*self as f32 * fraction) as usize
    }

    fn lerp(&self, t: f32, other: usize) -> usize {
        (*self as f32 * (1.0 - t) + other as f32 * t) as usize
    }
}

impl Animated<u32> for u32 {
    fn animate(&self, fraction: f32) -> u32 {
        (*self as f32 * fraction) as u32
    }

    fn lerp(&self, t: f32, other: u32) -> u32 {
        (*self as f32 * (1.0 - t) + other as f32 * t) as u32
    }
}

impl Animated<i32> for i32 {
    fn animate(&self, fraction: f32) -> i32 {
        (*self as f32 * fraction) as i32
    }

    fn lerp(&self, t: f32, other: i32) -> i32 {
        (*self as f32 * (1.0 - t) + other as f32 * t) as i32
    }
}

impl Animated<Rect> for Rect {
    fn animate(&self, fraction: f32) -> Rect {
        Rect::new(
            self.left.animate(fraction),
            self.top.animate(fraction),
            self.right.animate(fraction),
            self.bottom.animate(fraction),
        )
    }

    fn lerp(&self, t: f32, other: Rect) -> Rect {
        Rect::new(
            self.left.lerp(t, other.left),
            self.top.lerp(t, other.top),
            self.right.lerp(t, other.right),
            self.bottom.lerp(t, other.bottom),
        )
    }
}

impl Animated<Size> for Size {
    fn animate(&self, fraction: f32) -> Size {
        Size::new(
            self.width.animate(fraction),
            self.height.animate(fraction),
        )
    }

    fn lerp(&self, t: f32, other: Size) -> Size {
        Size::new(
            self.width.lerp(t, other.width),
            self.height.lerp(t, other.height),
        )
    }
}
