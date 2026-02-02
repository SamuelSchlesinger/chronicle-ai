//! Quest and objective tracking for D&D campaigns.
//!
//! This module provides structures for managing quests, their objectives,
//! and completion status.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A quest or objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>,
    pub giver: Option<String>,
}

impl Quest {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            status: QuestStatus::Active,
            objectives: Vec::new(),
            rewards: Vec::new(),
            giver: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Active,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub completed: bool,
    pub optional: bool,
}
