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

use crate::{Animation, AnimationCurves, Animator, AnyWidget, BoxedWidget, BuildContext, Event, EventContext, Fit, HitTestContext, LayoutContext, MeasuredSize, MountContext, PaintContext, PartialWidget, Tween, Widget, WidgetContext, UpdateContext};

#[derive(Clone, PartialEq, PartialWidget)]
pub struct AnimatedOpacity {
    opacity: f32,
    duration: Duration,
    child: BoxedWidget,
}

impl AnimatedOpacity {
    pub fn new(
        opacity: impl Into<Option<f32>>,
        duration: impl Into<Option<Duration>>,
        child: impl Into<BoxedWidget>,
    ) -> Self {
        AnimatedOpacity {
            opacity: opacity.into().unwrap_or(1.0),
            duration: duration.into().unwrap_or(Duration::from_millis(350)),
            child: child.into(),
        }
    }
}

impl Widget for AnimatedOpacity {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(AnimatedOpacityState::new(self.opacity))
    }

    fn update(&self, context: &mut WidgetContext, _: &mut UpdateContext) {
        let state = context.state_mut::<AnimatedOpacityState>().unwrap();
        if self.opacity != state.opacity {
            state.animation = self
                .duration
                .animate(Tween::new(state.animation(), self.opacity))
                .with(AnimationCurves::ease_out())
                .into();
            state.opacity = self.opacity;
            context.request_frame();
        }
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Frame(_) => {
                let state = context.state_mut::<AnimatedOpacityState>().unwrap();
                if let Some(animation) = &mut state.animation {
                    if animation.advance() {
                        context.request_frame();
                    } else {
                        state.animation = None;
                    }
                    event.mark_need_paint();
                }
            }
            _ => {}
        }
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state::<AnimatedOpacityState>().unwrap();
        paint.push_opacity(state.animation());
        paint.paint_children();
    }
}

struct AnimatedOpacityState {
    opacity: f32,
    animation: Option<Animation<f32>>,
}

impl AnimatedOpacityState {
    pub fn new(opacity: f32) -> Self {
        AnimatedOpacityState {
            opacity,
            animation: None,
        }
    }

    pub fn animation(&self) -> f32 {
        self.animation
            .as_ref()
            .map(|a| a.value())
            .unwrap_or(self.opacity)
    }
}
