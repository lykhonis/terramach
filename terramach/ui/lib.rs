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

#![feature(vec_remove_item)]
#![feature(panic_info_message)]

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;

pub mod platform;
pub mod gpu;
pub mod widgets;
mod animation;
mod app;
mod bundle;
mod common;
mod event;
mod gesture;
mod hit;
mod layer;
mod layers;
mod layout;
mod paint;
mod timers;
mod touch;
mod tree;
mod widget;
mod keys;

pub use platform::App;

pub(crate) use common::*;
pub(crate) use app::*;

pub use animation::*;
pub use bundle::*;
pub use event::*;
pub use gesture::*;
pub use hit::*;
pub use layer::*;
pub use layers::*;
pub use layout::*;
pub use paint::*;
pub use timers::*;
pub use touch::*;
pub use tree::*;
pub use widget::*;
pub use keys::*;

pub use terramach_graphics as graphics;

pub use terramach_support::{EventId, PartialWidget};

#[cfg(target_os = "macos")]
pub use terramach_support::noop_attribute as terramach_main;
#[cfg(target_os = "android")]
pub use terramach_support::terramach_main;
