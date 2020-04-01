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

#[derive(Debug, Default, Clone)]
pub struct IndexPool {
    current_index: usize,
    free_indexes: Vec<usize>,
}

impl IndexPool {
    pub fn new() -> Self {
        IndexPool {
            current_index: 0,
            free_indexes: Vec::new(),
        }
    }

    pub fn take(&mut self) -> usize {
        if let Some(index) = self.free_indexes.pop() {
            index
        } else {
            let index = self.current_index;
            self.current_index += 1;
            index
        }
    }

    pub fn give(&mut self, index: usize) {
        self.free_indexes.push(index);
    }
}
