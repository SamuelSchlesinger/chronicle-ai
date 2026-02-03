//! D&D 5e game world types.
//!
//! Contains all types for representing game state: characters, locations,
//! NPCs, quests, combat, conditions, and the complete game world.
//!
//! This module is organized into submodules for different aspects of the game:
//! - [`abilities`]: Ability scores (Strength, Dexterity, etc.)
//! - [`skills`]: Skills and proficiency levels
//! - [`conditions`]: Status conditions (poisoned, stunned, etc.)
//! - [`health`]: Hit points, damage, hit dice, and death saves
//! - [`defense`]: Armor class, armor types, and speed
//! - [`classes`]: Character classes, levels, and features
//! - [`spellcasting`]: Spell slots and spellcasting data
//! - [`equipment`]: Items, inventory, weapons, and armor
//! - [`races`]: Character races
//! - [`backgrounds`]: Character backgrounds
//! - [`character`]: Character and NPC types
//! - [`locations`]: Locations and connections
//! - [`quests`]: Quests and objectives
//! - [`combat`]: Combat state and combatants
//! - [`time`]: In-game time tracking
//! - [`game_world`]: The complete game world state

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Submodules
// ============================================================================

mod abilities;
mod backgrounds;
mod character;
mod classes;
mod combat;
mod conditions;
mod defense;
mod equipment;
mod game_world;
mod health;
mod locations;
pub mod mechanics;
mod quests;
mod races;
mod skills;
mod spellcasting;
mod subclasses;
mod time;

// ============================================================================
// ID Types
// ============================================================================

/// Unique identifier for characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(pub Uuid);

impl CharacterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CharacterId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(pub Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Re-exports
// ============================================================================

// Abilities
pub use abilities::{Ability, AbilityScores};

// Skills
pub use skills::{ProficiencyLevel, Skill};

// Conditions
pub use conditions::{ActiveCondition, Condition};

// Health
pub use health::{DamageResult, DeathSaves, HitDice, HitPoints};

// Defense
pub use defense::{ArmorClass, ArmorType, Speed};

// Classes
pub use classes::{CharacterClass, ClassLevel, ClassResources, Feature, FeatureUses, RechargeType};

// Subclasses
pub use subclasses::{Subclass, SubclassFeature};

// Spellcasting
pub use spellcasting::{SlotInfo, SpellSlots, SpellcastingData};

// Equipment
pub use equipment::{
    ArmorItem, ConsumableEffect, ConsumableItem, Equipment, Inventory, Item, ItemType,
    WeaponDamageType, WeaponItem, WeaponProperty,
};

// Races
pub use races::RaceType;

// Backgrounds
pub use backgrounds::Background;

// Character
pub use character::{Character, Disposition, Race, NPC};

// Locations
pub use locations::{Location, LocationConnection, LocationType};

// Quests
pub use quests::{Quest, QuestObjective, QuestStatus};

// Combat
pub use combat::{CombatState, Combatant};

// Time
pub use time::GameTime;

// Game World
pub use game_world::{
    create_sample_barbarian, create_sample_bard, create_sample_cleric, create_sample_druid,
    create_sample_fighter, create_sample_monk, create_sample_paladin, create_sample_sorcerer,
    GameMode, GameWorld, NarrativeEntry, NarrativeType,
};
