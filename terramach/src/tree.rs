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

use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{Deref, DerefMut};
use std::panic;
use std::time::Duration;
use std::any::Any;
use std::iter::FromIterator;

use crate::*;
use crate::gpu::{Frame, SharedPipeline, RenderTexture, TextureId};
use crate::platform::Cursor;

use terramach_graphics::{Point, Size};

use time_point::TimePoint;

pub struct RenderTree {
    tree: Tree<BoxedWidget>,
    root_widget: Id,
    states: HashMap<Id, WidgetState>,
    layer_tree: LayerTree,
    need_paint: bool,
    need_build: HashSet<Id>,
    requested_frame: HashSet<Id>,
    active_timers: HashSet<Id>,
    pipeline: SharedPipeline,
    texture_ids: IndexPool,
}

impl RenderTree {
    pub fn new(
        pipeline: SharedPipeline,
        root: BoxedWidget,
    ) -> Self {
        let mut tree = Tree::new();
        let root_widget = tree.insert(root, None);
        let mut need_build = HashSet::new();
        need_build.insert(root_widget);
        RenderTree {
            root_widget,
            tree,
            need_build,
            pipeline,
            states: HashMap::new(),
            layer_tree: LayerTree::new(),
            need_paint: false,
            requested_frame: HashSet::new(),
            active_timers: HashSet::new(),
            texture_ids: IndexPool::new(),
        }
    }

    pub fn needs_frame(&self) -> bool {
        !self.requested_frame.is_empty()
    }

    pub fn invalidate(&mut self) {
        for state in self.states.values_mut() {
            state.set_need_paint(true);
            state.set_need_layout(true);
        }
    }

    fn invalidate_requests(&mut self, id: Id) {
        if let Some(state) = self.states.get(&id) {
            let context = state.context();
            if context.frame_requested() {
                self.requested_frame.insert(id);
            }
            if context.has_active_timers() {
                self.active_timers.insert(id);
            } else {
                self.active_timers.remove(&id);
            }
        }
    }

    fn emit_event_direct(&mut self, id: Id, event: Event) -> bool {
        if let Some(widget) = self.tree.node(id) {
            if let Some(state) = self.states.get_mut(&id) {
                let mut event_context = EventContext::new(event);
                widget.event(state.context_mut().deref_mut(), &mut event_context);
                if event_context.need_build() {
                    self.invalidate_build(id);
                } else if event_context.need_layout() {
                    self.invalidate_layout(id);
                } else if event_context.need_paint() {
                    self.invalidate_paint(id);
                }
                self.invalidate_requests(id);
                return event_context.need_event();
            }
        }
        false
    }

    fn emit_widget_events(&mut self, id: Id) {
        if let Some(state) = self.states.get_mut(&id) {
            if let Some(events) = state.events_mut().poll() {
                for event in events {
                    self.emit_event_direct(id, event);
                }
            }
            if let Some(parent) = self.tree.parent(id) {
                self.emit_widget_events(parent);
            }
        }
    }

    pub fn emit_event(&mut self, id: impl Into<Option<Id>>, event: Event) {
        if let Some(id) = id.into() {
            if self.emit_event_direct(id, event) {
                if let Some(parent) = self.tree.parent(id) {
                    self.emit_widget_events(parent);
                }
            }
        } else {
            match event {
                Event::Frame(timestamp) => self.emit_frame_event(timestamp),
                _ => panic!("{:?} cannot be issued to all widgets", event),
            }
        }
    }

    fn emit_frame_event(&mut self, timestamp: TimePoint) {
        if self.requested_frame.is_empty() {
            return;
        }
        let widgets: Vec<Id> = self.requested_frame.drain().collect();
        for id in widgets {
            if let Some(state) = self.states.get_mut(&id) {
                state.context_mut().frame_requested = false;
                self.emit_event(id, Event::Frame(timestamp));
            }
        }
    }

    fn flush_pending_timers(&mut self) {
        if self.active_timers.is_empty() {
            return;
        }
        let widgets: Vec<Id> = self.active_timers.iter().copied().collect();
        for id in widgets {
            self.flush_pending_widget_timers(id);
        }
    }

