//! Armor and Defense types for D&D 5e.
//!
//! This module contains types for calculating armor class and movement speed.

use serde::{Deserialize, Serialize};

/// Armor class calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorClass {
    pub base: u8,
    pub armor_type: Option<ArmorType>,
    pub shield_bonus: u8,
}

impl ArmorClass {
    pub fn unarmored() -> Self {
        Self {
            base: 10,
            armor_type: None,
            shield_bonus: 0,
        }
    }

    pub fn calculate(&self, dex_mod: i8) -> u8 {
        let base = self.base as i8;
        let shield = self.shield_bonus as i8;

        let dex_bonus = match self.armor_type {
            None => dex_mod,
            Some(ArmorType::Light) => dex_mod,
            Some(ArmorType::Medium) => dex_mod.min(2),
            Some(ArmorType::Heavy) => 0,
        };

        (base + dex_bonus + shield).max(0) as u8
    }
}

impl Default for ArmorClass {
    fn default() -> Self {
        Self::unarmored()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
}

/// Movement speed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: u32,
    pub swim: Option<u32>,
    pub fly: Option<u32>,
    pub climb: Option<u32>,
}

impl Speed {
    pub fn new(walk: u32) -> Self {
        Self {
            walk,
            swim: None,
            fly: None,
            climb: None,
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(30)
    }
}
