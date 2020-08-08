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

use std::hash::{Hash, Hasher};

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Constraints, LayoutContext, MeasuredSize, PaintContext,
    PartialWidget, Widget, WidgetContext,
};

use terramach_graphics::{Point, Size};

#[derive(Clone, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    Middle,
    End,
}

impl Default for MainAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    Middle,
    End,
    Stretch,
}

impl Default for CrossAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, PartialEq)]
pub struct Alignment {
    horizontal: f32,
    vertical: f32,
}

impl Alignment {
    pub fn new(horizontal: impl Into<Option<f32>>, vertical: impl Into<Option<f32>>) -> Self {
        Self {
            horizontal: horizontal.into().unwrap_or_default(),
            vertical: vertical.into().unwrap_or_default(),
        }
    }

    pub fn top_left() -> Self {
        Self::new(-1.0, -1.0)
    }

    pub fn top_right() -> Self {
        Self::new(1.0, -1.0)
    }

    pub fn bottom_left() -> Self {
        Self::new(1.0, -1.0)
    }

    pub fn bottom_right() -> Self {
        Self::new(1.0, 1.0)
    }

    pub fn center() -> Self {
        Self::new(None, None)
    }

    pub fn top_center() -> Self {
        Self::new(None, -1.0)
    }

    pub fn bottom_center() -> Self {
        Self::new(None, 1.0)
    }

    pub fn left_center() -> Self {
        Self::new(-1.0, None)
    }

    pub fn right_center() -> Self {
        Self::new(1.0, None)
    }

    pub fn horizontal(&self) -> f32 {
        self.horizontal
    }

    pub fn vertical(&self) -> f32 {
        self.vertical
    }

    pub fn align(&self, area: impl Into<Size>, item: impl Into<Size>) -> Point {
        let area = area.into();
        let item = item.into();
        Point::new(
            (area.width - item.width) * (1.0 + self.horizontal) / 2.0,
            (area.height - item.height) * (1.0 + self.vertical) / 2.0,
        )
    }
}

impl Hash for Alignment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.horizontal.to_bits().hash(state);
        self.vertical.to_bits().hash(state);
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self::center()
    }
}

impl From<(f32, f32)> for Alignment {
    fn from(values: (f32, f32)) -> Self {
        Self::new(values.0, values.1)
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Align {
    alignment: Alignment,
    child: BoxedWidget,
}

impl Align {
    pub fn new(alignment: impl Into<Option<Alignment>>, child: impl Into<BoxedWidget>) -> Self {
        Self {
            alignment: alignment.into().unwrap_or_default(),
            child: child.into(),
        }
    }
}

impl Widget for Align {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let child_constraints = Constraints::new_loose(layout.constraints().maximum_size());
        let child_size = layout
            .layout_child(0, &child_constraints)
            .unwrap_or_default();
        let size = child_size.constrain(layout.constraints());
        layout.set_child_offset(0, self.alignment.align(size, child_size));
        size
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }
}
