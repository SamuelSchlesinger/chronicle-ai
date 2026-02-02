//! D&D 5e skills and proficiency levels.
//!
//! This module contains the skill definitions and proficiency tracking
//! for the D&D 5e rules system.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::Ability;

/// D&D 5e skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Athletics,
    Acrobatics,
    SleightOfHand,
    Stealth,
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl Skill {
    pub fn ability(&self) -> Ability {
        match self {
            Skill::Athletics => Ability::Strength,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Ability::Dexterity,
            Skill::Arcana
            | Skill::History
            | Skill::Investigation
            | Skill::Nature
            | Skill::Religion => Ability::Intelligence,
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Perception
            | Skill::Survival => Ability::Wisdom,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => {
                Ability::Charisma
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Skill::Athletics => "Athletics",
            Skill::Acrobatics => "Acrobatics",
            Skill::SleightOfHand => "Sleight of Hand",
            Skill::Stealth => "Stealth",
            Skill::Arcana => "Arcana",
            Skill::History => "History",
            Skill::Investigation => "Investigation",
            Skill::Nature => "Nature",
            Skill::Religion => "Religion",
            Skill::AnimalHandling => "Animal Handling",
            Skill::Insight => "Insight",
            Skill::Medicine => "Medicine",
            Skill::Perception => "Perception",
            Skill::Survival => "Survival",
            Skill::Deception => "Deception",
            Skill::Intimidation => "Intimidation",
            Skill::Performance => "Performance",
            Skill::Persuasion => "Persuasion",
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Proficiency level for skills/tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProficiencyLevel {
    #[default]
    None,
    Half,
    Proficient,
    Expertise,
}

impl ProficiencyLevel {
    pub fn bonus(&self, proficiency_bonus: i8) -> i8 {
        match self {
            ProficiencyLevel::None => 0,
            ProficiencyLevel::Half => proficiency_bonus / 2,
            ProficiencyLevel::Proficient => proficiency_bonus,
            ProficiencyLevel::Expertise => proficiency_bonus * 2,
        }
    }
}
