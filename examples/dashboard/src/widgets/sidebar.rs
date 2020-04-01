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

use terramach::graphics::{Color, Size};
use terramach::widgets::*;
use terramach::*;

#[derive(PartialEq, EventId)]
enum Button {
    Speed = 1,
    Earth = 2,
    Settings = 3,
    Help = 4,
    Power = 5,
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {}
    }

    fn build_button(
        &self,
        button: Button,
        event_emitter: WidgetEventEmitter,
        icon: &[u8],
        selected: bool,
    ) -> impl Widget {
        Gesture::new(
            button,
            event_emitter,
            TapGesture::default(),
            None,
            AspectRatio::new(
                1.0,
                AnimatedOpacity::new(
                    if selected { 1.0 } else { 0.3 },
                    None,
                    Decoration::new(
                        if selected {
                            Some(Color::new(0xFF3ABCF2))
                        } else {
                            None
                        },
                        None,
                        Image::from_bytes(None, None, Color::WHITE, icon),
                    ),
                ),
            ),
        )
    }
}

impl Widget for Sidebar {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(SidebarState::new())
    }

    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<SidebarState>().unwrap();
        build.add_child(Constrained::new(
            Constraints::new_tight(Size::new_unbound_height(80.0)),
            Decoration::new(
                Color::new(0xFF15191C),
                None,
                Column::default()
                    .with_child(Constrained::new_empty(Constraints::new_tight(
                        Size::new_unbound_width(140.0),
                    )))
                    .with_child(self.build_button(
                        Button::Speed,
                        build.event_emitter(),
                        include_bytes!("../../assets/icons/speedometer.png"),
                        state.selected_button == Button::Speed,
                    ))
                    .with_child(self.build_button(
                        Button::Earth,
                        build.event_emitter(),
                        include_bytes!("../../assets/icons/earth.png"),
                        state.selected_button == Button::Earth,
                    ))
                    .with_child(self.build_button(
                        Button::Settings,
                        build.event_emitter(),
                        include_bytes!("../../assets/icons/settings.png"),
                        state.selected_button == Button::Settings,
                    ))
                    .with_child(self.build_button(
                        Button::Help,
                        build.event_emitter(),
                        include_bytes!("../../assets/icons/help.png"),
                        state.selected_button == Button::Help,
                    ))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(self.build_button(
                        Button::Power,
                        build.event_emitter(),
                        include_bytes!("../../assets/icons/power.png"),
                        false,
                    ))
                    .with_child(Constrained::new_empty(Constraints::new_tight(
                        Size::new_unbound_width(40.0),
                    ))),
            ),
        ));
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Tap(id) => {
                let state = context.state_mut::<SidebarState>().unwrap();
                match id.into() {
                    Button::Speed | Button::Help | Button::Settings | Button::Earth => {
                        state.selected_button = id.into();
                        event.mark_need_build();
                    }
                    Button::Power => {
                        println!("Shutdown!");
                    }
                }
            }
            _ => {}
        }
    }
}

struct SidebarState {
    selected_button: Button,
}

impl SidebarState {
    pub fn new() -> Self {
        SidebarState {
            selected_button: Button::Earth,
        }
    }
}
