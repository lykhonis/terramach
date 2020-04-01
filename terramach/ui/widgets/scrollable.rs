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

use std::time::Duration;

use crate::{
    Animation, AnimationCurves, Animator, BoxedWidget, BuildContext, Constraints, Event,
    EventContext, HitTestContext, Id, LayoutContext, MeasuredSize, MountContext, PaintContext,
    PanGesture, PanGestureState, PartialWidget, Timer, Tween, Widget, WidgetContext,
};

use terramach_graphics::{Color4f, Paint, Point, Rect, Size};

#[derive(Clone, PartialEq)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Vertical
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Scrollable {
    direction: ScrollDirection,
    child: BoxedWidget,
}

impl Scrollable {
    const SCROLLBAR_THICKNESS: f32 = 6.0;
    const SCROLLBAR_MARGIN: f32 = 4.0;
    const SCROLLBAR_OPACITY_ACTIVE: f32 = 0.35;
    const SCROLLBAR_OPACITY_INACTIVE: f32 = 0.2;
    const SCROLLBAR_TIMEOUT: Duration = Duration::from_millis(1000);
    const SCROLLBAR_ANIMATION: Duration = Duration::from_millis(750);
    const SCROLLBAR_DEACTIVATE: usize = 1;

    pub fn new(direction: ScrollDirection, child: impl Into<BoxedWidget>) -> Self {
        Scrollable {
            direction,
            child: child.into(),
        }
    }
}

impl Widget for Scrollable {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(ScrollableState::new());
    }

    fn layout(&self, context: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let size = layout.constraints().maximum_size();
        let child_constraints = Constraints::new_loose(match self.direction {
            ScrollDirection::Vertical => Size::new_unbound_height(size.width),
            ScrollDirection::Horizontal => Size::new_unbound_width(size.height),
        });
        let child_size = layout
            .layout_child(0, &child_constraints)
            .unwrap()
            .constrain(&child_constraints);
        let state = context.state_mut::<ScrollableState>().unwrap();
        state.content_size = Some(child_size);
        state.size = Some(size);
        state.clamp_scroll_offset(state.offset);
        size
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Touch(touches) => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                match state.gesture.update(touches) {
                    PanGestureState::Began(location) => {
                        state.location = location;
                        state.scrollbar_state = ScrollbarState::Active;
                        state.scrollbar_animation = None;
                        event.mark_need_paint();
                    }
                    PanGestureState::Changed(location) => {
                        let delta = state.location - location;
                        state.location = location;
                        if state.scroll(delta) {
                            event.mark_need_paint();
                        }
                    }
                    PanGestureState::Ended => {
                        if !state.hover {
                            state.scrollbar_state = ScrollbarState::Inactive;
                            event.mark_need_paint();
                            context.schedule_timer(
                                Scrollable::SCROLLBAR_DEACTIVATE,
                                Timer::new(Scrollable::SCROLLBAR_TIMEOUT, None),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Event::Scroll(delta) => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                if state.scroll(-*delta) {
                    event.mark_need_paint();
                }
            }
            Event::Enter => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                state.hover = true;
                state.scrollbar_state = ScrollbarState::Active;
                state.scrollbar_animation = None;
                event.mark_need_paint();
            }
            Event::Leave => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                state.hover = false;
                if !state.gesture.is_active() {
                    state.scrollbar_state = ScrollbarState::Inactive;
                    event.mark_need_paint();
                    context.schedule_timer(
                        Scrollable::SCROLLBAR_DEACTIVATE,
                        Timer::new(Scrollable::SCROLLBAR_TIMEOUT, None),
                    );
                }
            }
            Event::Timer(timer) => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                if timer == &Scrollable::SCROLLBAR_DEACTIVATE && !state.hover && !state.gesture.is_active() {
                    state.scrollbar_animation = Scrollable::SCROLLBAR_ANIMATION
                        .animate(Tween::new(1.0, 0.0))
                        .with(AnimationCurves::ease_out())
                        .into();
                    context.request_frame();
                }
            }
            Event::Frame(_) => {
                let state = context.state_mut::<ScrollableState>().unwrap();
                if let Some(animation) = &mut state.scrollbar_animation {
                    if animation.advance() {
                        context.request_frame();
                    } else {
                        state.scrollbar_animation = None;
                        state.scrollbar_state = ScrollbarState::Invisible;
                    }
                    event.mark_need_paint();
                }
            }
            _ => {}
        }
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state::<ScrollableState>().unwrap();
        paint.push_clip_rect(Rect::from_size(paint.size()));
        paint.push_offset(-state.offset);
        paint.paint_children();

        // scroll bar
        if state.scrollbar_state == ScrollbarState::Invisible {
            return;
        }
        let scrollbar_area = paint
            .size()
            .deflate((Scrollable::SCROLLBAR_MARGIN * 2.0, Scrollable::SCROLLBAR_MARGIN * 2.0));
        if let Some(scrollbar_bounds) = state.scrollbar_bounds(scrollbar_area) {
            paint.push_offset((scrollbar_bounds.left(), scrollbar_bounds.top()));
            let background_paint = Paint::new(
                Color4f::new(
                    0.0,
                    0.0,
                    0.0,
                    state.scrollbar_animation()
                        * match state.scrollbar_state {
                        ScrollbarState::Invisible => 0.0,
                        ScrollbarState::Inactive => Scrollable::SCROLLBAR_OPACITY_INACTIVE,
                        ScrollbarState::Active => Scrollable::SCROLLBAR_OPACITY_ACTIVE,
                    },
                ),
                None,
            );
            let canvas = paint.canvas();
            canvas.draw_round_rect(
                match self.direction {
                    ScrollDirection::Vertical => Rect::from_xywh(
                        Scrollable::SCROLLBAR_MARGIN + scrollbar_area.width - Scrollable::SCROLLBAR_THICKNESS,
                        Scrollable::SCROLLBAR_MARGIN,
                        Scrollable::SCROLLBAR_THICKNESS,
                        scrollbar_bounds.height(),
                    ),
                    ScrollDirection::Horizontal => Rect::from_xywh(
                        Scrollable::SCROLLBAR_MARGIN,
                        Scrollable::SCROLLBAR_MARGIN + scrollbar_area.height - Scrollable::SCROLLBAR_THICKNESS,
                        scrollbar_bounds.width(),
                        Scrollable::SCROLLBAR_THICKNESS,
                    ),
                },
                Scrollable::SCROLLBAR_THICKNESS,
                Scrollable::SCROLLBAR_THICKNESS,
                &background_paint,
            );
        }
    }

    fn hit_test(&self, context: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        let state = context.state::<ScrollableState>().unwrap();
        hit_test.push_offset(-state.offset);
        hit_test.become_responder()
    }
}

