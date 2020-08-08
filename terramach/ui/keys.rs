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

use flagset::{FlagSet, flags};
use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign};
use std::collections::HashMap;

flags! {
    pub enum KeyModifier: u8 {
        Shift,
        Control,
        Alt,
        CapsLock,
        NumLock,
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct KeyModifiers(FlagSet<KeyModifier>);

impl KeyModifiers {
    pub fn new(modifiers: impl Into<FlagSet<KeyModifier>>) -> Self {
        Self(modifiers.into())
    }

    pub fn clear_all(&mut self) {
        self.0 = !self.0;
    }

    pub fn is_empty(&self) -> bool {
        self.0.bits() == 0
    }

    pub fn set(&mut self, modifiers: impl Into<FlagSet<KeyModifier>>) {
        self.0 |= modifiers.into();
    }

    pub fn clear(&mut self, modifiers: impl Into<FlagSet<KeyModifier>>) {
        self.0 &= !modifiers.into();
    }

    pub fn is_set(&self, modifier: KeyModifier) -> bool {
        self.0 & modifier == modifier
    }

    pub fn is_shift(&self) -> bool {
        self.is_set(KeyModifier::Shift)
    }

    pub fn is_control(&self) -> bool {
        self.is_set(KeyModifier::Control)
    }

    pub fn is_alt(&self) -> bool {
        self.is_set(KeyModifier::Alt)
    }

    pub fn is_caps_lock(&self) -> bool {
        self.is_set(KeyModifier::CapsLock)
    }

    pub fn is_num_lock(&self) -> bool {
        self.is_set(KeyModifier::NumLock)
    }
}

impl Into<FlagSet<KeyModifier>> for KeyModifiers {
    fn into(self) -> FlagSet<KeyModifier> {
        self.0
    }
}

impl BitOr<KeyModifier> for KeyModifiers {
    type Output = KeyModifiers;

    fn bitor(mut self, rhs: KeyModifier) -> Self::Output {
        self.set(rhs);
        self
    }
}

impl BitOrAssign<KeyModifier> for KeyModifiers {
    fn bitor_assign(&mut self, rhs: KeyModifier) {
        self.set(rhs);
    }
}

impl BitAnd<KeyModifier> for KeyModifiers {
    type Output = KeyModifiers;

    fn bitand(mut self, rhs: KeyModifier) -> Self::Output {
        self.clear(!rhs);
        self
    }
}

impl BitAndAssign<KeyModifier> for KeyModifiers {
    fn bitand_assign(&mut self, rhs: KeyModifier) {
        self.clear(!rhs)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyAction {
    Release,
    Press,
    Repeat,
}

pub type KeyScanCode = u32;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Key {
    Num1 = 18,
    Num2 = 19,
    Num3 = 20,
    Num4 = 21,
    Num5 = 23,
    Num6 = 22,
    Num7 = 26,
    Num8 = 28,
    Num9 = 25,
    Num0 = 29,
    BackSpace = 51,
    Escape = 53,
    Space = 49,
    LeftCommand = 55,
    RightCommand = 54,
    LeftAlt = 58,
    RightAlt = 61,
    Control = 59,
    Function = 63,
    Tab = 48,
    CapsLock = 57,
    LeftShift = 56,
    RightShift = 60,
    Q = 12,
    W = 13,
    E = 14,
    R = 15,
    T = 17,
    Y = 16,
    U = 32,
    I = 34,
    O = 31,
    P = 35,
    A = 0,
    S = 1,
    D = 2,
    F = 3,
    G = 5,
    H = 4,
    J = 38,
    K = 40,
    L = 37,
    Z = 6,
    X = 7,
    C = 8,
    V = 9,
    B = 11,
    N = 45,
    M = 46,
    GraveAccent = 50,
    Minus = 27,
    Equal = 24,
    LeftBracket = 33,
    RightBracket = 30,
    Backslash = 42,
    Semicolon = 41,
    Apostrophe = 39,
    Comma = 43,
    Period = 47,
    Slash = 44,
    Enter = 36,
    Left = 123,
    Right = 124,
    Up = 126,
    Down = 125,
    Delete = 117,
}

impl From<KeyScanCode> for Key {
    fn from(scan_code: u32) -> Self {
        unsafe { std::mem::transmute(scan_code) }
    }
}

#[derive(Debug, Clone)]
pub struct HitKey {
    character: Option<char>,
    scan_code: KeyScanCode,
    key: Key,
    action: KeyAction,
    modifiers: KeyModifiers,
}

impl HitKey {
    pub fn new(
        character: impl Into<Option<char>>,
        scan_code: KeyScanCode,
        action: KeyAction,
        modifiers: KeyModifiers,
    ) -> Self {
        Self {
            character: character.into(),
            key: Key::from(scan_code),
            scan_code,
            action,
            modifiers,
        }
    }

    pub fn printable_character(&self) -> Option<char> {
        self.character
    }

    pub fn scan_code(&self) -> KeyScanCode {
        self.scan_code
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn action(&self) -> KeyAction {
        self.action
    }

    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }
}

struct ActiveKey {
    character: Option<char>,
    action: Option<KeyAction>,
}

pub struct KeyTracker {
    modifiers: KeyModifiers,
    scan_code: Option<KeyScanCode>,
    keys: HashMap<KeyScanCode, ActiveKey>,
}

impl KeyTracker {
    pub fn new() -> Self {
        Self {
            modifiers: KeyModifiers::default(),
            scan_code: None,
            keys: HashMap::new(),
        }
    }

    pub fn poll_keys(&mut self) -> Option<Vec<HitKey>> {
        if self.keys.is_empty() {
            None
        } else {
            let mut keys = Vec::new();
            let mut removed = Vec::new();
            for (scan_code, key) in self.keys.iter_mut() {
                if let Some(action) = key.action {
                    keys.push(HitKey::new(
                        key.character,
                        *scan_code,
                        action,
                        self.modifiers,
                    ));
                    if action == KeyAction::Release {
                        removed.push(*scan_code);
                    }
                    key.action = None;
                }
            }
            for scan_code in &removed {
                self.keys.remove(scan_code);
            }
            Some(keys)
        }
    }

    pub fn push_character(&mut self, character: char) {
        if let Some(scan_code) = &self.scan_code {
            if let Some(key) = self.keys.get_mut(scan_code) {
                key.character = Some(character);
            }
        }
    }

    pub fn set_modifiers(&mut self, modifiers: impl Into<FlagSet<KeyModifier>>) {
        self.modifiers.set(modifiers);
    }

    pub fn clear_modifiers(&mut self, modifiers: impl Into<FlagSet<KeyModifier>>) {
        self.modifiers.clear(modifiers);
    }

    pub fn push_action(&mut self, action: KeyAction) {
        if let Some(scan_code) = &self.scan_code {
            if let Some(key) = self.keys.get_mut(scan_code) {
                key.action = Some(action);
            }
        }
    }

    pub fn push_scan_code(&mut self, scan_code: KeyScanCode) {
        self.scan_code = Some(scan_code);
        if !self.keys.contains_key(&scan_code) {
            self.keys.insert(scan_code, ActiveKey {
                character: None,
                action: None,
            });
        }
    }
}
