//! D&D 5e game engine with AI Dungeon Master.
//!
//! This crate provides:
//! - Complete D&D 5e game mechanics
//! - AI-powered Dungeon Master using Claude
//! - Intent/Effect rules system for deterministic game state
//! - Campaign persistence
//!
//! # Quick Start
//!
//! ```ignore
//! use chronicle_core::{GameSession, SessionConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SessionConfig::new("My Campaign")
//!         .with_character_name("Thorin");
//!
//!     let mut session = GameSession::new(config).await?;
//!
//!     let response = session.player_action("I look around the tavern").await?;
//!     println!("{}", response.narrative);
//!
//!     session.save("my_campaign.json").await?;
//!     Ok(())
//! }
//! ```

pub mod character_builder;
pub mod class_data;
pub mod dice;
pub mod dm;
pub mod headless;
pub mod items;
pub mod persist;
pub mod rules;
pub mod session;
pub mod spells;
pub mod testing;
pub mod world;

// Re-export for convenience
pub use chronicle_macros::Tool;

// Primary public API
pub use character_builder::{AbilityMethod, CharacterBuilder};
pub use headless::{HeadlessConfig, HeadlessGame};
pub use persist::{CharacterMetadata, CharacterSaveInfo, SavedCharacter};
pub use session::{GameSession, Response, SessionConfig, SessionError};
pub use testing::{MockDm, MockResponse, TestHarness};
pub use world::{Background, CharacterClass, RaceType};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    /// Roll dice using standard D&D notation
    #[derive(Tool, Deserialize)]
    #[tool(name = "roll_dice")]
    #[allow(dead_code)]
    struct RollDice {
        /// Dice notation like "2d6+3" or "1d20"
        notation: String,
        /// Optional purpose for the roll
        purpose: Option<String>,
    }

    #[test]
    fn test_tool_derive() {
        assert_eq!(RollDice::tool_name(), "roll_dice");
        assert_eq!(
            RollDice::tool_description(),
            "Roll dice using standard D&D notation"
        );
    }

    #[test]
    fn test_tool_schema() {
        let schema = RollDice::input_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["notation"]["type"], "string");
        assert_eq!(schema["properties"]["purpose"]["type"], "string");

        // notation should be required, purpose should not be (it's Option)
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "notation"));
        assert!(!required.iter().any(|v| v == "purpose"));
    }

    #[test]
    fn test_tool_as_tool() {
        let tool = RollDice::as_tool();
        assert_eq!(tool.name, "roll_dice");
        assert!(!tool.description.is_empty());
    }

    // Test: struct without explicit name attribute (snake_case default)
    /// Apply damage to a target
    #[derive(Tool, Deserialize)]
    #[allow(dead_code)]
    struct ApplyDamageToTarget {
        /// Amount of damage
        amount: i32,
    }

    #[test]
    fn test_tool_default_snake_case_name() {
        assert_eq!(ApplyDamageToTarget::tool_name(), "apply_damage_to_target");
    }

    // Test: various field types
    /// Tool with various field types
    #[derive(Tool, Deserialize)]
    #[tool(name = "various_types")]
    #[allow(dead_code)]
    struct VariousTypes {
        /// A string field
        text: String,
        /// An integer field
        count: i32,
        /// A boolean field
        enabled: bool,
        /// A float field
        multiplier: f64,
        /// An array field
        targets: Vec<String>,
        /// An optional integer
        limit: Option<i32>,
    }

    #[test]
    fn test_tool_various_field_types() {
        let schema = VariousTypes::input_schema();
        assert_eq!(schema["properties"]["text"]["type"], "string");
        assert_eq!(schema["properties"]["count"]["type"], "integer");
        assert_eq!(schema["properties"]["enabled"]["type"], "boolean");
        assert_eq!(schema["properties"]["multiplier"]["type"], "number");
        assert_eq!(schema["properties"]["targets"]["type"], "array");
        assert_eq!(schema["properties"]["targets"]["items"]["type"], "string");
        assert_eq!(schema["properties"]["limit"]["type"], "integer");

        // Required should include non-Option fields only
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "text"));
        assert!(required.iter().any(|v| v == "count"));
        assert!(required.iter().any(|v| v == "enabled"));
        assert!(required.iter().any(|v| v == "multiplier"));
        assert!(required.iter().any(|v| v == "targets"));
        assert!(!required.iter().any(|v| v == "limit")); // Option<i32> not required
    }

    // Test: field with rename attribute
    /// Tool with renamed field
    #[derive(Tool, Deserialize)]
    #[tool(name = "renamed_field_tool")]
    #[allow(dead_code)]
    struct RenamedFieldTool {
        /// The target name
        #[tool(rename = "target_name")]
        target: String,
    }

    #[test]
    fn test_tool_field_rename() {
        let schema = RenamedFieldTool::input_schema();
        // Should have "target_name" not "target"
        assert!(schema["properties"]["target_name"].is_object());
        assert!(schema["properties"]["target"].is_null());
    }

    // Test: explicit optional attribute
    /// Tool with explicit optional
    #[derive(Tool, Deserialize)]
    #[tool(name = "explicit_optional")]
    #[allow(dead_code)]
    struct ExplicitOptional {
        /// Required field
        required_field: String,
        /// Explicitly marked optional (still String type, not Option)
        #[tool(optional)]
        optional_field: String,
    }

    #[test]
    fn test_tool_explicit_optional() {
        let schema = ExplicitOptional::input_schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "required_field"));
        assert!(!required.iter().any(|v| v == "optional_field"));
    }

    // Test: description from multi-line doc comments
    /// This is a tool
    /// with multi-line documentation.
    #[derive(Tool, Deserialize)]
    #[tool(name = "multi_doc")]
    #[allow(dead_code)]
    struct MultiLineDoc {
        /// Field docs
        field: String,
    }

    #[test]
    fn test_tool_multiline_description() {
        let desc = MultiLineDoc::tool_description();
        assert!(desc.contains("This is a tool"));
        assert!(desc.contains("multi-line documentation"));
    }

    // Test: integer subtypes
    /// Tool with integer subtypes
    #[derive(Tool, Deserialize)]
    #[tool(name = "int_types")]
    #[allow(dead_code)]
    struct IntegerTypes {
        i8_field: i8,
        i16_field: i16,
        i64_field: i64,
        u8_field: u8,
        u32_field: u32,
        usize_field: usize,
    }

    #[test]
    fn test_tool_integer_subtypes() {
        let schema = IntegerTypes::input_schema();
        assert_eq!(schema["properties"]["i8_field"]["type"], "integer");
        assert_eq!(schema["properties"]["i16_field"]["type"], "integer");
        assert_eq!(schema["properties"]["i64_field"]["type"], "integer");
        assert_eq!(schema["properties"]["u8_field"]["type"], "integer");
        assert_eq!(schema["properties"]["u32_field"]["type"], "integer");
        assert_eq!(schema["properties"]["usize_field"]["type"], "integer");
    }
}
