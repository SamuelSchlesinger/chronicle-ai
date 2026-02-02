//! D&D 5e playable races.
//!
//! This module defines the playable races available in D&D 5e, including
//! their ability score bonuses, base speeds, and descriptions.

use super::AbilityScores;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Races
// ============================================================================

/// D&D 5e playable races.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RaceType {
    Human,
    Elf,
    Dwarf,
    Halfling,
    HalfOrc,
    HalfElf,
    Tiefling,
    Gnome,
    Dragonborn,
}

impl RaceType {
    pub fn name(&self) -> &'static str {
        match self {
            RaceType::Human => "Human",
            RaceType::Elf => "Elf",
            RaceType::Dwarf => "Dwarf",
            RaceType::Halfling => "Halfling",
            RaceType::HalfOrc => "Half-Orc",
            RaceType::HalfElf => "Half-Elf",
            RaceType::Tiefling => "Tiefling",
            RaceType::Gnome => "Gnome",
            RaceType::Dragonborn => "Dragonborn",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RaceType::Human => {
                "Versatile and ambitious, humans are the most adaptable of all races."
            }
            RaceType::Elf => "Graceful and long-lived, elves are masters of magic and artistry.",
            RaceType::Dwarf => {
                "Stout and hardy, dwarves are renowned craftsmen and fierce warriors."
            }
            RaceType::Halfling => {
                "Small but brave, halflings are known for their luck and stealth."
            }
            RaceType::HalfOrc => {
                "Strong and enduring, half-orcs combine human versatility with orcish might."
            }
            RaceType::HalfElf => "Charismatic and adaptable, half-elves bridge two worlds.",
            RaceType::Tiefling => {
                "Touched by infernal heritage, tieflings possess innate magical abilities."
            }
            RaceType::Gnome => {
                "Curious and inventive, gnomes are natural tinkers and illusionists."
            }
            RaceType::Dragonborn => "Proud and powerful, dragonborn carry the blood of dragons.",
        }
    }

    /// Apply racial ability score bonuses to base scores.
    pub fn apply_ability_bonuses(&self, scores: &mut AbilityScores) {
        match self {
            RaceType::Human => {
                scores.strength += 1;
                scores.dexterity += 1;
                scores.constitution += 1;
                scores.intelligence += 1;
                scores.wisdom += 1;
                scores.charisma += 1;
            }
            RaceType::Elf => {
                scores.dexterity += 2;
            }
            RaceType::Dwarf => {
                scores.constitution += 2;
            }
            RaceType::Halfling => {
                scores.dexterity += 2;
            }
            RaceType::HalfOrc => {
                scores.strength += 2;
                scores.constitution += 1;
            }
            RaceType::HalfElf => {
                scores.charisma += 2;
                // Note: Half-elves also get +1 to two other abilities of choice
                // This is handled in character builder
            }
            RaceType::Tiefling => {
                scores.charisma += 2;
                scores.intelligence += 1;
            }
            RaceType::Gnome => {
                scores.intelligence += 2;
            }
            RaceType::Dragonborn => {
                scores.strength += 2;
                scores.charisma += 1;
            }
        }
    }

    /// Get ability bonus description for display.
    pub fn ability_bonuses(&self) -> &'static str {
        match self {
            RaceType::Human => "+1 to all abilities",
            RaceType::Elf => "+2 Dexterity",
            RaceType::Dwarf => "+2 Constitution",
            RaceType::Halfling => "+2 Dexterity",
            RaceType::HalfOrc => "+2 Strength, +1 Constitution",
            RaceType::HalfElf => "+2 Charisma, +1 to two others",
            RaceType::Tiefling => "+2 Charisma, +1 Intelligence",
            RaceType::Gnome => "+2 Intelligence",
            RaceType::Dragonborn => "+2 Strength, +1 Charisma",
        }
    }

    pub fn base_speed(&self) -> u32 {
        match self {
            RaceType::Dwarf | RaceType::Halfling | RaceType::Gnome => 25,
            _ => 30,
        }
    }

    pub fn all() -> &'static [RaceType] {
        &[
            RaceType::Human,
            RaceType::Elf,
            RaceType::Dwarf,
            RaceType::Halfling,
            RaceType::HalfOrc,
            RaceType::HalfElf,
            RaceType::Tiefling,
            RaceType::Gnome,
            RaceType::Dragonborn,
        ]
    }
}

impl fmt::Display for RaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
