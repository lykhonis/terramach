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

use lazy_static;
use config;

lazy_static! {
    static ref SETTINGS: Settings = {
        let mut config = config::Config::default();
        let _ = config.merge(config::File::with_name("Settings"));
        let mapbox = config.get_table("mapbox").unwrap_or_default();
        let access_token = mapbox.get("access-token")
            .and_then(|value| value.clone().into_str().ok())
            .unwrap_or_else(|| "NO_ACCESS_TOKEN".to_string());
        let cache_path = mapbox.get("cache-path")
            .and_then(|value| value.clone().into_str().ok())
            .unwrap_or_else(|| "/tmp/mapbox.cache.db".to_string());
        Settings {
            mapbox_access_token: access_token,
            mapbox_cache_path: cache_path,
        }
    };
}

pub struct Settings {
    mapbox_access_token: String,
    mapbox_cache_path: String,
}

impl Settings {
    pub fn mapbox_cache_path() -> &'static str {
        &SETTINGS.mapbox_cache_path
    }

    pub fn mapbox_access_token() -> &'static str {
        &SETTINGS.mapbox_access_token
    }
}
