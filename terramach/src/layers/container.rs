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

use terramach_graphics::{Canvas, Size};

#[derive(Clone)]
pub struct ContainerLayer {
    layers: Vec<BoxedLayer>,
}

impl ContainerLayer {
    pub fn new() -> Self {
        ContainerLayer { layers: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn push(&mut self, layer: BoxedLayer) -> &mut Self {
        self.layers.push(layer);
        self
    }
}

impl Layer for ContainerLayer {
    fn draw(&self, draw: &mut DrawContext) {
        draw.draw_children_with_layers(&self.layers);
    }

    fn clone_boxed(&self) -> BoxedLayer {
        Box::new(self.clone())
    }
}
