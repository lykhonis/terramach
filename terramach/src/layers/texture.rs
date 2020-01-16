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

use crate::gpu::TextureId;
use crate::{Layer, DrawContext, BoxedLayer};

#[derive(Clone)]
pub struct TextureLayer {
    texture_id: TextureId,
}

impl TextureLayer {
    pub fn new(texture_id: impl Into<TextureId>) -> Self {
        TextureLayer {
            texture_id: texture_id.into(),
        }
    }
}

impl Layer for TextureLayer {
    fn draw(&self, draw: &mut DrawContext) {
        draw.draw_texture(self.texture_id);
        draw.draw_children();
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
