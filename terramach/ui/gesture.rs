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

use std::ops::Sub;

use crate::{Touches, TouchId};

use terramach_graphics::Point;

const MINIMUM_PAN_DISTANCE: f32 = 5.0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TapGestureState {
    Possible,
    Began,
    Changed,
    Ended,
}

pub struct TapGesture {
    touches_count: usize,
    tap_count: usize,
    start_touches_count: usize,
    state: TapGestureState,
}

impl TapGesture {
    pub fn new(touches_count: usize, tap_count: usize) -> Self {
        TapGesture {
            touches_count,
            tap_count,
            start_touches_count: 0,
            state: TapGestureState::Possible,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            TapGestureState::Possible | TapGestureState::Ended => false,
            TapGestureState::Began | TapGestureState::Changed => true,
        }
    }

    pub fn update(&mut self, touches: &Touches) -> TapGestureState {
        let touches_count = touches.len();
        let state = match self.state {
            TapGestureState::Possible | TapGestureState::Ended => {
                if touches_count >= self.touches_count {
                    self.start_touches_count = touches_count;
                    TapGestureState::Began
                } else {
                    TapGestureState::Possible
                }
            }
            TapGestureState::Began | TapGestureState::Changed => {
                if self.start_touches_count - touches_count == self.tap_count {
                    TapGestureState::Ended
                } else {
                    TapGestureState::Changed
                }
            }
        };
        self.state = state;
        state
    }
}

impl Default for TapGesture {
    fn default() -> Self {
        TapGesture::new(1, 1)
    }
}

impl Clone for TapGesture {
    fn clone(&self) -> Self {
        TapGesture::new(self.touches_count, self.tap_count)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PanGestureState {
    Possible,
    Began(Point),
    Changed(Point),
    Ended,
}

pub struct PanGesture {
    minimum_touches: usize,
    maximum_touches: usize,
    start_location: Option<Point>,
    state: PanGestureState,
}

impl PanGesture {
    pub fn new(minimum_touches: usize, maximum_touches: usize) -> Self {
        PanGesture {
            minimum_touches,
            maximum_touches,
            start_location: None,
            state: PanGestureState::Possible,
        }
    }

    pub fn limit(touches_count: usize) -> Self {
        PanGesture::new(touches_count, touches_count)
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            PanGestureState::Possible | PanGestureState::Ended => false,
            PanGestureState::Began(_) | PanGestureState::Changed(_) => true,
        }
    }

    pub fn update(&mut self, touches: &Touches) -> PanGestureState {
        let touches_count = touches.len();
        if touches_count < self.minimum_touches || touches_count > self.maximum_touches {
            let state = match self.state {
                PanGestureState::Possible => return PanGestureState::Possible,
                PanGestureState::Ended => PanGestureState::Possible,
                PanGestureState::Began(_) | PanGestureState::Changed(_) => PanGestureState::Ended,
            };
            self.start_location = None;
            self.state = state;
            return state;
        }
        let location = center_location(touches);
        let state = match self.state {
            PanGestureState::Possible | PanGestureState::Ended => {
                let threshold = self.start_location.get_or_insert(location).sub(location);
                if threshold.x.abs() >= MINIMUM_PAN_DISTANCE
                    || threshold.y.abs() >= MINIMUM_PAN_DISTANCE
                {
                    PanGestureState::Began(location)
                } else {
                    PanGestureState::Possible
                }
            }
            PanGestureState::Began(_) | PanGestureState::Changed(_) => {
                PanGestureState::Changed(location)
            }
        };
        self.state = state;
        state
    }

    pub fn location(&self) -> Option<Point> {
        match self.state {
            PanGestureState::Began(location) | PanGestureState::Changed(location) => {
                Some(location)
            }
            _ => None,
        }
    }
}

impl Default for PanGesture {
    fn default() -> Self {
        PanGesture::new(1, 10)
    }
}

impl Clone for PanGesture {
    fn clone(&self) -> Self {
        PanGesture::new(self.minimum_touches, self.maximum_touches)
    }
}

fn center_location(touches: &Touches) -> Point {
    let touches_count = touches.len();
    if touches_count == 0 {
        return Point::default();
    }
    let total_location = touches
        .all()
        .iter()
        .map(|touch| {
            let location = touch.location();
            (location.x, location.y)
        })
        .fold((0.0f32, 0.0f32), |total, touch| {
            (total.0 + touch.0, total.1 + touch.1)
        });
    Point::new(
        total_location.0 / touches_count as f32,
        total_location.1 / touches_count as f32,
    )
}

#[derive(Debug, Copy, Clone)]
pub enum PinchGestureState {
    Possible,
    Began,
    Changed {
        scale: f32,
        velocity: f32,
    },
    Ended,
}

pub struct PinchGesture {
    start_distance: Option<Point>,
    center_location: Option<Point>,
    state: PinchGestureState,
}

impl PinchGesture {
    pub fn new() -> Self {
        PinchGesture {
            state: PinchGestureState::Possible,
            start_distance: None,
            center_location: None,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            PinchGestureState::Possible | PinchGestureState::Ended => false,
            PinchGestureState::Began | PinchGestureState::Changed { .. } => true,
        }
    }

    pub fn update(&mut self, touches: &Touches) -> PinchGestureState {
        let touches_required = 2; // two fingers
        let touches_count = touches.len();
        if touches_count != touches_required {
            let state = match self.state {
                PinchGestureState::Possible => return PinchGestureState::Possible,
                PinchGestureState::Ended => PinchGestureState::Possible,
                PinchGestureState::Began | PinchGestureState::Changed { .. } => PinchGestureState::Ended,
            };
            self.start_distance = None;
            self.state = state;
            return state;
        }
        self.center_location = Some(center_location(touches));
        let state = match self.state {
            PinchGestureState::Possible | PinchGestureState::Ended => {
                let touches = touches.all();
                let first = touches.get(0).unwrap();
                let second = touches.get(1).unwrap();
                self.start_distance = Some(second.location() - first.location());
                PinchGestureState::Began
            }
            PinchGestureState::Began | PinchGestureState::Changed { .. } => {
                let touches = touches.all();
                let first = touches.get(0).unwrap();
                let second = touches.get(1).unwrap();
                let distance = second.location() - first.location();
                let start_distance = self.start_distance.unwrap();
                let scale = distance.y.hypot(distance.x) /
                    start_distance.y.hypot(start_distance.x);
                // TODO: Implement velocity
                let velocity = 0.0;
                PinchGestureState::Changed {
                    scale,
                    velocity,
                }
            }
        };
        self.state = state;
        state
    }

    pub fn center_location(&self) -> Option<Point> {
        match self.state {
            PinchGestureState::Possible | PinchGestureState::Ended => None,
            PinchGestureState::Began | PinchGestureState::Changed { .. } => self.center_location,
        }
    }

    pub fn scale(&self) -> Option<f32> {
        match self.state {
            PinchGestureState::Began => Some(1.0),
            PinchGestureState::Changed { scale, .. } => Some(scale),
            _ => None,
        }
    }

    pub fn velocity(&self) -> Option<f32> {
        match self.state {
            PinchGestureState::Began => Some(0.0),
            PinchGestureState::Changed { velocity, .. } => Some(velocity),
            _ => None,
        }
    }
}

impl Default for PinchGesture {
    fn default() -> Self {
        PinchGesture::new()
    }
}

impl Clone for PinchGesture {
    fn clone(&self) -> Self {
        PinchGesture::new()
    }
}
