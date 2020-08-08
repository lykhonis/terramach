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

use std::sync::Arc;

use crate::{BoxedLayer, DrawContext, Layer};

use terramach_graphics::Picture;

#[derive(Clone)]
pub struct PictureLayer {
    picture: Arc<Picture>,
}

unsafe impl Send for PictureLayer {}

impl PictureLayer {
    pub fn new(picture: Picture) -> Self {
        Self {
            picture: Arc::new(picture),
        }
    }
}

impl Layer for PictureLayer {
    fn draw(&self, draw: &mut DrawContext) {
        self.picture.playback(draw.canvas());
        draw.draw_children();
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