    fn flush_pending_widget_timers(&mut self, id: Id) -> Option<()> {
        let expired_timers: Vec<usize> = {
            let state = self.states.get(&id)?;
            let mut context = state.context_mut();
            let timers = context.timers.as_mut()?;
            let expired_timers: HashSet<Id> = HashSet::from_iter(timers.fire()?);
            context.timers_ids.iter()
                .flat_map(|(timer_id, timer)| {
                    if expired_timers.contains(timer) {
                        Some(*timer_id)
                    } else {
                        None
                    }
                })
                .collect()
        };
        for timer_id in expired_timers {
            self.emit_event(id, Event::Timer(timer_id));
        }
        Some(())
    }

    pub fn request_next_timer_time(&self) -> Option<Duration> {
        let mut time = None;
        for id in &self.active_timers {
            let context = self.states.get(id)?.context();
            let timers = context.timers.as_ref()?;
            if let Some(next_time) = timers.next_fire_time() {
                if let Some(time_) = time {
                    if time_ > next_time {
                        time = Some(next_time);
                    }
                } else {
                    time = Some(next_time);
                }
            }
        }
        time
    }

    pub fn hit_test(
        &self,
        id: impl Into<Option<Id>>,
        location: impl Into<Point>,
    ) -> Option<EventResponder> {
        self.hit_test_widget(id.into().unwrap_or(self.root_widget), location.into())
    }

    fn hit_test_widget(&self, id: Id, location: Point) -> Option<EventResponder> {
        let widget = self.tree.node(id)?;
        let state = self.states.get(&id)?;

        let size = if let Some(size) = state.size() {
            size
        } else {
            debug_assert!(false, "Widget needs to be laid out before hit testing!");
            return None;
        };

        let offset = state.offset().unwrap_or_default();
        let location = location - offset;

        let mut hit_test = HitTestContext::new(size, location);
        let mut context = state.context_mut();
        if !widget.hit_test(context.deref_mut(), &mut hit_test) {
            return None;
        }

        if !hit_test.absorbed() {
            if let Some(children) = self.tree.children(id) {
                let location = hit_test.transformation().map_point(location);
                for child in children {
                    if let Some(mut responder) = self.hit_test_widget(*child, location) {
                        hit_test.push_offset(offset);
                        responder.push_transformation(*hit_test.transformation());
                        return Some(responder);
                    }
                }
            }
        }

        if hit_test.requested_become_responder() {
            return Some(EventResponder::new(
                id,
                *hit_test.transformation(),
                context.cursor(),
            ));
        }

        None
    }

    fn invalidate_paint(&mut self, id: impl Into<Option<Id>>) {
        let id = id.into().unwrap_or(self.root_widget);
        if let Some(state) = self.states.get_mut(&id) {
            state.set_need_paint(true);
            self.need_paint = true;
        }
    }

    fn invalidate_build(&mut self, id: impl Into<Option<Id>>) {
        let id = id.into().unwrap_or(self.root_widget);
        if let Some(state) = self.states.get_mut(&id) {
            state.set_need_build(true);
            self.need_build.insert(id);
        }
    }

    fn invalidate_layout(&mut self, id: impl Into<Option<Id>>) {
        let id = id.into().unwrap_or(self.root_widget);
        if let Some(state) = self.states.get_mut(&id) {
            state.set_need_layout(true);
            state.set_need_paint(true);
            self.need_paint = true;
            if let Some(parent) = self.tree.parent(id) {
                self.invalidate_layout(parent);
            }
        }
    }

