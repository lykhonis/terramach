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

use std::sync::mpsc::{channel, Receiver, Sender};

use terramach::*;
use terramach::gpu::*;
use terramach::graphics::ISize;

use mapbox;

use crate::settings::Settings;

#[derive(Clone)]
enum MapCommand {
    JumpTo(mapbox::CameraOptions),
    EaseTo(mapbox::CameraOptions, mapbox::AnimationOptions),
    MoveBy(mapbox::ScreenCoordinate, Option<mapbox::AnimationOptions>),
    ScaleBy(f32, Option<mapbox::ScreenCoordinate>, Option<mapbox::AnimationOptions>),
}

unsafe impl Send for MapCommand {}

#[derive(Default)]
pub struct MapController {
    channels: Vec<Channel<MapCommand>>,
}

impl MapController {
    pub fn new() -> Self {
        MapController {
            channels: Vec::new(),
        }
    }

    fn bind(&mut self, channel: Channel<MapCommand>) {
        self.channels.push(channel);
    }

    fn broadcast(&mut self, command: MapCommand) {
        for i in self.channels.len() - 1..=0 {
            let channel = &mut self.channels[i];
            if channel.is_active() {
                channel.send(command.clone());
            } else {
                self.channels.remove(i);
            }
        }
    }

    pub fn jump_to(&mut self, camera: mapbox::CameraOptions) {
        self.broadcast(MapCommand::JumpTo(camera));
    }

    pub fn ease_to(
        &mut self,
        camera: mapbox::CameraOptions,
        animation: mapbox::AnimationOptions,
    ) {
        self.broadcast(MapCommand::EaseTo(camera, animation));
    }

    pub fn move_by(
        &mut self,
        coordinate: impl Into<mapbox::ScreenCoordinate>,
        animation: impl Into<Option<mapbox::AnimationOptions>>,
    ) {
        self.broadcast(MapCommand::MoveBy(coordinate.into(), animation.into()));
    }

    pub fn scale_by(
        &mut self,
        scale: f32,
        anchor: impl Into<Option<mapbox::ScreenCoordinate>>,
        animation: impl Into<Option<mapbox::AnimationOptions>>,
    ) {
        self.broadcast(MapCommand::ScaleBy(scale, anchor.into(), animation.into()));
    }
}

#[derive(Clone, PartialWidget)]
pub struct Map {
    channel: Channel<MapCommand>,
    camera: mapbox::CameraOptions,
}

impl Map {
    pub fn new<'a>(
        camera: &mapbox::CameraOptions,
        controller: impl Into<Option<&'a mut MapController>>,
    ) -> Self {
        let channel = Channel::new();
        if let Some(controller) = controller.into() {
            controller.bind(channel.clone());
        }
        Map {
            channel,
            camera: camera.clone(),
        }
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.camera == other.camera
    }
}

impl Widget for Map {
    fn mount(&self, context: &mut WidgetContext, mount: &mut MountContext) {
        let (sender, receiver) = channel();
        let mut texture = mount.register_texture(RenderMap::new(receiver));
        let mut state = MapState::new(texture, self.channel.clone(), sender);
        state.rebind();
        state.channel.send(MapCommand::JumpTo(self.camera.clone()));
        context.set_state(state);
    }

    fn update(&self, context: &mut WidgetContext, _update: &mut UpdateContext) {
        let state = context.state_mut::<MapState>().unwrap();
        state.channel = self.channel.clone();
        state.rebind();
        state.channel.send(MapCommand::JumpTo(self.camera.clone()));
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        let state = context.state_mut::<MapState>().unwrap();
        match event.get() {
            Event::Touch(touches) => {
                // pinch
                match state.pinch_gesture.update(touches) {
                    PinchGestureState::Began => {
                        state.last_scale = None;
                        return;
                    }
                    PinchGestureState::Changed { scale, velocity } => {
                        let anchor = state.pinch_gesture.center_location().unwrap();
                        let scale = 1.0 + scale - state.last_scale.replace(scale).unwrap_or(scale);
                        state.channel.send(MapCommand::ScaleBy(
                            scale,
                            Some(mapbox::ScreenCoordinate::new(anchor.x, anchor.y)),
                            None,
                        ));
                        return;
                    }
                    _ => {}
                }
                // pan
                let current_location = state.pan_gesture.location();
                if let PanGestureState::Changed(location) = state.pan_gesture.update(touches) {
                    let diff = location - current_location.unwrap_or(location);
                    state.channel.send(MapCommand::MoveBy(
                        mapbox::ScreenCoordinate::new(diff.x, diff.y),
                        None,
                    ));
                }
            }
            Event::Scroll(scroll) => {
                let delta = scroll.y;
                let mut scale = 2.0 / (1.0 + (-delta.abs() / 100.0).exp());
                // zoom out
                if delta < 0.0 && scale != 0.0 {
                    scale = 1.0 / scale;
                }
                state.channel.send(MapCommand::ScaleBy(scale, None, None));
            }
            _ => {}
        }
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state::<MapState>().unwrap();
        paint.push_texture(state.texture.id());
        paint.paint_children();
    }

