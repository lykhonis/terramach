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

use terramach_graphics::{Color4f, Paint, SaveLayerRec};

#[derive(Clone)]
pub struct OpacityLayer {
    opacity: f32,
}

impl OpacityLayer {
    pub fn new(opacity: impl Into<f32>) -> Self {
        Self {
            opacity: opacity.into(),
        }
    }
}

impl Layer for OpacityLayer {
    fn draw(&self, draw: &mut DrawContext) {
        let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, self.opacity), None);
        let layer = SaveLayerRec::default().paint(&paint);
        let canvas = draw.canvas();
        let count = canvas.save_layer(&layer);
        draw.draw_children();
        let canvas = draw.canvas();
        canvas.restore_to_count(count);
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
