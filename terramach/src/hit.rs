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

use crate::{Constraints, EventContext, Id, LayoutContext, MeasuredSize, PaintContext, Touch};
use crate::platform::Cursor;

use terramach_graphics::{Matrix, Point, Size};

pub struct EventResponder {
    widget: Id,
    transformation: Option<Matrix>,
    cursor: Option<Cursor>,
}

impl EventResponder {
    pub fn new(
        widget: Id,
        transformation: impl Into<Option<Matrix>>,
        cursor: impl Into<Option<Cursor>>,
    ) -> Self {
        EventResponder {
            widget,
            transformation: transformation.into(),
            cursor: cursor.into(),
        }
    }

    pub fn widget(&self) -> Id {
        self.widget
    }

    pub fn transformation(&self) -> Option<Matrix> {
        self.transformation
    }

    pub fn cursor(&self) -> Option<Cursor> {
        self.cursor
    }

    pub fn has_cursor(&self) -> bool {
        self.cursor.is_some()
    }

    pub fn transform_point(&self, point: impl Into<Point>) -> Point {
        if let Some(transformation) = self.transformation {
            transformation.map_point(point.into())
        } else {
            point.into()
        }
    }

    pub fn push_transformation(&mut self, transformation: impl Into<Option<Matrix>>) {
        let transformation = transformation.into();
        if let Some(current) = &mut self.transformation {
            if let Some(transformation) = transformation {
                current.pre_concat(&transformation);
            }
        } else {
            self.transformation = transformation;
        }
    }

    pub fn transform_touch(&self, touch: &Touch) -> Touch {
        Touch::new(touch.id(), self.transform_point(touch.location()))
    }
}

pub struct HitTestContext {
    size: Size,
    location: Point,
    transformation: Matrix,
    absorb: bool,
    become_responder: bool,
}

impl HitTestContext {
    pub fn new(size: impl Into<Size>, location: impl Into<Point>) -> Self {
        HitTestContext {
            size: size.into(),
            location: location.into(),
            transformation: Matrix::default(),
            absorb: false,
            become_responder: false,
        }
    }

    pub fn become_responder(&mut self) -> bool {
        self.become_responder = true;
        self.in_bounds()
    }

    pub fn requested_become_responder(&self) -> bool {
        self.become_responder
    }

    pub fn absorb(&mut self) -> bool {
        self.absorb = true;
        self.in_bounds()
    }

    pub fn absorbed(&self) -> bool {
        self.absorb
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn location(&self) -> Point {
        self.location
    }

    pub fn in_bounds(&self) -> bool {
        self.size.contains(self.location)
    }

    pub fn push_offset(&mut self, offset: impl Into<Point>) {
        self.transformation.post_translate(-offset.into());
    }

    pub fn transformation(&self) -> &Matrix {
        &self.transformation
    }
}
