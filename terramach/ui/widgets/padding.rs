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

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Constraints, HitTestContext, LayoutContext, MeasuredSize,
    PaintContext, PartialWidget, Widget, WidgetContext,
};

use terramach_graphics::Size;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Padding {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    child: Option<BoxedWidget>,
}

impl Padding {
    pub fn new_empty(
        left: impl Into<Option<f32>>,
        top: impl Into<Option<f32>>,
        right: impl Into<Option<f32>>,
        bottom: impl Into<Option<f32>>,
    ) -> Self {
        Padding {
            left: left.into().unwrap_or_default(),
            top: top.into().unwrap_or_default(),
            right: right.into().unwrap_or_default(),
            bottom: bottom.into().unwrap_or_default(),
            child: None,
        }
    }

    pub fn new(
        left: impl Into<Option<f32>>,
        top: impl Into<Option<f32>>,
        right: impl Into<Option<f32>>,
        bottom: impl Into<Option<f32>>,
        child: impl Into<BoxedWidget>,
    ) -> Self {
        Padding {
            left: left.into().unwrap_or_default(),
            top: top.into().unwrap_or_default(),
            right: right.into().unwrap_or_default(),
            bottom: bottom.into().unwrap_or_default(),
            child: Some(child.into()),
        }
    }

    pub fn new_all(padding: impl Into<Option<f32>>, child: impl Into<BoxedWidget>) -> Self {
        let padding = padding.into();
        Padding::new(padding, padding, padding, padding, child)
    }

    pub fn new_horizontal(padding: impl Into<Option<f32>>, child: impl Into<BoxedWidget>) -> Self {
        let padding = padding.into();
        Padding::new(padding, None, padding, None, child)
    }

    pub fn new_vertical(padding: impl Into<Option<f32>>, child: impl Into<BoxedWidget>) -> Self {
        let padding = padding.into();
        Padding::new(None, padding, None, padding, child)
    }
}

impl Widget for Padding {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let horizontal_padding = self.left + self.right;
        let vertical_padding = self.top + self.bottom;
        let constraints = layout.constraints();
        let maximum_size = constraints
            .maximum_size()
            .deflate((horizontal_padding, vertical_padding));
        let minimum_size = constraints.minimum_size().min(maximum_size);
        layout.set_child_offset(0, (self.left, self.top));
        layout.layout_child(0, &Constraints::new(minimum_size, maximum_size))
            .unwrap_or_default()
            .inflate((horizontal_padding, vertical_padding))
            .constrain(layout.constraints())
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        if let Some(child) = &self.child {
            build.add_child(child.clone());
        }
    }
}
