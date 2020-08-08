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
    BoxedWidget, BuildContext, Constraints, LayoutContext, MeasuredSize, PaintContext,
    PartialWidget, Widget, WidgetContext,
};

use terramach_graphics::{Color, Color4f, Paint, Point, RRect, Rect, Size};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BorderRadius {
    top_left: Option<Point>,
    top_right: Option<Point>,
    bottom_left: Option<Point>,
    bottom_right: Option<Point>,
}

impl BorderRadius {
    pub fn new(
        top_left: impl Into<Option<Point>>,
        top_right: impl Into<Option<Point>>,
        bottom_left: impl Into<Option<Point>>,
        bottom_right: impl Into<Option<Point>>,
    ) -> Self {
        Self {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_left: bottom_left.into(),
            bottom_right: bottom_right.into(),
        }
    }

    pub fn new_all(radius: impl Into<Option<f32>>) -> Self {
        let radius = radius.into().map(|r| Point::new(r, r));
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn top_left(&self) -> Option<Point> {
        self.top_left
    }

    pub fn bottom_left(&self) -> Option<Point> {
        self.bottom_left
    }

    pub fn top_right(&self) -> Option<Point> {
        self.top_right
    }

    pub fn bottom_right(&self) -> Option<Point> {
        self.bottom_right
    }

    pub fn radii(&self) -> [Point; 4] {
        [
            self.top_left.unwrap_or_default(),
            self.top_right.unwrap_or_default(),
            self.bottom_right.unwrap_or_default(),
            self.bottom_left.unwrap_or_default(),
        ]
    }
}

impl Hash for BorderRadius {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(point) = &self.top_left {
            state.write_u32(point.x.to_bits());
            state.write_u32(point.y.to_bits());
        }
        if let Some(point) = &self.top_right {
            state.write_u32(point.x.to_bits());
            state.write_u32(point.y.to_bits());
        }
        if let Some(point) = &self.bottom_left {
            state.write_u32(point.x.to_bits());
            state.write_u32(point.y.to_bits());
        }
        if let Some(point) = &self.bottom_right {
            state.write_u32(point.x.to_bits());
            state.write_u32(point.y.to_bits());
        }
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Decoration {
    background_color: Option<Color>,
    border_radius: Option<BorderRadius>,
    child: Option<BoxedWidget>,
}

impl Decoration {
    pub fn new_empty(
        background_color: impl Into<Option<Color>>,
        border_radius: impl Into<Option<BorderRadius>>,
    ) -> Self {
        Self {
            background_color: background_color.into(),
            border_radius: border_radius.into(),
            child: None,
        }
    }

    pub fn new(
        background_color: impl Into<Option<Color>>,
        border_radius: impl Into<Option<BorderRadius>>,
        child: impl Into<BoxedWidget>,
    ) -> Self {
        Self {
            background_color: background_color.into(),
            border_radius: border_radius.into(),
            child: Some(child.into()),
        }
    }
}

impl Widget for Decoration {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        layout.layout_child(0, &layout.constraints().clone());
        layout.constraints().maximum_size()
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        if let Some(child) = &self.child {
            build.add_child(child.clone());
        }
    }

    fn paint(&self, _: &mut WidgetContext, paint: &mut PaintContext) {
        let bounds = Rect::from_size(paint.size());

        if let Some(border_radius) = self.border_radius {
            paint.push_clip_rrect(RRect::new_rect_radii(bounds, &border_radius.radii()));
        }

        if let Some(background_color) = self.background_color {
            let background = Paint::new(Color4f::from(background_color), None);
            let canvas = paint.canvas();
            canvas.draw_rect(bounds, &background);
        }

        paint.paint_children();
    }
}