    fn build_widget(&mut self, id: Id) {
        if let Some(state) = self.states.get(&id) {
            if !state.need_build() {
                return;
            }
        } else {
            self.states.insert(id, WidgetState::new());
        }
        let widget = self.tree.node(id).unwrap();
        let state = self.states.get_mut(&id).unwrap();
        if state.mounted() {
            let mut update = UpdateContext::new(id, &self.tree);
            widget.update(state.context_mut().deref_mut(), &mut update);
        } else {
            state.set_mounted(true);
            let mut mount = MountContext::new(
                id,
                &self.tree,
                &mut self.texture_ids,
                &mut self.pipeline,
            );
            widget.mount(state.context_mut().deref_mut(), &mut mount);
            state.set_texture(mount.texture);
        }

        let event_emitter = state.events_mut().emitter();
        let mut build = BuildContext::new(event_emitter);
        widget.build(state.context_mut().deref_mut(), &mut build);
        state.set_need_build(false);
        state.set_need_layout(true);
        state.set_need_paint(true);

        self.need_paint = true;
        self.invalidate_requests(id);

        let old_children = self.tree.children(id).cloned();
        let mut index = 0;
        for new_child in build.children {
            let old_child_id = if let Some(old_children) = &old_children {
                old_children.get(index)
            } else {
                None
            };
            index += 1;

            if let Some(old_child_id) = old_child_id {
                let old_child = self.tree.node(*old_child_id).unwrap();
                if new_child.same(old_child) {
                    self.tree.replace(*old_child_id, new_child);
                    let state = self.states.get_mut(old_child_id).unwrap();
                    state.set_need_build(true);
                    self.build_widget(*old_child_id);
                    continue;
                }
                self.tree.remove(*old_child_id);
                if let Some(state) = self.states.remove(old_child_id) {
                    if let Some(texture) = state.texture() {
                        self.pipeline.unregister_texture(texture);
                        self.texture_ids.give(texture);
                    }
                }
            }

            let child_id = self.tree.insert(new_child, id);
            self.build_widget(child_id);
        }
    }

    fn layout_widget(
        &self,
        id: Id,
        constraints: Constraints,
        results: &mut HashMap<Id, (Size, Option<Point>)>,
    ) -> Option<Size> {
        let state = self.states.get(&id)?;
        if !state.need_layout() {
            if let Some(size) = state.size() {
                results.insert(id, (size, None));
                return Some(size);
            }
        }

        let child_id = |index: usize| -> Option<Id> { self.tree.children(id)?.get(index).copied() };
        let mut layout_child = |id: Id, child_constraints: &Constraints| -> Option<Size> {
            self.layout_widget(id, *child_constraints, results)
        };
        let mut layout = LayoutContext::new(
            constraints,
            self.tree.child_count(id),
            &child_id,
            &mut layout_child,
        );

        let widget = self.tree.node(id)?;
        let size = widget.layout(state.context_mut().deref_mut(), &mut layout);
        for (id, offset) in layout.child_offsets {
            results.get_mut(&id).expect("A child is not laid out").1 = Some(offset);
        }
        results.insert(id, (size, None));
        Some(size)
    }

    pub fn render(&mut self, size: impl Into<Size>) {
        self.flush_pending_timers();

        if !self.need_build.is_empty() {
            for id in self.need_build.drain().collect::<Vec<Id>>() {
                self.invalidate_layout(id);
                self.build_widget(id);
            }
        }

        debug_assert!(
            self.tree.len() == self.states.len(),
            "Render tree is corrupted after a build"
        );

        let mut results = HashMap::new();
        self.layout_widget(
            self.root_widget,
            Constraints::new_tight(size.into()),
            &mut results,
        );

        for (id, (size, offset)) in results {
            if let Some(state) = self.states.get_mut(&id) {
                state.set_size(size);
                state.set_offset(offset);
            }
        }

        if self.need_paint {
            self.need_paint = false;
            self.paint_widget(self.root_widget);
            self.pipeline.submit_frame(Frame::new(&self.layer_tree));
        }
    }

    fn widget_parent_layer(&self, id: Id) -> Option<Id> {
        let parent_id = self.tree.parent(id)?;
        if let Some(layer_id) = self.layer_tree.parent_key_layer(parent_id) {
            Some(layer_id)
        } else {
            self.widget_parent_layer(parent_id)
        }
    }

    fn paint_widget(&mut self, id: Id) {
        if let Some(state) = self.states.get(&id) {
            let size = if let Some(size) = state.size() {
                size
            } else {
                debug_assert!(false, "Widget needs to be laid out before painting!");
                return;
            };

            self.layer_tree.drop_key_layer(id);

            if state.need_paint() {
                let mut paint = PaintContext::new(size);
                if let Some(widget) = self.tree.node(id) {
                    widget.paint(state.context_mut().deref_mut(), &mut paint);
                    self.invalidate_requests(id);
                }
                let state = self.states.get_mut(&id).unwrap();
                state.set_need_paint(false);
                state.set_layer(paint.layers().cloned());
                state.set_leaf_layer(paint.leaf_layers().cloned());
            }

            let mut parent_layer_id = self.widget_parent_layer(id);

            let state = self.states.get(&id).unwrap();
            if let Some(offset) = state.offset() {
                let id = self.layer_tree.insert(
                    id,
                    size,
                    Box::new(OffsetLayer::new(offset)),
                    parent_layer_id,
                );
                parent_layer_id = Some(id);
            }

            if let Some(layer) = state.layer() {
                self.layer_tree.insert(id, size, layer.clone_boxed(), parent_layer_id);
            }

            if let Some(children) = self.tree.children(id).cloned() {
                for child in children {
                    self.paint_widget(child);
                }
            }

            let state = self.states.get(&id).unwrap();
            if let Some(layer) = state.leaf_layer() {
                self.layer_tree
                    .insert_leaf(id, size, layer.clone_boxed(), parent_layer_id);
            }
        }
    }
}

