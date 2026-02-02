//! Combat state tracking for D&D 5e.
//!
//! This module provides types for managing combat encounters, including
//! initiative tracking, combatant management, and turn order.

use serde::{Deserialize, Serialize};

use super::CharacterId;

/// Combat participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: CharacterId,
    pub name: String,
    pub initiative: i32,
    pub is_player: bool,
    pub is_ally: bool,
    pub current_hp: i32,
    pub max_hp: i32,
    pub armor_class: u8,
}

/// Combat state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub active: bool,
    pub round: u32,
    pub turn_index: usize,
    pub combatants: Vec<Combatant>,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            active: true,
            round: 1,
            turn_index: 0,
            combatants: Vec::new(),
        }
    }

    pub fn add_combatant(&mut self, combatant: Combatant) {
        self.combatants.push(combatant);
        self.combatants
            .sort_by_key(|c| std::cmp::Reverse(c.initiative));
    }

    pub fn current_combatant(&self) -> Option<&Combatant> {
        self.combatants.get(self.turn_index)
    }

    pub fn next_turn(&mut self) {
        self.turn_index += 1;
        if self.turn_index >= self.combatants.len() {
            self.turn_index = 0;
            self.round += 1;
        }
    }

    pub fn end_combat(&mut self) {
        self.active = false;
    }

    /// Update a combatant's HP
    pub fn update_combatant_hp(&mut self, id: CharacterId, new_hp: i32) {
        if let Some(combatant) = self.combatants.iter_mut().find(|c| c.id == id) {
            combatant.current_hp = new_hp;
        }
    }

    /// Get non-player combatants (enemies and allies)
    pub fn get_enemies(&self) -> Vec<&Combatant> {
        self.combatants.iter().filter(|c| !c.is_player).collect()
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}
