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

use crate::{BoxedLayer, DrawContext, Layer};

use terramach_graphics::Rect;

#[derive(Clone)]
pub struct ClipRectLayer {
    clip_rect: Rect,
}

impl ClipRectLayer {
    pub fn new(clip_rect: impl Into<Rect>) -> Self {
        ClipRectLayer {
            clip_rect: clip_rect.into(),
        }
    }
}

impl Layer for ClipRectLayer {
    fn draw(&self, draw: &mut DrawContext) {
        let canvas = draw.canvas();
        let count = canvas.save();
        canvas.clip_rect(self.clip_rect, None, None);
        draw.draw_children();
        let canvas = draw.canvas();
        canvas.restore_to_count(count);
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
