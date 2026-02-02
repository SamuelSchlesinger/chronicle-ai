//! Hit points and health-related types.
//!
//! Contains types for tracking hit points, hit dice, and death saving throws.

use crate::dice::DieType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Hit Points and Health
// ============================================================================

/// Hit points tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: i32,
    pub maximum: i32,
    pub temporary: i32,
}

impl HitPoints {
    pub fn new(maximum: i32) -> Self {
        Self {
            current: maximum,
            maximum,
            temporary: 0,
        }
    }

    pub fn take_damage(&mut self, amount: i32) -> DamageResult {
        let mut remaining = amount;

        if self.temporary > 0 {
            if self.temporary >= remaining {
                self.temporary -= remaining;
                return DamageResult {
                    damage_taken: amount,
                    dropped_to_zero: false,
                };
            } else {
                remaining -= self.temporary;
                self.temporary = 0;
            }
        }

        self.current -= remaining;
        DamageResult {
            damage_taken: amount,
            dropped_to_zero: self.current <= 0,
        }
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let old = self.current;
        self.current = (self.current + amount).min(self.maximum);
        self.current - old
    }

    pub fn add_temp_hp(&mut self, amount: i32) {
        self.temporary = self.temporary.max(amount);
    }

    pub fn is_unconscious(&self) -> bool {
        self.current <= 0
    }

    pub fn ratio(&self) -> f32 {
        (self.current as f32 / self.maximum as f32).max(0.0)
    }
}

/// Result of taking damage.
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage_taken: i32,
    pub dropped_to_zero: bool,
}

/// Hit dice tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HitDice {
    pub total: HashMap<DieType, u8>,
    pub remaining: HashMap<DieType, u8>,
}

impl HitDice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, die_type: DieType, count: u8) {
        *self.total.entry(die_type).or_insert(0) += count;
        *self.remaining.entry(die_type).or_insert(0) += count;
    }

    pub fn spend(&mut self, die_type: DieType) -> bool {
        if let Some(remaining) = self.remaining.get_mut(&die_type) {
            if *remaining > 0 {
                *remaining -= 1;
                return true;
            }
        }
        false
    }

    pub fn recover_half(&mut self) {
        for (die_type, total) in &self.total {
            let to_recover = (*total as f32 / 2.0).ceil() as u8;
            if let Some(remaining) = self.remaining.get_mut(die_type) {
                *remaining = (*remaining + to_recover).min(*total);
            }
        }
    }
}

/// Death saving throws.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeathSaves {
    pub successes: u8,
    pub failures: u8,
}

impl DeathSaves {
    pub fn add_success(&mut self) -> bool {
        self.successes += 1;
        self.successes >= 3
    }

    pub fn add_failure(&mut self) -> bool {
        self.failures += 1;
        self.failures >= 3
    }

    pub fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
    }
}
