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

use std::any::Any;
use std::cell::{RefCell, RefMut, Ref};
use std::collections::HashMap;
use std::rc::Rc;

use crate::{BuildContext, EventContext, HitTestContext, Id, LayoutContext, MeasuredSize, MountContext, PaintContext, Timer, Timers, UpdateContext, ContainerLayer, WidgetEvents};
use crate::platform::Cursor;

use terramach_graphics::{Size, Point};

pub type AnyWidget = dyn Any;

pub type BoxedWidget = Box<dyn Widget>;

pub trait PartialWidget {
    fn as_any(&self) -> &AnyWidget;

    fn clone_boxed(&self) -> BoxedWidget;

    fn same(&self, other: &BoxedWidget) -> bool {
        self.as_any().type_id() == other.as_any().type_id()
    }

    fn same_content(&self, other: &BoxedWidget) -> bool;
}

pub trait Widget: PartialWidget {
    fn mount(&self, _context: &mut WidgetContext, _mount: &mut MountContext) {}

    fn update(&self, _context: &mut WidgetContext, _update: &mut UpdateContext) {}

    fn layout(&self, _context: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let mut size = Size::new_empty();
        let child_constraints = *layout.constraints();
        for child in 0..layout.child_count() {
            if let Some(child_size) = layout.layout_child(child, &child_constraints) {
                let child_size = child_size.constrain(&child_constraints);
                if size.width < child_size.width {
                    size.width = child_size.width;
                }
                if size.height < child_size.height {
                    size.height = child_size.height;
                }
            }
        }
        size.constrain(layout.constraints())
    }

    fn build(&self, _context: &mut WidgetContext, _build: &mut BuildContext) {}

    fn event(&self, _context: &mut WidgetContext, _event: &mut EventContext) {}

    fn paint(&self, _context: &mut WidgetContext, paint: &mut PaintContext) {
        paint.paint_children();
    }

    fn hit_test(&self, _context: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        hit_test.in_bounds()
    }
}

impl PartialEq<&BoxedWidget> for BoxedWidget {
    fn eq(&self, other: &&BoxedWidget) -> bool {
        self.same_content(other)
    }
}

impl PartialEq<BoxedWidget> for BoxedWidget {
    fn eq(&self, other: &BoxedWidget) -> bool {
        self.same_content(other)
    }
}

impl Clone for BoxedWidget {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl<T: 'static + Widget> From<T> for BoxedWidget {
    fn from(widget: T) -> Self {
        Box::new(widget)
    }
}

pub struct WidgetContext {
    state: Option<Box<dyn Any>>,
    cursor: Option<Cursor>,
    pub(crate) timers_ids: HashMap<usize, Id>,
    pub(crate) frame_requested: bool,
    pub(crate) timers: Option<Timers>,
}

impl WidgetContext {
    pub fn new() -> Self {
        WidgetContext {
            state: None,
            frame_requested: false,
            timers: None,
            timers_ids: HashMap::new(),
            cursor: None,
        }
    }

    pub fn set_state<T: 'static>(&mut self, state: impl Into<Option<T>>) {
        match state.into() {
            None => self.state = None,
            Some(state) => self.state = Some(Box::new(state)),
        }
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.state.as_ref()?.downcast_ref::<T>()
    }

    pub fn state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.as_mut()?.downcast_mut::<T>()
    }

    pub fn frame_requested(&self) -> bool {
        self.frame_requested
    }

    pub fn request_frame(&mut self) {
        self.frame_requested = true;
    }

    pub fn schedule_timer(&mut self, id: usize, timer: Timer) {
        let timers = self.timers.get_or_insert_with(|| Timers::new());
        let timer_id = timers.add(timer);
        let previous_timer = self.timers_ids.insert(id, timer_id);
        if let Some(previous_timer) = previous_timer {
            if previous_timer != timer_id {
                timers.remove(previous_timer);
            }
        }
    }

    pub fn cancel_timer(&mut self, id: usize) {
        if let Some(id) = self.timers_ids.remove(&id) {
            if let Some(timers) = &mut self.timers {
                timers.remove(id);
            }
        }
    }

    pub fn cancel_all_timers(&mut self) {
        self.timers_ids.clear();
        self.timers = None;
    }

    pub fn has_active_timers(&self) -> bool {
        if let Some(timers) = &self.timers {
            !timers.is_empty()
        } else {
            false
        }
    }

    pub fn set_cursor(&mut self, cursor: impl Into<Option<Cursor>>) {
        self.cursor = cursor.into();
    }

    pub fn cursor(&self) -> Option<Cursor> {
        self.cursor
    }
}

