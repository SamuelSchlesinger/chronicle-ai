//! D&D 5e conditions system.
//!
//! This module defines the standard conditions from the SRD 5.2,
//! including effects like Blinded, Charmed, Frightened, etc.

use std::fmt;

use serde::{Deserialize, Serialize};

// ============================================================================
// Conditions
// ============================================================================

/// D&D 5e conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
    Exhaustion(u8),
}

impl Condition {
    pub fn name(&self) -> &'static str {
        match self {
            Condition::Blinded => "Blinded",
            Condition::Charmed => "Charmed",
            Condition::Deafened => "Deafened",
            Condition::Frightened => "Frightened",
            Condition::Grappled => "Grappled",
            Condition::Incapacitated => "Incapacitated",
            Condition::Invisible => "Invisible",
            Condition::Paralyzed => "Paralyzed",
            Condition::Petrified => "Petrified",
            Condition::Poisoned => "Poisoned",
            Condition::Prone => "Prone",
            Condition::Restrained => "Restrained",
            Condition::Stunned => "Stunned",
            Condition::Unconscious => "Unconscious",
            Condition::Exhaustion(_) => "Exhaustion",
        }
    }

    pub fn is_incapacitating(&self) -> bool {
        matches!(
            self,
            Condition::Incapacitated
                | Condition::Paralyzed
                | Condition::Petrified
                | Condition::Stunned
                | Condition::Unconscious
        )
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Exhaustion(level) => write!(f, "Exhaustion ({level})"),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// A condition applied to a creature with tracking info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub condition: Condition,
    pub source: String,
    pub duration_rounds: Option<u32>,
}

impl ActiveCondition {
    pub fn new(condition: Condition, source: impl Into<String>) -> Self {
        Self {
            condition,
            source: source.into(),
            duration_rounds: None,
        }
    }

    pub fn with_duration(mut self, rounds: u32) -> Self {
        self.duration_rounds = Some(rounds);
        self
    }
}