#[derive(Clone)]
pub struct WidgetTexture {
    texture: TextureId,
    pipeline: SharedPipeline,
}

impl WidgetTexture {
    fn new(pipeline: &SharedPipeline, texture: TextureId) -> Self {
        WidgetTexture {
            pipeline: pipeline.clone(),
            texture,
        }
    }

    pub fn id(&self) -> TextureId {
        self.texture
    }

    pub fn update(&mut self) {
        self.pipeline.update_texture(self.texture);
    }

    pub fn invalidate(&mut self) {
        self.pipeline.invalidate_texture(self.texture);
    }
}

pub struct MountContext<'a> {
    id: Id,
    tree: &'a Tree<BoxedWidget>,
    texture_ids: &'a mut IndexPool,
    texture: Option<Id>,
    pipeline: &'a mut SharedPipeline,
}

impl<'a> MountContext<'a> {
    pub fn new(
        id: Id,
        tree: &'a Tree<BoxedWidget>,
        texture_ids: &'a mut IndexPool,
        pipeline: &'a mut SharedPipeline,
    ) -> Self {
        MountContext {
            id,
            texture_ids,
            tree,
            texture: None,
            pipeline,
        }
    }

    pub fn register_texture<T: 'static + RenderTexture + Send>(&mut self, texture: T) -> WidgetTexture {
        debug_assert!(self.texture.is_none(), "Cannot register multiple external textures");
        let id = self.texture_ids.take();
        self.texture = Some(id);
        self.pipeline.register_texture(id, texture);
        WidgetTexture::new(self.pipeline, id)
    }

    pub fn ancestor_widget<T: 'static + Widget>(&self) -> Option<&T> {
        let mut ids = VecDeque::new();
        if let Some(parent) = self.tree.parent(self.id) {
            ids.push_back(parent);
        }
        while let Some(id) = ids.pop_front() {
            if let Some(widget) = self.tree.node(id) {
                if let Some(widget) = widget.as_any().downcast_ref::<T>() {
                    return Some(widget);
                }
            }
            if let Some(parent) = self.tree.parent(id) {
                ids.push_back(parent);
            }
        }
        None
    }
}

pub struct UpdateContext<'a> {
    id: Id,
    tree: &'a Tree<BoxedWidget>,
}

impl<'a> UpdateContext<'a> {
    pub fn new(id: Id, tree: &'a Tree<BoxedWidget>) -> Self {
        UpdateContext { id, tree }
    }

    pub fn ancestor_widget<T: 'static + Widget>(&self) -> Option<&T> {
        let mut ids = VecDeque::new();
        if let Some(parent) = self.tree.parent(self.id) {
            ids.push_back(parent);
        }
        while let Some(id) = ids.pop_front() {
            if let Some(widget) = self.tree.node(id) {
                if let Some(widget) = widget.as_any().downcast_ref::<T>() {
                    return Some(widget);
                }
            }
            if let Some(parent) = self.tree.parent(id) {
                ids.push_back(parent);
            }
        }
        None
    }
}

pub struct BuildContext {
    children: Vec<BoxedWidget>,
    event_emitter: WidgetEventEmitter,
}

impl BuildContext {
    pub fn new(event_emitter: WidgetEventEmitter) -> Self {
        BuildContext {
            children: Vec::new(),
            event_emitter,
        }
    }

    pub fn add_child(&mut self, widget: impl Into<BoxedWidget>) -> &mut Self {
        self.children.push(widget.into());
        self
    }

    pub fn event_emitter(&self) -> WidgetEventEmitter {
        self.event_emitter.clone()
    }
}
