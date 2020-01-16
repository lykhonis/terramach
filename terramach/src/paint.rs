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

use terramach_graphics::{Canvas, PictureRecorder, Point, Rect, RRect, Size};

use crate::{ClipRectLayer, ContainerLayer, Layer, OffsetLayer, OpacityLayer, PictureLayer, TextureLayer, ClipRRectLayer};
use crate::gpu::TextureId;

pub struct PaintContext {
    recorder: Option<PictureRecorder>,
    size: Size,
    painted_children: bool,
    layer: Option<ContainerLayer>,
    leaf_layer: Option<ContainerLayer>,
}

impl PaintContext {
    pub fn new(size: impl Into<Size>) -> Self {
        PaintContext {
            recorder: None,
            size: size.into(),
            painted_children: false,
            layer: None,
            leaf_layer: None,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn layers(&mut self) -> Option<&ContainerLayer> {
        self.push_current_if_any();
        self.layer.as_ref()
    }

    pub fn leaf_layers(&mut self) -> Option<&ContainerLayer> {
        self.push_current_if_any();
        self.leaf_layer.as_ref()
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        let bounds = Rect::from_size(self.size);
        let recorder = self.recorder.get_or_insert_with(|| {
            let mut recorder = PictureRecorder::new();
            recorder.begin_recording(bounds, None, None);
            recorder
        });
        recorder.recording_canvas()
    }

    fn push_current_if_any(&mut self) {
        if let Some(mut recorder) = self.recorder.take() {
            if let Some(picture) = recorder.finish_recording_as_picture(None) {
                self.push_inner_layer(PictureLayer::new(picture));
            }
        }
    }

    fn push_inner_layer<T: 'static + Layer>(&mut self, layer: T) {
        let layer = Box::new(layer);
        if self.painted_children {
            self.leaf_layer
                .get_or_insert_with(|| ContainerLayer::new())
                .push(layer);
        } else {
            self.layer
                .get_or_insert_with(|| ContainerLayer::new())
                .push(layer);
        }
    }

    pub fn push_layer<T: 'static + Layer>(&mut self, layer: T) {
        self.push_current_if_any();
        self.push_inner_layer(layer);
    }

    pub fn push_texture(&mut self, texture_id: impl Into<TextureId>) {
        self.push_layer(TextureLayer::new(texture_id));
    }

    pub fn push_offset(&mut self, offset: impl Into<Point>) {
        self.push_layer(OffsetLayer::new(offset));
    }

    pub fn push_clip_rect(&mut self, rect: impl Into<Rect>) {
        self.push_layer(ClipRectLayer::new(rect));
    }

    pub fn push_clip_rrect(&mut self, rect: impl Into<RRect>) {
        self.push_layer(ClipRRectLayer::new(rect));
    }

    pub fn push_opacity(&mut self, opacity: impl Into<f32>) {
        self.push_layer(OpacityLayer::new(opacity));
    }

    pub fn paint_children(&mut self) {
        self.push_current_if_any();
        self.painted_children = true;
    }
}
