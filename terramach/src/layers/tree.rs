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

use terramach_graphics::{Canvas, Picture, PictureRecorder, Rect, Size, Display};
use terramach_graphics::gpu;

use crate::{BoxedLayer, DrawContext, Id, Tree};
use crate::gpu::TextureRegistry;

#[derive(Clone)]
struct LayerKey {
    size: Size,
    layers: Vec<Id>,
    leaf_layers: Vec<Id>,
}

impl LayerKey {
    fn new(
        size: impl Into<Size>,
        layers: impl Into<Option<Vec<Id>>>,
        leaf_layers: impl Into<Option<Vec<Id>>>,
    ) -> Self {
        LayerKey {
            size: size.into(),
            layers: layers.into().unwrap_or_default(),
            leaf_layers: leaf_layers.into().unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub struct LayerTree {
    tree: Tree<BoxedLayer>,
    keys: HashMap<Id, LayerKey>,
    layer_key: HashMap<Id, Id>,
}

impl LayerTree {
    pub fn new() -> Self {
        LayerTree {
            tree: Tree::new(),
            keys: HashMap::new(),
            layer_key: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: Id,
        size: impl Into<Size>,
        layer: BoxedLayer,
        parent: impl Into<Option<Id>>,
    ) -> Id {
        let id = self.tree.insert(layer, parent);
        if let Some(key) = self.keys.get_mut(&key) {
            key.layers.push(id);
        } else {
            self.keys.insert(key, LayerKey::new(size, vec![id], None));
        }
        self.layer_key.insert(id, key);
        id
    }

    pub fn insert_leaf(
        &mut self,
        key: Id,
        size: impl Into<Size>,
        layer: BoxedLayer,
        parent: impl Into<Option<Id>>,
    ) -> Id {
        let id = self.tree.insert(layer, parent);
        if let Some(key) = self.keys.get_mut(&key) {
            key.leaf_layers.push(id);
        } else {
            self.keys.insert(key, LayerKey::new(size, None, vec![id]));
        }
        self.layer_key.insert(id, key);
        id
    }

    pub fn key_layers(&self, key: Id) -> Option<&Vec<Id>> {
        Some(&self.keys.get(&key)?.layers)
    }

    pub fn drop_key_layer(&mut self, key: Id) {
        if let Some(key) = self.keys.remove(&key) {
            for id in &key.layers {
                self.layer_key.remove(id);
                if let Some(removed) = self.tree.remove_all(*id) {
                    for id in removed {
                        if let Some(key) = self.layer_key.remove(&id) {
                            self.drop_key_layer(key);
                        }
                    }
                }
            }
        }
    }

    pub fn parent_key_layer(&self, key: Id) -> Option<Id> {
        let key = self.keys.get(&key)?;
        let leaf_layer = key.layers.last()?;
        Some(*leaf_layer)
    }

    pub fn layer(&self, id: Id) -> Option<&BoxedLayer> {
        self.tree.node(id)
    }

    pub fn layer_mut(&mut self, id: Id) -> Option<&mut BoxedLayer> {
        self.tree.node_mut(id)
    }

    pub fn draw(&self, canvas: &mut Canvas, size: impl Into<Size>, textures: &mut TextureRegistry) {
        if let Some(children) = self.tree.children(None) {
            for child in children {
                self.draw_layer(canvas, textures, *child);
            }
        }
    }

    fn draw_layer(&self, canvas: &mut Canvas, textures: &mut TextureRegistry, id: Id) {
        if let Some(layer) = self.tree.node(id) {
            let mut draw_children = move |canvas: &mut Canvas, textures: &mut TextureRegistry| {
                if let Some(child_layers) = self.tree.children(id) {
                    for child in child_layers {
                        self.draw_layer(canvas, textures, *child);
                    }
                }
            };
            let size = {
                let key = self
                    .layer_key
                    .get(&id)
                    .expect("Layer does not have associated key!");
                let key = self
                    .keys
                    .get(&key)
                    .expect("Layer does not have associated key!");
                key.size
            };
            layer.draw(&mut DrawContext::new(
                size,
                canvas,
                textures,
                &mut draw_children,
            ));
        }
    }
}
