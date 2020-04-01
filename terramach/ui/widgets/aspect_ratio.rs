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
    AnyWidget, BoxedWidget, BuildContext, Constraints, LayoutContext, MeasuredSize, PaintContext,
    PartialWidget, Widget, WidgetContext,
};

use terramach_graphics::Size;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct AspectRatio {
    ratio: f32,
    child: Option<BoxedWidget>,
}

impl AspectRatio {
    pub fn new(ratio: impl Into<f32>, child: impl Into<BoxedWidget>) -> Self {
        AspectRatio {
            ratio: ratio.into(),
            child: Some(child.into()),
        }
    }

    pub fn new_empty(ratio: impl Into<f32>) -> Self {
        AspectRatio {
            ratio: ratio.into(),
            child: None,
        }
    }

    fn apply_ratio(&self, size: Size) -> Size {
        if size.width.is_finite() {
            Size::new(size.width, size.width / self.ratio)
        } else {
            Size::new(size.height * self.ratio, size.height)
        }
    }
}

impl Widget for AspectRatio {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let child_constraints = Constraints::new(
            self.apply_ratio(layout.constraints().minimum_size()),
            self.apply_ratio(layout.constraints().maximum_size()),
        )
        .constrain(layout.constraints());
        layout
            .layout_child(0, &child_constraints)
            .unwrap_or(child_constraints.minimum_size())
            .constrain(&child_constraints)
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        if let Some(child) = &self.child {
            build.add_child(child.clone());
        }
    }
}
