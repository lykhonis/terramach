/*
 * Terra Mach
 * Copyright [2020] Volodymyr Lykhonis
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

use std::collections::HashMap;

use terramach_graphics::{Point, Size};

use crate::Id;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Fit {
    Contain,
    Cover,
    Fill,
}

impl Default for Fit {
    fn default() -> Self {
        Fit::Contain
    }
}

pub trait MeasuredSize {
    fn new_unbound() -> Self;

    fn new_unbound_height(width: f32) -> Self;

    fn new_unbound_width(height: f32) -> Self;

    fn constrain(&self, constraints: &Constraints) -> Size;

    fn deflate(&self, amount: impl Into<(f32, f32)>) -> Size;

    fn inflate(&self, amount: impl Into<(f32, f32)>) -> Size;

    fn contains(&self, point: impl Into<Point>) -> bool;

    fn min(&self, other: impl Into<Size>) -> Size;

    fn max(&self, other: impl Into<Size>) -> Size;

    fn fit(&self, size: impl Into<Size>, fit: Fit) -> Size;
}

impl MeasuredSize for Size {
    fn new_unbound() -> Self {
        Size::new(std::f32::INFINITY, std::f32::INFINITY)
    }

    fn new_unbound_height(width: f32) -> Self {
        Size::new(width, std::f32::INFINITY)
    }

    fn new_unbound_width(height: f32) -> Self {
        Size::new(std::f32::INFINITY, height)
    }

    fn constrain(&self, constraints: &Constraints) -> Size {
        Size::new(
            self.width
                .min(constraints.maximum_size().width)
                .max(constraints.minimum_size().width),
            self.height
                .min(constraints.maximum_size().height)
                .max(constraints.minimum_size().height),
        )
    }

    fn deflate(&self, amount: impl Into<(f32, f32)>) -> Size {
        let amount = amount.into();
        Size::new(self.width - amount.0, self.height - amount.1)
    }

    fn inflate(&self, amount: impl Into<(f32, f32)>) -> Size {
        let amount = amount.into();
        Size::new(self.width + amount.0, self.height + amount.1)
    }

    fn contains(&self, point: impl Into<Point>) -> bool {
        let point = point.into();
        point.x >= 0.0 && point.y >= 0.0 && point.x < self.width && point.y < self.height
    }

    fn min(&self, other: impl Into<Size>) -> Size {
        let size = other.into();
        Size::new(self.width.min(size.width), self.height.min(size.height))
    }

    fn max(&self, other: impl Into<Size>) -> Size {
        let size = other.into();
        Size::new(self.width.max(size.width), self.height.max(size.height))
    }

    fn fit(&self, size: impl Into<Size>, fit: Fit) -> Size {
        let size = size.into();
        let width_scale = size.width / self.width;
        let height_scale = size.height / self.height;
        match fit {
            Fit::Contain => {
                let scale = width_scale.min(height_scale).min(1.0);
                Size::new(self.width * scale, self.height * scale)
            }
            Fit::Cover => {
                let scale = width_scale.max(height_scale).max(1.0);
                Size::new(self.width * scale, self.height * scale)
            }
            Fit::Fill => size,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Constraints {
    minimum_size: Size,
    maximum_size: Size,
}

impl Constraints {
    pub fn new(
        minimum_size: impl Into<Option<Size>>,
        maximum_size: impl Into<Option<Size>>,
    ) -> Self {
        Constraints {
            minimum_size: minimum_size.into().unwrap_or(Size::new_empty()),
            maximum_size: maximum_size.into().unwrap_or(Size::new_unbound()),
        }
    }

    pub fn new_tight(size: impl Into<Size>) -> Self {
        let size = size.into();
        Constraints {
            minimum_size: size,
            maximum_size: size,
        }
    }

    pub fn new_loose(size: impl Into<Size>) -> Self {
        let size = size.into();
        Constraints {
            minimum_size: Size::new_empty(),
            maximum_size: size,
        }
    }

    pub fn minimum_size(&self) -> Size {
        self.minimum_size
    }

    pub fn maximum_size(&self) -> Size {
        self.maximum_size
    }

    pub fn constrain(&self, constraints: &Constraints) -> Constraints {
        Constraints::new(
            self.minimum_size.constrain(constraints),
            self.maximum_size.constrain(constraints),
        )
    }
}

impl From<(Size, Size)> for Constraints {
    fn from(size: (Size, Size)) -> Self {
        Constraints::new(size.0, size.1)
    }
}

pub struct LayoutContext<'a> {
    constraints: Constraints,
    child_count: usize,
    child_id: &'a dyn Fn(usize) -> Option<Id>,
    layout_child: &'a mut dyn FnMut(Id, &Constraints) -> Option<Size>,
    pub(crate) child_offsets: HashMap<Id, Point>,
}

impl<'a> LayoutContext<'a> {
    pub fn new<C, L>(
        constraints: impl Into<Option<Constraints>>,
        child_count: usize,
        child_id: &'a C,
        layout_child: &'a mut L,
    ) -> Self where
        C: Fn(usize) -> Option<Id>,
        L: FnMut(Id, &Constraints) -> Option<Size> {
        LayoutContext {
            constraints: constraints
                .into()
                .unwrap_or(Constraints::new_loose(Size::new_unbound())),
            child_count,
            child_offsets: HashMap::new(),
            child_id,
            layout_child,
        }
    }

    pub fn child_count(&self) -> usize {
        self.child_count
    }

    pub fn constraints(&self) -> &Constraints {
        &self.constraints
    }

    pub fn layout_child(&mut self, index: usize, constraints: &Constraints) -> Option<Size> {
        let child_id = (self.child_id)(index)?;
        (self.layout_child)(child_id, &constraints)
    }

    pub fn set_child_offset(&mut self, index: usize, offset: impl Into<Point>) {
        if let Some(child_id) = (self.child_id)(index) {
            self.child_offsets.insert(child_id, offset.into());
        }
    }
}
