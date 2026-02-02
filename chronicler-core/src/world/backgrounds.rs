//! D&D 5e character backgrounds.
//!
//! This module contains the background definitions for D&D 5e characters,
//! including skill proficiencies and descriptions.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::Skill;

/// D&D 5e character backgrounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Background {
    Acolyte,
    Charlatan,
    Criminal,
    Entertainer,
    FolkHero,
    GuildArtisan,
    Hermit,
    Noble,
    Outlander,
    Sage,
    Sailor,
    Soldier,
    Urchin,
}

impl Background {
    pub fn name(&self) -> &'static str {
        match self {
            Background::Acolyte => "Acolyte",
            Background::Charlatan => "Charlatan",
            Background::Criminal => "Criminal",
            Background::Entertainer => "Entertainer",
            Background::FolkHero => "Folk Hero",
            Background::GuildArtisan => "Guild Artisan",
            Background::Hermit => "Hermit",
            Background::Noble => "Noble",
            Background::Outlander => "Outlander",
            Background::Sage => "Sage",
            Background::Sailor => "Sailor",
            Background::Soldier => "Soldier",
            Background::Urchin => "Urchin",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Background::Acolyte => "You have spent your life in service to a temple.",
            Background::Charlatan => "You have always had a way with people.",
            Background::Criminal => "You have a history of breaking the law.",
            Background::Entertainer => "You thrive in front of an audience.",
            Background::FolkHero => {
                "You come from a humble background but are destined for greatness."
            }
            Background::GuildArtisan => "You are a member of an artisan's guild.",
            Background::Hermit => "You lived in seclusion for a formative part of your life.",
            Background::Noble => "You understand wealth, power, and privilege.",
            Background::Outlander => "You grew up in the wilds, far from civilization.",
            Background::Sage => "You spent years learning the lore of the multiverse.",
            Background::Sailor => "You sailed on a seagoing vessel for years.",
            Background::Soldier => "You trained as a soldier and served in a military.",
            Background::Urchin => "You grew up on the streets, alone and poor.",
        }
    }

    pub fn skill_proficiencies(&self) -> [Skill; 2] {
        match self {
            Background::Acolyte => [Skill::Insight, Skill::Religion],
            Background::Charlatan => [Skill::Deception, Skill::SleightOfHand],
            Background::Criminal => [Skill::Deception, Skill::Stealth],
            Background::Entertainer => [Skill::Acrobatics, Skill::Performance],
            Background::FolkHero => [Skill::AnimalHandling, Skill::Survival],
            Background::GuildArtisan => [Skill::Insight, Skill::Persuasion],
            Background::Hermit => [Skill::Medicine, Skill::Religion],
            Background::Noble => [Skill::History, Skill::Persuasion],
            Background::Outlander => [Skill::Athletics, Skill::Survival],
            Background::Sage => [Skill::Arcana, Skill::History],
            Background::Sailor => [Skill::Athletics, Skill::Perception],
            Background::Soldier => [Skill::Athletics, Skill::Intimidation],
            Background::Urchin => [Skill::SleightOfHand, Skill::Stealth],
        }
    }

    /// Returns the tool proficiencies granted by this background.
    pub fn tool_proficiencies(&self) -> Vec<&'static str> {
        match self {
            Background::Acolyte => vec![],
            Background::Charlatan => vec!["Disguise kit", "Forgery kit"],
            Background::Criminal => vec!["Thieves' tools", "Gaming set"],
            Background::Entertainer => vec!["Disguise kit", "Musical instrument"],
            Background::FolkHero => vec!["Artisan's tools", "Vehicles (land)"],
            Background::GuildArtisan => vec!["Artisan's tools"],
            Background::Hermit => vec!["Herbalism kit"],
            Background::Noble => vec!["Gaming set"],
            Background::Outlander => vec!["Musical instrument"],
            Background::Sage => vec![],
            Background::Sailor => vec!["Navigator's tools", "Vehicles (water)"],
            Background::Soldier => vec!["Gaming set", "Vehicles (land)"],
            Background::Urchin => vec!["Disguise kit", "Thieves' tools"],
        }
    }

    pub fn all() -> &'static [Background] {
        &[
            Background::Acolyte,
            Background::Charlatan,
            Background::Criminal,
            Background::Entertainer,
            Background::FolkHero,
            Background::GuildArtisan,
            Background::Hermit,
            Background::Noble,
            Background::Outlander,
            Background::Sage,
            Background::Sailor,
            Background::Soldier,
            Background::Urchin,
        ]
    }
}

impl fmt::Display for Background {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
