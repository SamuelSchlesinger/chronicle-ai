//! Location types for the D&D game world.
//!
//! Contains types for representing locations, location types, and connections
//! between locations in the game world.

use serde::{Deserialize, Serialize};

use super::{CharacterId, LocationId};

/// A location in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub location_type: LocationType,
    pub description: String,
    pub connections: Vec<LocationConnection>,
    pub npcs_present: Vec<CharacterId>,
    pub items: Vec<String>,
}

impl Location {
    pub fn new(name: impl Into<String>, location_type: LocationType) -> Self {
        Self {
            id: LocationId::new(),
            name: name.into(),
            location_type,
            description: String::new(),
            connections: Vec::new(),
            npcs_present: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LocationType {
    Wilderness,
    Town,
    City,
    Dungeon,
    Building,
    Room,
    Road,
    Cave,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConnection {
    pub destination_id: LocationId,
    pub destination_name: String,
    pub direction: Option<String>,
    pub travel_time_minutes: u32,
}
