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
    AnyWidget, BoxedWidget, BuildContext, Fit, HitTestContext, LayoutContext, MeasuredSize,
    PaintContext, PartialWidget, Widget, WidgetContext,
};

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Opacity {
    opacity: f32,
    child: BoxedWidget,
}

impl Opacity {
    pub fn new(opacity: impl Into<f32>, child: impl Into<BoxedWidget>) -> Self {
        Opacity {
            opacity: opacity.into(),
            child: child.into(),
        }
    }
}

impl Widget for Opacity {
    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }

    fn paint(&self, _: &mut WidgetContext, paint: &mut PaintContext) {
        paint.push_opacity(self.opacity);
        paint.paint_children();
    }
}