    fn hit_test(&self, _: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        hit_test.become_responder()
    }
}

struct MapState {
    texture: WidgetTexture,
    channel: Channel<MapCommand>,
    sender: Sender<MapCommand>,
    pan_gesture: PanGesture,
    pinch_gesture: PinchGesture,
    last_scale: Option<f32>,
}

impl MapState {
    pub fn new(
        texture: WidgetTexture,
        channel: Channel<MapCommand>,
        sender: Sender<MapCommand>,
    ) -> Self {
        MapState {
            texture,
            channel,
            sender,
            pan_gesture: PanGesture::limit(1),
            pinch_gesture: PinchGesture::default(),
            last_scale: None,
        }
    }

    pub fn rebind(&mut self) {
        let sender = self.sender.clone();
        let mut texture = self.texture.clone();
        self.channel.bind(move |command| {
            let _ = sender.send(command);
            texture.update();
        });
    }
}

impl Drop for MapState {
    fn drop(&mut self) {
        self.channel.deactivate();
    }
}

struct RenderMap {
    receiver: Receiver<MapCommand>,
    map: Option<mapbox::Map>,
    last_size: Option<ISize>,
}

unsafe impl Send for RenderMap {}

impl RenderMap {
    pub fn new(receiver: Receiver<MapCommand>) -> Self {
        RenderMap {
            receiver,
            map: None,
            last_size: None,
        }
    }
}

impl RenderTexture for RenderMap {
    fn preroll(&mut self, preroll: &mut PrerollContext) {
        let size = preroll.size();
        let texture = preroll.texture();
        let mut pipeline = preroll.pipeline().clone();
        let mut gl = preroll.gl().clone();
        let view_port = gl.view_port();
        self.last_size = Some(size);
        self.map = mapbox::Map::new(
            mapbox::RendererFrontend::new(
                mapbox::RendererBackend::new(
                    move |name| gl.load_function(name),
                    move || mapbox::Size::new(view_port.width() as u32, view_port.height() as u32),
                    || {},
                    || {},
                    || {},
                ),
                preroll.pixel_ratio(),
                move || pipeline.invalidate_texture(texture),
            ),
            &mapbox::MapOptions::default()
                .with_size(mapbox::Size::new(size.width as u32, size.height as u32))
                .with_pixel_ratio(preroll.pixel_ratio()),
            &mapbox::ResourceOptions::default()
                .with_cache_path(Settings::mapbox_cache_path())
                .with_access_token(Settings::mapbox_access_token()),
        ).into();
    }

    fn update(&mut self, _: &mut TextureContext) {
        if let Some(map) = &mut self.map {
            while let Ok(command) = self.receiver.try_recv() {
                match command {
                    MapCommand::EaseTo(camera, animation) => {
                        map.ease_to(&camera, &animation);
                    }
                    MapCommand::MoveBy(coordinate, animation) => {
                        map.move_by(coordinate, &animation);
                    }
                    MapCommand::ScaleBy(scale, anchor, animation) => {
                        map.scale_by(scale, anchor, &animation);
                    }
                    MapCommand::JumpTo(camera) => {
                        map.jump_to(&camera);
                    }
                }
            }
        }
    }

    fn render(&mut self, render: &mut RenderContext) {
        if let Some(map) = &mut self.map {
            let size = render.size();
            if self.last_size != Some(size) {
                map.set_size(mapbox::Size::new(size.width as u32, size.height as u32));
                self.last_size = Some(size);
            }
            map.render();
        }
    }
}
