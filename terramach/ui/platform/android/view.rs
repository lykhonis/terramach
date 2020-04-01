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

use std::sync::MutexGuard;

use jni::JNIEnv;
use jni::sys::jfloat;
use jni::objects::{JClass, JString, JValue, JObject};

use crate::{run_app, AppEvents, EventEmitter, AppEvent};
use crate::platform::{bindings, RunLoop, App, SharedRunLoop};
use crate::platform;

use terramach_graphics::{gl, Display, Color, Paint, Rect};
use terramach_graphics::gl::egl;

struct Egl {
    display: egl::EGLDisplay,
    surface: egl::EGLSurface,
    context: egl::EGLContext,
}

impl Egl {
    pub fn new(display: egl::EGLDisplay, surface: egl::EGLSurface, context: egl::EGLContext) -> Self {
        Egl {
            display,
            surface,
            context,
        }
    }

    pub fn clear_current(&self) {
        let result = egl::make_current(self.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);
        debug_assert!(result);
    }

    pub fn make_current(&self) {
        let result = egl::make_current(self.display, self.surface, self.surface, self.context);
        debug_assert!(result);
    }

    pub fn present_current(&self) {
        let result = egl::swap_buffers(self.display, self.surface);
        debug_assert!(result);
    }
}

impl Drop for Egl {
    fn drop(&mut self) {
        let result = egl::make_current(self.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);
        debug_assert!(result);
        let result = egl::destroy_surface(self.display, self.surface);
        debug_assert!(result);
        let result = egl::destroy_context(self.display, self.context);
        debug_assert!(result);
    }
}

struct AppState {
    event_emitter: EventEmitter<AppEvent>,
    run_loop: SharedRunLoop,
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.run_loop.stop();
    }
}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachView_runApp(env: JNIEnv, obj: JObject, surface: JObject) {
    let app: jni::errors::Result<MutexGuard<AppState>> = env.get_rust_field(obj, "app");
    if app.is_ok() {
        return;
    }

    let display = egl::get_display(egl::EGL_DEFAULT_DISPLAY)
        .expect("No default EGL display available");

    let mut majorVersion = 0;
    let mut minorVersion = 0;
    let result = egl::initialize(display, &mut majorVersion, &mut minorVersion);
    debug_assert!(result, "Failed to initialize EGL");

    let config = egl::android_choose_config(display);
    let context = egl::android_new_context(display, config);

    let window = unsafe {
        bindings::ANativeWindow_fromSurface(env.get_native_interface() as *mut _, surface.into_inner() as *mut _)
    };
    debug_assert!(!window.is_null(), "Failed to access a native window from a surface");

    let (width, height) = unsafe {
        let pixel_ratio = env.call_method(
            obj,
            "getDevicePixelRatio",
            "()F",
            &[],
        ).unwrap().f().unwrap();
        (
            bindings::ANativeWindow_getWidth(window) as f32 / pixel_ratio,
            bindings::ANativeWindow_getHeight(window) as f32 / pixel_ratio,
        )
    };

    let surface = egl::create_window_surface(display, config, window as *mut _, &[])
        .expect("Failed to create a EGL surface");
    let result = egl::make_current(display, surface, surface, context);
    debug_assert!(result);

    let display = gl::Display::new(
        (width, height),
        Egl::new(display, surface, context),
        |_, symbol| egl::get_proc_address(symbol) as *mut _,
        |egl| egl.clear_current(),
        |egl| egl.make_current(),
        |egl| egl.present_current(),
    ).expect("Failed to initialize EGL display");

    let mut app = platform::new_app().expect("No app is running");
    let content = app.take_content().expect("App is empty");
    let run_loop = RunLoop::new();
    let mut app_events = AppEvents::new();
    env.set_rust_field(
        obj,
        "app",
        AppState {
            event_emitter: app_events.emitter(),
            run_loop: run_loop.share(),
        },
    ).unwrap();

    run_app(
        run_loop,
        app_events,
        display,
        content,
    );
}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachView_stopApp(env: JNIEnv, obj: JObject) {
    let _: jni::errors::Result<AppState> = env.take_rust_field(obj, "app");
}