pub(crate) struct WidgetState {
    context: RefCell<WidgetContext>,
    offset: Option<Point>,
    size: Option<Size>,
    mounted: bool,
    need_paint: bool,
    need_layout: bool,
    need_build: bool,
    layer: Option<ContainerLayer>,
    leaf_layer: Option<ContainerLayer>,
    events: WidgetEvents,
    texture: Option<Id>,
}

impl WidgetState {
    pub fn new() -> WidgetState {
        WidgetState {
            context: RefCell::new(WidgetContext::new()),
            offset: None,
            size: None,
            mounted: false,
            need_layout: true,
            need_paint: true,
            need_build: true,
            layer: None,
            leaf_layer: None,
            texture: None,
            events: WidgetEvents::new(),
        }
    }

    pub fn texture(&self) -> Option<Id> {
        self.texture
    }

    pub fn set_texture(&mut self, id: impl Into<Option<Id>>) {
        self.texture = id.into();
    }

    pub fn events(&self) -> &WidgetEvents {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut WidgetEvents {
        &mut self.events
    }

    pub fn layer(&self) -> Option<&ContainerLayer> {
        self.layer.as_ref()
    }

    pub fn leaf_layer(&self) -> Option<&ContainerLayer> {
        self.leaf_layer.as_ref()
    }

    pub fn set_layer(&mut self, layer: impl Into<Option<ContainerLayer>>) {
        self.layer = layer.into();
    }

    pub fn set_leaf_layer(&mut self, layer: impl Into<Option<ContainerLayer>>) {
        self.leaf_layer = layer.into();
    }

    pub fn mounted(&self) -> bool {
        self.mounted
    }

    pub fn set_mounted(&mut self, mounted: bool) {
        self.mounted = mounted;
    }

    pub fn need_build(&self) -> bool {
        self.need_build
    }

    pub fn need_layout(&self) -> bool {
        self.need_layout
    }

    pub fn need_paint(&self) -> bool {
        self.need_paint
    }

    pub fn set_need_paint(&mut self, need_paint: bool) {
        self.need_paint = need_paint;
    }

    pub fn set_need_layout(&mut self, need_layout: bool) {
        self.need_layout = need_layout;
    }

    pub fn set_need_build(&mut self, need_build: bool) {
        self.need_build = need_build;
    }

    pub fn set_size(&mut self, size: impl Into<Option<Size>>) {
        self.need_layout = false;
        self.size = size.into();
    }

    pub fn set_offset(&mut self, offset: impl Into<Option<Point>>) {
        self.offset = offset.into();
    }

    pub fn context(&self) -> Ref<WidgetContext> {
        self.context.borrow()
    }

    pub fn context_mut(&self) -> RefMut<WidgetContext> {
        self.context.borrow_mut()
    }

    pub fn offset(&self) -> Option<Point> {
        self.offset
    }

    pub fn size(&self) -> Option<Size> {
        self.size
    }
}

struct ChannelInner<T> {
    active: bool,
    send: Option<Box<dyn FnMut(T)>>,
}

#[derive(Clone)]
pub struct Channel<T> {
    inner: Rc<RefCell<ChannelInner<T>>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel {
            inner: Rc::new(RefCell::new(ChannelInner {
                active: true,
                send: None,
            })),
        }
    }

    pub fn is_active(&self) -> bool {
        self.inner.borrow_mut().active
    }

    pub fn bind<F>(&mut self, f: F) where F: 'static + FnMut(T) {
        self.inner.borrow_mut().send.replace(Box::new(f));
    }

    pub fn send(&mut self, command: T) {
        let mut inner = self.inner.borrow_mut();
        debug_assert!(inner.send.is_some());
        if let Some(send) = &mut inner.send {
            (send)(command);
        }
    }

    pub fn deactivate(&mut self) {
        self.inner.borrow_mut().active = false;
    }
}
