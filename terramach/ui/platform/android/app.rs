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

use std::sync::Mutex;

use crate::{AppEvents, BoxedWidget};
use crate::platform::RunLoop;

use terramach_graphics::{gl, Display, Color, Paint, ISize, Rect};
use terramach_graphics::gl::egl;

use lazy_static::lazy_static;

pub struct App {
    content: Option<BoxedWidget>,
}

impl App {
    pub fn new() -> Self {
        App {
            content: None,
        }
    }

    pub fn with_size(self, _size: impl Into<ISize>) -> Self {
        // size is set by a surface view
        self
    }

    pub fn with_title(self, _title: impl AsRef<str>) -> Self {
        // no title for an app
        self
    }

    pub fn run(mut self, content: impl Into<BoxedWidget>) {
        self.content = content.into().into();
        set_current_app(self);
    }

    pub(crate) fn take_content(&mut self) -> Option<BoxedWidget> {
        self.content.take()
    }
}

pub(crate) fn new_app() -> Option<App> {
    extern "C" {
        fn terramach_main();
    }
    unsafe {
        terramach_main();
    }
    if let Ok(mut current_app) = current_app().lock() {
        current_app.take()
    } else {
        None
    }
}

unsafe impl Send for App {}

lazy_static! {
    static ref CURRENT_APP: Mutex<Option<App>> = Mutex::new(None);
}

//noinspection RsTypeCheck
fn current_app<'a>() -> &'a Mutex<Option<App>> {
    &CURRENT_APP
}

fn set_current_app(app: App) {
    let mut current_app = current_app().lock().expect("Unable to lock app!");
    current_app.replace(app);
}