#[derive(PartialEq)]
enum ScrollbarState {
    Invisible,
    Inactive,
    Active,
}

struct ScrollableState {
    size: Option<Size>,
    content_size: Option<Size>,
    location: Point,
    offset: Point,
    scrollbar_state: ScrollbarState,
    gesture: PanGesture,
    hover: bool,
    scrollbar_animation: Option<Animation<f32>>,
}

impl ScrollableState {
    pub fn new() -> Self {
        ScrollableState {
            size: None,
            content_size: None,
            location: Point::default(),
            offset: Point::default(),
            scrollbar_state: ScrollbarState::Invisible,
            gesture: PanGesture::new(1, 1),
            hover: false,
            scrollbar_animation: None,
        }
    }

    pub fn scrollbar_animation(&self) -> f32 {
        self.scrollbar_animation
            .as_ref()
            .map(|a| a.value())
            .unwrap_or(1.0)
    }

    pub fn scrollbar_bounds(&self, area: Size) -> Option<Rect> {
        let content_size = self.content_size.as_ref()?;
        let size = self.size.as_ref()?;
        if content_size.width <= size.width && content_size.height <= size.height {
            return None;
        }
        Rect::from_xywh(
            area.width * self.offset.x / content_size.width,
            area.height * self.offset.y / content_size.height,
            area.width * size.width / content_size.width,
            area.height * size.height / content_size.height,
        ).into()
    }

    pub fn scrollable_size(&self) -> Option<Size> {
        let content_size = self.content_size.as_ref()?;
        let size = self.size.as_ref()?;
        Size::new(
            (content_size.width - size.width).max(0.0),
            (content_size.height - size.height).max(0.0),
        ).into()
    }

    pub fn scroll(&mut self, delta: impl Into<Point>) -> bool {
        let delta = delta.into();
        self.clamp_scroll_offset(self.offset + delta)
    }

    pub fn clamp_scroll_offset(&mut self, offset: impl Into<Point>) -> bool {
        if let Some(size) = self.scrollable_size() {
            let offset = offset.into();
            let offset = Point::new(
                offset.x.min(size.width).max(0.0),
                offset.y.min(size.height).max(0.0),
            );
            if self.offset == offset {
                false
            } else {
                self.offset = offset;
                true
            }
        } else {
            false
        }
    }
}
