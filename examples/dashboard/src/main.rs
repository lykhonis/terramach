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

use crate::widgets::*;

use terramach::*;
use terramach::graphics::*;
use terramach::widgets::*;

fn main() {
    App::new((1020, 640))
        .with_title("Terra Mach - Dashboard")
        .run(DefaultTextStyle::new(
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
        ));
}
