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

#[derive(PartialEq, EventId)]
enum Button {
    RealTime = 1,
    Analysis = 2,
    Diagnostics = 3,
    Configuration = 4,
}

#[derive(Clone, PartialEq, PartialWidget)]
struct Tab {
    title: String,
    selected: bool,
}

impl Tab {
    pub fn new(
        title: impl AsRef<str>,
        selected: bool,
    ) -> Self {
        Tab {
            title: title.as_ref().to_string(),
            selected,
        }
    }
}

impl Widget for Tab {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(TabState::new(self.selected));
    }

    fn update(&self, context: &mut WidgetContext, _: &mut UpdateContext) {
        let state = context.state_mut::<TabState>().unwrap();
        state.animation = Duration::from_millis(350)
            .animate(Tween::new(
                state.animation().unwrap_or(
                    if state.selected { 1.0 } else { 0.0 }
                ),
                if self.selected { 1.0 } else { 0.0 },
            ))
            .with(AnimationCurves::ease_out())
            .into();
        state.selected = self.selected;
        context.request_frame();
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(
            Align::new(
                Alignment::center(),
                Text::new(
                    &self.title,
                    TextStyle::default()
                        .with_font_style(FontStyle::new(
                            FontStyleWeight::MEDIUM,
                            FontStyleWidth::CONDENSED,
                            FontStyleSlant::Upright,
                        ))
                        .with_font_size(12.0)
                        .with_color(if self.selected { Color::new(0xFF2BC1FE) } else { Color::new(0xFF637079) }),
                ),
            ),
        );
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Frame(_) => {
                let state = context.state_mut::<TabState>().unwrap();
                if let Some(animation) = &mut state.animation {
                    if animation.advance() {
                        event.mark_need_paint();
                    } else {
                        state.animation = None;
                    }
                    context.request_frame();
                }
            }
            _ => {}
        }
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state_mut::<TabState>().unwrap();

        if self.selected || state.animation.is_some() {
            let animation = state.animation().unwrap_or(1.0);

            let color = Color4f::from(Color::new(0xFF2BC1FE));
            let size = paint.size();

            let canvas = paint.canvas();

            let bar_width = 3.0;

            let mut paint = Paint::new(&color, None);
            paint.set_alpha_f(animation);
            let rect = Rect::from_size((size.width, bar_width));
            canvas.draw_rect(rect, &paint);

            let mut paint = Paint::new(&color, None);
            paint.set_image_filter(image_filters::blur(
                (bar_width * 2.0, bar_width * 3.0),
                None,
                None,
                None,
            ));
            paint.set_alpha_f(animation);
            let width = size.width + bar_width * 2.0;
            let rect = Rect::from_xywh(
                (size.width - width) / 2.0,
                bar_width,
                width,
                bar_width,
            );
            canvas.draw_rect(rect, &paint);
        }

        paint.paint_children();
    }
}

struct TabState {
    animation: Option<Animation<f32>>,
    selected: bool,
}

impl TabState {
    pub fn new(selected: bool) -> Self {
        TabState {
            animation: None,
            selected,
        }
    }

    pub fn animation(&self) -> Option<f32> {
        self.animation.as_ref()?.value().into()
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Topbar {}

impl Topbar {
    pub fn new() -> Self {
        Topbar {}
    }

    fn build_tab(
        &self,
        button: Button,
        event_emitter: WidgetEventEmitter,
        title: &str,
        selected: bool,
    ) -> impl Widget {
        Gesture::new(
            button,
            event_emitter,
            TapGesture::default(),
            None,
            Padding::new_horizontal(
                25.0,
                Tab::new(title, selected),
            ),
        )
    }

    fn build_search(&self) -> impl Widget {
        let height = 36.0;
        let radius = height / 2.0;
        Align::new(
            Alignment::right_center(),
            Constrained::new(
                Constraints::new(
                    Size::new(100.0, height),
                    Size::new(200.0, height),
                ),
                Decoration::new(
                    Color::new(0xFF1E2429),
                    BorderRadius::new_all(radius),
                    Padding::new(
                        radius, 0.0, radius, 0.0,
                        Align::new(
                            Alignment::left_center(),
                            TextInput::new_empty()
                                .with_hint("Search"),
                        ),
                    ),
                ),
            ),
        )
    }
}

impl Widget for Topbar {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(TopbarState::new());
    }

    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<TopbarState>().unwrap();
        build.add_child(Constrained::new(
            Constraints::new_loose(Size::new_unbound_width(60.0)),
            Decoration::new(
                Color::new(0xFF242C30),
                None,
                Row::new(MainAxisAlignment::default(), CrossAxisAlignment::Stretch)
                    .with_child(Constrained::new_empty(Constraints::new_tight(Size::new_unbound_height(20.0))))
                    .with_child(Align::new(
                        Alignment::center(),
                        Text::new(
                            "SAN DIEGO, CA",
                            TextStyle::default()
                                .with_font_style(FontStyle::new(
                                    FontStyleWeight::NORMAL,
                                    FontStyleWidth::EXTRA_CONDENSED,
                                    FontStyleSlant::Upright,
                                ))
                                .with_color(Color::new(0xFF2BC1FE)),
                        ),
                    ))
                    .with_flex_child(1, Constrained::new_empty(Constraints::new_loose(Size::new_unbound())))
                    .with_child(self.build_tab(Button::RealTime, build.event_emitter(), "REAL-TIME", state.selected_button == Button::RealTime))
                    .with_child(self.build_tab(Button::Analysis, build.event_emitter(), "ANALYSIS", state.selected_button == Button::Analysis))
                    .with_child(self.build_tab(Button::Diagnostics, build.event_emitter(), "DIAGNOSTICS", state.selected_button == Button::Diagnostics))
                    .with_child(self.build_tab(Button::Configuration, build.event_emitter(), "CONFIGURATION", state.selected_button == Button::Configuration))
                    .with_flex_child(1, Constrained::new_empty(Constraints::new_loose(Size::new_unbound())))
                    .with_child(self.build_search())
                    .with_child(Constrained::new_empty(Constraints::new_tight(Size::new_unbound_height(20.0)))),
            ),
        ));
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::Tap(id) => {
                let state = context.state_mut::<TopbarState>().unwrap();
                state.selected_button = id.into();
                event.mark_need_build();
            }
            _ => {}
        }
    }
}

struct TopbarState {
    selected_button: Button,
}

impl TopbarState {
    pub fn new() -> Self {
        TopbarState {
            selected_button: Button::RealTime,
        }
    }
}
