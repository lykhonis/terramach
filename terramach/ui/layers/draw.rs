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

use terramach_graphics::{Canvas, Size};
use terramach_graphics::gpu;

use crate::gpu::{TextureRegistry, TextureId};
use crate::BoxedLayer;

pub struct DrawContext<'a> {
    size: Size,
    canvas: &'a mut Canvas,
    textures: &'a mut TextureRegistry,
    draw_children: &'a mut dyn FnMut(&mut Canvas, &mut TextureRegistry),
    draw_layers: Option<&'a [&'a BoxedLayer]>,
}

impl<'a> DrawContext<'a> {
    pub fn new<F>(
        size: impl Into<Size>,
        canvas: &'a mut Canvas,
        textures: &'a mut TextureRegistry,
        draw_children: &'a mut F,
    ) -> Self where F: FnMut(&mut Canvas, &mut TextureRegistry) {
        DrawContext {
            size: size.into(),
            canvas,
            textures,
            draw_children,
            draw_layers: None,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    pub fn draw_texture(&mut self, id: TextureId) {
        if let Some(texture) = self.textures.texture(id) {
            texture.preroll(self.size);
            texture.draw(self.canvas);
        }
    }

    pub fn draw_children(&mut self) {
        if let Some(layers) = &self.draw_layers {
            if let Some(layer) = layers.first() {
                let mut context = DrawContext {
                    size: self.size,
                    canvas: self.canvas,
                    textures: self.textures,
                    draw_children: self.draw_children,
                    draw_layers: Some(&layers[1..]),
                };
                layer.draw(&mut context);
                return;
            }
        }
        (self.draw_children)(self.canvas, self.textures);
    }

    pub fn draw_children_with_layers(&mut self, layers: &Vec<BoxedLayer>) {
        let mut draw_layers = Vec::new();
        if let Some(layers) = self.draw_layers {
            draw_layers.extend(layers);
        }
        draw_layers.extend(layers);
        let mut context = DrawContext {
            size: self.size,
            canvas: self.canvas,
            textures: self.textures,
            draw_children: self.draw_children,
            draw_layers: Some(&draw_layers[..]),
        };
        context.draw_children();
    }
}
