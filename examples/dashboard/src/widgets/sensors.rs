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
use terramach::graphics::*;
use terramach::widgets::*;
use terramach::*;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Sensor {
    level: f32,
}

impl Sensor {
    pub fn new(level: f32) -> Self {
        Self { level }
    }
}

impl Widget for Sensor {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(SensorState::new());
        let state = context.state_mut::<SensorState>().unwrap();
        state.animation = Duration::from_secs(20)
            .animate(Tween::new(0.0, self.level))
            .with(AnimationCurves::ease_out())
            .into();
        context.request_frame();
    }

    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<SensorState>().unwrap();
        build.add_child(Constrained::new(
            Constraints::new_loose(Size::new_unbound_height(3.0)),
            Stack::new()
                .with_child(Decoration::new_empty(Color::new(0xFF353D41), None))
                .with_child(Align::new(
                    Alignment::bottom_center(),
                    Fractional::new(
                        (1.0, state.animation().unwrap_or_default()),
                        Decoration::new_empty(Color::new(0xFF3ABCF2), BorderRadius::new_all(3.0)),
                    ),
                )),
        ));
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Frame(_) => {
                let state = context.state_mut::<SensorState>().unwrap();
                if let Some(animation) = &mut state.animation {
                    if animation.advance() {
                        context.request_frame();
                    }
                    event.mark_need_build();
                }
            }
            _ => {}
        }
    }
}

struct SensorState {
    animation: Option<Animation<f32>>,
}

impl SensorState {
    pub fn new() -> Self {
        Self { animation: None }
    }

    pub fn animation(&self) -> Option<f32> {
        self.animation.as_ref()?.value().into()
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Sensors {}

impl Sensors {
    pub fn new() -> Self {
        Self {}
    }

    fn build_sensor_data(&self, title: impl AsRef<str>) -> impl Widget {
        Column::default()
            .with_child(Text::new(
                title,
                TextStyle::default().with_color(Color::new(0xFF637079)),
            ))
            .with_child(Constrained::new_empty(Constraints::new_tight(
                Size::new_unbound_width(30.0),
            )))
            .with_flex_child(
                1,
                Row::default()
                    .with_child(Sensor::new(0.5))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.7))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.6))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.8))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.4))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.3))
                    .with_flex_child(
                        1,
                        Constrained::new_empty(Constraints::new_loose(Size::new_unbound())),
                    )
                    .with_child(Sensor::new(0.5)),
            )
    }
}

impl Widget for Sensors {
    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(Constrained::new(
            Constraints::new_loose(Size::new_unbound_width(240.0)),
            Decoration::new(
                Color::new(0xFF242C30),
                BorderRadius::new_all(5.0),
                Padding::new_all(
                    20.0,
                    Column::default()
                        .with_child(
                            Row::default()
                                .with_child(Text::new(
                                    "Sensor Data",
                                    TextStyle::default().with_font_style(FontStyle::new(
                                        FontStyleWeight::MEDIUM,
                                        FontStyleWidth::NORMAL,
                                        FontStyleSlant::Upright,
                                    )),
                                ))
                                .with_flex_child(
                                    1,
                                    Constrained::new_empty(Constraints::new_loose(
                                        Size::new_unbound(),
                                    )),
                                )
                                .with_child(Text::new(
                                    "LAST 7 DAYS",
                                    TextStyle::default()
                                        .with_font_style(FontStyle::new(
                                            FontStyleWeight::MEDIUM,
                                            FontStyleWidth::NORMAL,
                                            FontStyleSlant::Upright,
                                        ))
                                        .with_color(Color::new(0xFF637079)),
                                )),
                        )
                        .with_child(Padding::new_vertical(
                            20.0,
                            Constrained::new(
                                Constraints::new_tight(Size::new_unbound_width(1.0)),
                                Decoration::new_empty(Color::new(0xFF353D41), None),
                            ),
                        ))
                        .with_flex_child(
                            1,
                            Row::default()
                                .with_flex_child(
                                    1,
                                    Column::default()
                                        .with_child(Text::new(
                                            "Zone",
                                            TextStyle::default().with_color(Color::new(0xFF637079)),
                                        )),
                                )
                                .with_child(Constrained::new_empty(Constraints::new_tight(
                                    Size::new_unbound_height(60.0),
                                )))
                                .with_flex_child(1, self.build_sensor_data("Volume"))
                                .with_child(Constrained::new_empty(Constraints::new_tight(
                                    Size::new_unbound_height(60.0),
                                )))
                                .with_flex_child(1, self.build_sensor_data("Occupancy"))
                                .with_child(Constrained::new_empty(Constraints::new_tight(
                                    Size::new_unbound_height(60.0),
                                )))
                                .with_flex_child(1, self.build_sensor_data("Speed")),
                        ),
                ),
            ),
        ));
    }
}
