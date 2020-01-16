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

#[macro_use]
extern crate lazy_static;

mod widgets;
mod settings;

use glfw;
use glfw::Context;

use terramach::graphics::{Color, Point, Size};
use terramach::graphics::gl::Display;
use terramach::platform::RunLoop;
use terramach::widgets::{
    Column, Constrained, Decoration, DefaultTextStyle, Padding, Row, ScrollDirection, Scrollable,
    TextStyle,
};
use terramach::{App, AppEvent, AppEvents, Constraints, MeasuredSize, TouchId, TouchTracker, KeyTracker, KeyScanCode, KeyModifier, KeyAction};

use crate::widgets::*;

fn main() {
    let (width, height) = (1020, 640);

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::CocoaGraphicsSwitching(true));

    let (mut window, gflw_events) = glfw
        .create_window(width, height, "Terra Mach", glfw::WindowMode::Windowed)
        .unwrap();
    window.set_resizable(true);
    window.set_close_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_enter_polling(true);
    window.set_mouse_button_polling(true);
    window.set_size_polling(true);
    window.set_scroll_polling(true);
    window.set_key_polling(true);
    window.set_char_polling(true);
    window.set_focus_polling(true);

    glfw::make_context_current(None);

    let display = Display::new(
        (width as f32, height as f32),
        window,
        |window, symbol| window.get_proc_address(symbol),
        |_| glfw::make_context_current(None),
        |window| window.make_current(),
        |window| window.swap_buffers(),
    ).expect("Failed to create display");

    let mut events = AppEvents::new();
    let mut event_emitter = events.emitter();
    let mut touches = TouchTracker::new();
    let mut keys = KeyTracker::new();
    let mut run_loop = RunLoop::new();

    let pinch_touch_id = 0;
    let mut pinch_location: Option<Point> = None;
    let mut pinch_locked = None;

    run_loop.add_observer(move || {
        glfw.poll_events_unbuffered(|_, event| match event.1 {
            glfw::WindowEvent::Size(width, height) => {
                event_emitter.emit_event(AppEvent::Resize(Size::new(width as f32, height as f32)));
                None
            }
            _ => Some(event),
        });
        for (_, event) in glfw::flush_messages(&gflw_events) {
            match event {
                glfw::WindowEvent::Close => {
                    event_emitter.emit_event(AppEvent::Quit);
                }
                glfw::WindowEvent::Focus(focused) => {
                    touches.reset();
                    event_emitter.emit_event(AppEvent::Focus(focused));
                }
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    let id = 1 /* reserved */ + button as TouchId;
                    if glfw::Action::Press == action {
                        if let Some(location) = pinch_location {
                            let touch = touches.begin_touch_with_offset(pinch_touch_id, location);
                            event_emitter.emit_event(AppEvent::TouchBegin(touch));
                        }
                        if let Some(touch) = touches.begin_touch(id) {
                            event_emitter.emit_event(AppEvent::TouchBegin(touch));
                        }
                    } else if glfw::Action::Release == action {
                        if let Some(touch) = touches.end_touch(pinch_touch_id) {
                            event_emitter.emit_event(AppEvent::TouchEnd(touch));
                        }
                        if let Some(touch) = touches.end_touch(id) {
                            event_emitter.emit_event(AppEvent::TouchEnd(touch));
                        }
                    }
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    let location = Point::new(x as f32, y as f32);
                    if pinch_location.is_none() && pinch_locked.unwrap_or_default() {
                        pinch_location = Some(location);
                    }
                    event_emitter.emit_event(AppEvent::Hover(location));
                    if let Some(touch) = touches.push_offset(location) {
                        event_emitter.emit_event(AppEvent::TouchUpdate(touch));
                    }
                }
                glfw::WindowEvent::Scroll(x, y) => {
                    event_emitter.emit_event(AppEvent::Scroll(Point::new(x as f32, y as f32)));
                }
                glfw::WindowEvent::Char(character) => {
                    keys.push_character(character);
                }
                glfw::WindowEvent::Key(key, scan_code, action, modifiers) => {
                    if key == glfw::Key::LeftSuper || key == glfw::Key::RightSuper {
                        pinch_location = None;
                        match action {
                            glfw::Action::Press => pinch_locked = Some(true),
                            glfw::Action::Release => {
                                pinch_locked = Some(false);
                                if let Some(touch) = touches.end_touch(pinch_touch_id) {
                                    event_emitter.emit_event(AppEvent::TouchEnd(touch));
                                }
                            }
                            _ => {}
                        }
                    }
                    keys.push_scan_code(scan_code as KeyScanCode);
                    match action {
                        glfw::Action::Release => keys.push_action(KeyAction::Release),
                        glfw::Action::Press => keys.push_action(KeyAction::Press),
                        glfw::Action::Repeat => keys.push_action(KeyAction::Repeat),
                    }
                    keys.clear_modifiers(KeyModifier::Shift | KeyModifier::NumLock | KeyModifier::Alt | KeyModifier::Control);
                    if modifiers & glfw::Modifiers::Shift == glfw::Modifiers::Shift {
                        keys.set_modifiers(KeyModifier::Shift);
                    }
                    if modifiers & glfw::Modifiers::NumLock == glfw::Modifiers::NumLock {
                        keys.set_modifiers(KeyModifier::NumLock);
                    }
                    if modifiers & glfw::Modifiers::CapsLock == glfw::Modifiers::CapsLock {
                        keys.set_modifiers(KeyModifier::CapsLock);
                    }
                    if modifiers & glfw::Modifiers::Alt == glfw::Modifiers::Alt {
                        keys.set_modifiers(KeyModifier::Alt);
                    }
                    if modifiers & glfw::Modifiers::Control == glfw::Modifiers::Control {
                        keys.set_modifiers(KeyModifier::Control);
                    }
                    if key == glfw::Key::CapsLock {
                        if action == glfw::Action::Press {
                            keys.set_modifiers(KeyModifier::CapsLock);
                        } else if action == glfw::Action::Release {
                            keys.clear_modifiers(KeyModifier::CapsLock);
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(keys) = keys.poll_keys() {
            for key in keys {
                event_emitter.emit_event(AppEvent::Key(key));
            }
        }
    });

    let app = App::new(
        run_loop,
        events,
        display,
        DefaultTextStyle::new(
            TextStyle::default()
                .with_color(Color::new(0xFFFFFFFF))
                .with_font_families(&["Helvetica Neue"]),
            Decoration::new(
                Color::new(0xFF1E2429),
                None,
                Row::default()
                    .with_child(Sidebar::new())
                    .with_flex_child(
                        1,
                        Column::default()
                            .with_child(Topbar::new())
                            .with_flex_child(
                                1,
                                Scrollable::new(
                                    ScrollDirection::Vertical,
                                    Padding::new_all(
                                        20.0,
                                        Column::default()
                                            .with_child(Navigation::new())
                                            .with_child(Constrained::new_empty(Constraints::new_tight(
                                                Size::new_unbound_width(20.0),
                                            )))
                                            .with_child(Metrics::new())
                                            .with_child(Constrained::new_empty(Constraints::new_tight(
                                                Size::new_unbound_width(20.0),
                                            )))
                                            .with_child(Sensors::new()),
                                    ),
                                ),
                            ),
                    ),
            ),
        ),
    );
    app.run();
}
