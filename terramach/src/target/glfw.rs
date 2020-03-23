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

use std::sync::mpsc::Receiver;

use glfw;
use glfw::Context;

use terramach_graphics::{ISize, Size, Point, Display};
use terramach_graphics::gl;

use crate::platform::RunLoop;
use crate::{AppEvents, TouchTracker, KeyTracker, AppEvent, KeyModifier, KeyScanCode, KeyAction, TouchId, Widget, BoxedWidget, run_app};

pub struct App {
    title: Option<String>,
    size: ISize,
}

impl App {
    pub fn new(size: impl Into<ISize>) -> Self {
        App {
            title: None,
            size: size.into(),
        }
    }

    pub fn with_title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().to_string().into();
        self
    }

    pub fn run(self, content: impl Into<BoxedWidget>) {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .expect("Failed to initialize GLFW");
        if cfg!(target_os = "macos") {
            glfw.window_hint(glfw::WindowHint::CocoaGraphicsSwitching(true));
        }
        let (mut window, events) = glfw.create_window(
            self.size.width as u32,
            self.size.height as u32,
            &self.title.unwrap_or_default(),
            glfw::WindowMode::Windowed,
        ).expect("Failed to create a native window");
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

        let mut display = gl::Display::new(
            self.size,
            window,
            |window, symbol| window.get_proc_address(symbol),
            |_| glfw::make_context_current(None),
            |window| window.make_current(),
            |window| window.swap_buffers(),
        ).expect("Failed to create a display in GL window");

        display.clean_current();

        let mut run_loop = RunLoop::new();
        let mut app_events = AppEvents::new();
        let mut event_emitter = app_events.emitter();
        let mut touches = TouchTracker::new();
        let mut keys = KeyTracker::new();

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
            for (_, event) in glfw::flush_messages(&events) {
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

        run_app(run_loop, app_events, display, content);
    }
}
