//! D&D 5e character and NPC types.
//!
//! Contains the complete Character struct for player characters and the NPC struct
//! for non-player characters, along with supporting types like Race and Disposition.

use super::{
    Ability, AbilityScores, ActiveCondition, ArmorClass, ArmorType, Background, CharacterId,
    ClassLevel, ClassResources, Condition, DeathSaves, Equipment, Feature, HitDice, HitPoints,
    Inventory, LocationId, ProficiencyLevel, RaceType, Skill, Speed, SpellcastingData,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Character
// ============================================================================

/// D&D race (legacy struct for compatibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub subrace: Option<String>,
    #[serde(default)]
    pub race_type: Option<RaceType>,
}

/// Complete D&D 5e character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub player_name: Option<String>,

    // Core stats
    pub ability_scores: AbilityScores,
    pub level: u8,
    pub experience: u32,

    // Health
    pub hit_points: HitPoints,
    pub hit_dice: HitDice,
    pub death_saves: DeathSaves,

    // Combat
    pub armor_class: ArmorClass,
    pub speed: Speed,
    pub conditions: Vec<ActiveCondition>,

    // Class features
    pub classes: Vec<ClassLevel>,
    pub features: Vec<Feature>,
    pub class_resources: ClassResources,

    // Spellcasting
    pub spellcasting: Option<SpellcastingData>,

    // Skills & proficiencies
    pub skill_proficiencies: HashMap<Skill, ProficiencyLevel>,
    pub saving_throw_proficiencies: HashSet<Ability>,
    pub tool_proficiencies: HashSet<String>,
    pub languages: Vec<String>,

    // Equipment
    pub inventory: Inventory,
    pub equipment: Equipment,

    // Background and race
    pub race: Race,
    pub race_type: RaceType,
    pub background: Background,
    pub background_name: String, // For display/legacy

    // Player backstory
    pub backstory: Option<String>,
}

impl Character {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            player_name: None,
            ability_scores: AbilityScores::default(),
            level: 1,
            experience: 0,
            hit_points: HitPoints::new(10),
            hit_dice: HitDice::new(),
            death_saves: DeathSaves::default(),
            armor_class: ArmorClass::default(),
            speed: Speed::default(),
            conditions: Vec::new(),
            classes: Vec::new(),
            features: Vec::new(),
            class_resources: ClassResources::new(),
            spellcasting: None,
            skill_proficiencies: HashMap::new(),
            saving_throw_proficiencies: HashSet::new(),
            tool_proficiencies: HashSet::new(),
            languages: vec!["Common".to_string()],
            inventory: Inventory {
                items: Vec::new(),
                gold: 15, // Starting gold
                silver: 0,
            },
            equipment: Equipment::default(),
            race: Race {
                name: "Human".to_string(),
                subrace: None,
                race_type: Some(RaceType::Human),
            },
            race_type: RaceType::Human,
            background: Background::Soldier,
            background_name: "Soldier".to_string(),
            backstory: None,
        }
    }

    pub fn proficiency_bonus(&self) -> i8 {
        match self.level {
            0 => 2, // Invalid level, but default to minimum
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            // Level 17+ caps at proficiency bonus 6 (D&D 5e max level is 20)
            _ => 6,
        }
    }

    pub fn initiative_modifier(&self) -> i8 {
        self.ability_scores.modifier(Ability::Dexterity)
    }

    pub fn skill_modifier(&self, skill: Skill) -> i8 {
        let ability_mod = self.ability_scores.modifier(skill.ability());
        let proficiency = self
            .skill_proficiencies
            .get(&skill)
            .copied()
            .unwrap_or(ProficiencyLevel::None);
        ability_mod + proficiency.bonus(self.proficiency_bonus())
    }

    pub fn saving_throw_modifier(&self, ability: Ability) -> i8 {
        let ability_mod = self.ability_scores.modifier(ability);
        if self.saving_throw_proficiencies.contains(&ability) {
            ability_mod + self.proficiency_bonus()
        } else {
            ability_mod
        }
    }

    /// Calculate current AC from equipped armor and shield.
    ///
    /// If equipment is set, AC is calculated from equipped armor.
    /// Otherwise, falls back to the armor_class field for backwards compatibility.
    pub fn current_ac(&self) -> u8 {
        let dex_mod = self.ability_scores.modifier(Ability::Dexterity);

        // Calculate base AC from equipped armor or unarmored
        let base_ac = if let Some(ref armor) = self.equipment.armor {
            match armor.armor_type {
                ArmorType::Light => armor.base_ac as i8 + dex_mod,
                ArmorType::Medium => armor.base_ac as i8 + dex_mod.min(2),
                ArmorType::Heavy => armor.base_ac as i8,
            }
        } else if self.equipment.main_hand.is_some()
            || self.equipment.shield.is_some()
            || self.equipment.off_hand.is_some()
        {
            // Equipment is being used but no armor - unarmored defense
            10 + dex_mod
        } else {
            // No equipment set - use legacy armor_class field
            return self
                .armor_class
                .calculate(self.ability_scores.modifier(Ability::Dexterity));
        };

        // Add shield bonus if equipped
        let shield_bonus: i8 = if self.equipment.shield.is_some() {
            2
        } else {
            0
        };

        (base_ac + shield_bonus).max(1) as u8
    }

    pub fn is_conscious(&self) -> bool {
        self.hit_points.current > 0
    }

    /// Check if the character has a specific condition.
    pub fn has_condition(&self, condition: Condition) -> bool {
        self.conditions
            .iter()
            .any(|c| std::mem::discriminant(&c.condition) == std::mem::discriminant(&condition))
    }

    /// Add a condition if not already present. Returns true if the condition was added.
    pub fn add_condition(&mut self, condition: Condition, source: impl Into<String>) -> bool {
        self.add_condition_with_duration(condition, source, None)
    }

    /// Add a condition with optional duration. Returns true if the condition was added.
    pub fn add_condition_with_duration(
        &mut self,
        condition: Condition,
        source: impl Into<String>,
        duration_rounds: Option<u32>,
    ) -> bool {
        if self.has_condition(condition) {
            false
        } else {
            let mut active = ActiveCondition::new(condition, source);
            if let Some(duration) = duration_rounds {
                active = active.with_duration(duration);
            }
            self.conditions.push(active);
            true
        }
    }

    pub fn passive_perception(&self) -> i8 {
        10 + self.skill_modifier(Skill::Perception)
    }
}

// ============================================================================
// NPCs
// ============================================================================

/// An NPC in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub occupation: Option<String>,
    pub location_id: Option<LocationId>,
    pub disposition: Disposition,
    pub known_information: Vec<String>,
}

impl NPC {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            description: String::new(),
            personality: String::new(),
            occupation: None,
            location_id: None,
            disposition: Disposition::Neutral,
            known_information: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Helpful,
}
