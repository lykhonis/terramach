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

use crate::{BoxedLayer, DrawContext, Layer};

use terramach_graphics::Point;

#[derive(Clone)]
pub struct OffsetLayer {
    offset: Point,
}

impl OffsetLayer {
    pub fn new(offset: impl Into<Point>) -> Self {
        OffsetLayer {
            offset: offset.into(),
        }
    }
}

impl Layer for OffsetLayer {
    fn draw(&self, draw: &mut DrawContext) {
        let canvas = draw.canvas();
        let count = canvas.save();
        canvas.translate(self.offset);
        draw.draw_children();
        let canvas = draw.canvas();
        canvas.restore_to_count(count);
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
