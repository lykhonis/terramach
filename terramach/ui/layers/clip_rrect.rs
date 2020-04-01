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

use terramach_graphics::RRect;

#[derive(Clone)]
pub struct ClipRRectLayer {
    clip_rrect: RRect,
}

impl ClipRRectLayer {
    pub fn new(clip_rrect: impl Into<RRect>) -> Self {
        ClipRRectLayer {
            clip_rrect: clip_rrect.into(),
        }
    }
}

impl Layer for ClipRRectLayer {
    fn draw(&self, draw: &mut DrawContext) {
        let canvas = draw.canvas();
        let count = canvas.save();
        canvas.clip_rrect(&self.clip_rrect, None, true);
        draw.draw_children();
        let canvas = draw.canvas();
        canvas.restore_to_count(count);
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
