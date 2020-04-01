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

use terramach::*;
use terramach::graphics::*;
use terramach::widgets::*;

#[derive(Default, Clone, PartialEq, PartialWidget)]
pub struct Counter {}

#[derive(Default)]
struct CounterState {
    counter: usize,
}

impl Widget for Counter {
    // prepare state for the counter
    fn mount(&self, context: &mut WidgetContext, mount: &mut MountContext) {
        context.set_state(CounterState::default());
    }

    // build a counter widget with tap gesture and white background
    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<CounterState>().unwrap();
        build.add_child(
            Gesture::new(
                0 /* unique event id within a scope of the widget */,
                build.event_emitter(),
                TapGesture::default(),
                None,
                Decoration::new(
                    Color::WHITE,
                    None,
                    Align::new(
                        Alignment::center(),
                        Text::new_text(format!("Counter {}", state.counter).as_str()),
                    ),
                ),
            ),
        );
    }

    // handle a single tap
    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        if let Event::Tap(_) = event.get() {
            let state = context.state_mut::<CounterState>().unwrap();
            state.counter += 1;
            event.mark_need_build();
        }
    }
}
