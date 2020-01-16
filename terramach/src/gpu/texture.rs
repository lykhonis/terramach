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

use std::collections::HashMap;

use crate::Id;
use crate::gpu::Texture;

pub type TextureId = Id;

pub struct TextureContext {}

impl TextureContext {
    pub fn new() -> Self {
        TextureContext {}
    }
}

#[derive(Default)]
pub struct TextureRegistry {
    textures: HashMap<TextureId, Texture>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        TextureRegistry {
            textures: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: TextureId, texture: Texture) {
        debug_assert!(!self.textures.contains_key(&id), "Texture with the same id already registered");
        self.textures.insert(id, texture);
    }

    pub fn unregister(&mut self, id: TextureId) -> bool {
        self.textures.remove(&id).is_some()
    }

    pub fn texture(&mut self, id: TextureId) -> Option<&mut Texture> {
        self.textures.get_mut(&id)
    }

    pub fn textures(&self) -> std::collections::hash_map::Values<TextureId, Texture> {
        self.textures.values()
    }

    pub fn textures_mut(&mut self) -> std::collections::hash_map::ValuesMut<TextureId, Texture> {
        self.textures.values_mut()
    }
}
