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

use jni::JNIEnv;
use jni::objects::{JClass, JString, JValue, JObject};
use std::sync::MutexGuard;

use crate::{run_app, AppEvents, EventEmitter, AppEvent};
use crate::platform::{RunLoop, App};
use crate::platform;

use terramach_graphics::{gl, Display, Color, Paint, Rect};
use terramach_graphics::gl::egl;

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachController_create(env: JNIEnv, obj: JObject) {}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachController_destroy(env: JNIEnv, obj: JObject) {}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachController_start(env: JNIEnv, obj: JObject) {}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_TerraMachController_stop(env: JNIEnv, obj: JObject) {}
