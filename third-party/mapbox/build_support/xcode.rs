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

use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn sdk_path(sdk: impl AsRef<str>) -> Option<PathBuf> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("--sdk").arg(sdk.as_ref()).arg("--show-sdk-path");
    let output = cmd.stderr(Stdio::inherit()).output().ok()?;
    if output.status.code() != Some(0) {
        return None;
    }
    Some({
        let str = String::from_utf8(output.stdout).unwrap();
        PathBuf::from(str.trim())
    })
}
