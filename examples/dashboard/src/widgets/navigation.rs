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

use crate::widgets::{Map, MapController};

use mapbox;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Navigation {}

impl Navigation {
    pub fn new() -> Self {
        Navigation {}
    }
}

impl Widget for Navigation {
    fn mount(&self, context: &mut WidgetContext, _: &mut MountContext) {
        context.set_state(NavigationState::new());
    }

    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state_mut::<NavigationState>().unwrap();
        build.add_child(Constrained::new(
            Constraints::new_tight(Size::new_unbound_width(300.0)),
            Decoration::new(
                Color::new(0xFF242C30),
                BorderRadius::new_all(5.0),
                Column::default()
                    .with_child(
                        Constrained::new(
                            Constraints::new_tight(Size::new_unbound()),
                            Map::new(
                                &mapbox::CameraOptions::new().with_zoom(3.0),
                                &mut state.map,
                            ),
                        ),
                    ),
            ),
        ));
    }
}

struct NavigationState {
    map: MapController,
}

impl NavigationState {
    pub fn new() -> Self {
        NavigationState {
            map: MapController::new(),
        }
    }
}
