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

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Event, EventContext, EventId, HitTestContext,
    LayoutContext, MountContext, PanGesture, PartialWidget, TapGesture, TapGestureState, Touches,
    Widget, WidgetContext, WidgetEventEmitter,
};

#[derive(Clone, PartialWidget)]
pub struct Gesture {
    tap: Option<TapGesture>,
    pan: Option<PanGesture>,
    event_id: EventId,
    event_emitter: WidgetEventEmitter,
    child: BoxedWidget,
}

impl Gesture {
    pub fn new(
        event_id: impl Into<EventId>,
        event_emitter: WidgetEventEmitter,
        tap: impl Into<Option<TapGesture>>,
        pan: impl Into<Option<PanGesture>>,
        child: impl Into<BoxedWidget>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_emitter,
            tap: tap.into(),
            pan: pan.into(),
            child: child.into(),
        }
    }
}

impl PartialEq for Gesture {
    fn eq(&self, other: &Self) -> bool {
        self.tap.is_some() == other.tap.is_some()
            && self.pan.is_some() == other.pan.is_some()
            && self.event_id == other.event_id
            && self.child == other.child
    }
}

impl Widget for Gesture {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(GestureState::new(
            self.event_id,
            self.event_emitter.clone(),
            self.tap.clone(),
            self.pan.clone(),
        ));
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Touch(touches) => {
                let state = context.state_mut::<GestureState>().unwrap();
                if state.recognize(touches) {
                    event.mark_need_event();
                }
            }
            Event::Enter => {
                let state = context.state_mut::<GestureState>().unwrap();
                state.hover = true;
            }
            Event::Leave => {
                let state = context.state_mut::<GestureState>().unwrap();
                state.hover = false;
            }
            _ => {}
        }
    }

    fn hit_test(&self, _: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        hit_test.become_responder()
    }
}

struct GestureState {
    event_id: EventId,
    event_emitter: WidgetEventEmitter,
    tap: Option<TapGesture>,
    pan: Option<PanGesture>,
    hover: bool,
}

impl GestureState {
    pub fn new(
        event_id: EventId,
        event_emitter: WidgetEventEmitter,
        tap: Option<TapGesture>,
        pan: Option<PanGesture>,
    ) -> Self {
        Self {
            event_id,
            event_emitter,
            tap,
            pan,
            hover: false,
        }
    }

    pub fn recognize(&mut self, touches: &Touches) -> bool {
        if let Some(tap) = &mut self.tap {
            if tap.update(touches) == TapGestureState::Ended && self.hover {
                self.event_emitter.emit_event(Event::Tap(self.event_id));
                return true;
            }
        }
        false
    }
}
