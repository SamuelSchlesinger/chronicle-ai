//! Spellcasting data structures and mechanics.
//!
//! This module contains types for tracking spellcasting abilities,
//! spell slots, and related functionality for D&D 5e characters.

use serde::{Deserialize, Serialize};

use super::{Ability, AbilityScores};

/// Spellcasting data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcastingData {
    pub ability: Ability,
    pub spells_known: Vec<String>,
    pub spells_prepared: Vec<String>,
    pub cantrips_known: Vec<String>,
    pub spell_slots: SpellSlots,
}

impl SpellcastingData {
    pub fn spell_save_dc(&self, ability_scores: &AbilityScores, proficiency: i8) -> u8 {
        let ability_mod = ability_scores.modifier(self.ability);
        (8 + proficiency + ability_mod).max(0) as u8
    }

    pub fn spell_attack_bonus(&self, ability_scores: &AbilityScores, proficiency: i8) -> i8 {
        ability_scores.modifier(self.ability) + proficiency
    }
}

/// Spell slot tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlots {
    pub slots: [SlotInfo; 9],
}

impl SpellSlots {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| SlotInfo { total: 0, used: 0 }),
        }
    }

    pub fn use_slot(&mut self, level: u8) -> bool {
        if (1..=9).contains(&level) {
            let slot = &mut self.slots[level as usize - 1];
            if slot.available() > 0 {
                slot.used += 1;
                return true;
            }
        }
        false
    }

    pub fn recover_all(&mut self) {
        for slot in &mut self.slots {
            slot.used = 0;
        }
    }
}

impl Default for SpellSlots {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlotInfo {
    pub total: u8,
    pub used: u8,
}

impl SlotInfo {
    pub fn available(&self) -> u8 {
        self.total.saturating_sub(self.used)
    }
}
