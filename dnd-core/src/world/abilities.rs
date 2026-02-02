//! Ability scores for D&D 5e characters.
//!
//! This module defines the six ability scores (Strength, Dexterity, Constitution,
//! Intelligence, Wisdom, Charisma) and the container type for storing them.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Ability Scores
// ============================================================================

/// The six ability scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Ability::Strength => "STR",
            Ability::Dexterity => "DEX",
            Ability::Constitution => "CON",
            Ability::Intelligence => "INT",
            Ability::Wisdom => "WIS",
            Ability::Charisma => "CHA",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Charisma => "Charisma",
        }
    }

    pub fn all() -> [Ability; 6] {
        [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ]
    }
}

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Ability scores container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> Self {
        Self {
            strength: str,
            dexterity: dex,
            constitution: con,
            intelligence: int,
            wisdom: wis,
            charisma: cha,
        }
    }

    pub fn standard_array() -> Self {
        Self::new(15, 14, 13, 12, 10, 8)
    }

    pub fn get(&self, ability: Ability) -> u8 {
        match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        }
    }

    pub fn set(&mut self, ability: Ability, value: u8) {
        match ability {
            Ability::Strength => self.strength = value,
            Ability::Dexterity => self.dexterity = value,
            Ability::Constitution => self.constitution = value,
            Ability::Intelligence => self.intelligence = value,
            Ability::Wisdom => self.wisdom = value,
            Ability::Charisma => self.charisma = value,
        }
    }

    pub fn modifier(&self, ability: Ability) -> i8 {
        let score = self.get(ability) as i8;
        // Use floor division to correctly handle negative numbers
        // D&D 5e: score 8-9 = -1, 10-11 = 0, 12-13 = +1, etc.
        (score - 10).div_euclid(2)
    }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10, 10)
    }
}
