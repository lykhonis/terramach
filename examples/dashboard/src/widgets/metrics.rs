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

use std::time::Duration;
use terramach::graphics::*;
use terramach::widgets::*;
use terramach::*;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Metric {
    level: i32,
    suffix: String,
    description: String,
    color: Color,
    value: f32,
}

impl Metric {
    pub fn new(
        level: i32,
        suffix: impl AsRef<str>,
        description: impl AsRef<str>,
        color: impl Into<Color>,
        value: f32,
    ) -> Self {
        Metric {
            level,
            suffix: suffix.as_ref().to_string(),
            description: description.as_ref().to_string(),
            color: color.into(),
            value,
        }
    }
}

impl Widget for Metric {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(MetricState::new(self.level, self.value));
        let state = context.state_mut::<MetricState>().unwrap();
        state.animation = Duration::from_secs(25)
            .animate(Tween::new(0.0, 1.0))
            .with(AnimationCurves::ease_out())
            .into();
        context.request_frame();
    }

    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<MetricState>().unwrap();
        build.add_child(Decoration::new(
            Color::new(0xFF242C30),
            BorderRadius::new_all(5.0),
            Padding::new_all(
                20.0,
                Column::default()
                    .with_child(Text::new(
                        &format!("{}{}", state.level(), self.suffix),
                        TextStyle::default()
                            .with_font_style(FontStyle::new(
                                FontStyleWeight::THIN,
                                FontStyleWidth::EXTRA_CONDENSED,
                                FontStyleSlant::Upright,
                            ))
                            .with_font_size(54.0),
                    ))
                    .with_child(Constrained::new_empty(Constraints::new_tight(
                        Size::new_unbound_width(5.0),
                    )))
                    .with_child(Text::new(
                        &self.description,
                        TextStyle::default().with_color(Color::new(0xFF637079)),
                    ))
                    .with_child(Constrained::new_empty(Constraints::new_tight(
                        Size::new_unbound_width(35.0),
                    )))
                    .with_child(Constrained::new(
                        Constraints::new_loose(Size::new_unbound_width(3.0)),
                        Stack::new()
                            .with_child(Decoration::new_empty(Color::new(0xFF353D41), None))
                            .with_child(Fractional::new(
                                (state.value(), 1.0),
                                Decoration::new_empty(self.color, BorderRadius::new_all(3.0)),
                            )),
                    )),
            ),
        ));
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Frame(_) => {
                let state = context.state_mut::<MetricState>().unwrap();
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

struct MetricState {
    animation: Option<Animation<f32>>,
    level: Tween<i32>,
    value: Tween<f32>,
}

impl MetricState {
    pub fn new(level: i32, value: f32) -> Self {
        MetricState {
            animation: None,
            level: Tween::new(0, level),
            value: Tween::new(0.0, value),
        }
    }

    pub fn animation(&self) -> Option<f32> {
        self.animation.as_ref()?.value().into()
    }

    pub fn level(&self) -> i32 {
        self.level.animate(self.animation().unwrap_or(1.0))
    }

    pub fn value(&self) -> f32 {
        self.value.animate(self.animation().unwrap_or_default())
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Metrics {}

impl Metrics {
    pub fn new() -> Self {
        Metrics {}
    }
}

impl Widget for Metrics {
    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(Constrained::new(
            Constraints::new_loose(Size::new_unbound_width(180.0)),
            Row::default()
                .with_flex_child(1, Metric::new(81, "%", "Average Volume", 0xFFCA4848, 0.81))
                .with_child(Constrained::new_empty(Constraints::new_tight(
                    Size::new_unbound_height(20.0),
                )))
                .with_flex_child(1, Metric::new(67, "%", "Average Occupancy", 0xFFD29E57, 0.67))
                .with_child(Constrained::new_empty(Constraints::new_tight(
                    Size::new_unbound_height(20.0),
                )))
                .with_flex_child(1, Metric::new(92, "", "Average Speed", 0xFF54B5B5, 0.88)),
        ));
    }
}
