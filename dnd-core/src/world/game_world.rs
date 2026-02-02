//! Game world state and management.
//!
//! Contains the complete game world state including the player character,
//! NPCs, locations, quests, combat state, and narrative history.
//!
//! Game mechanics (rest recovery, combat transitions) are implemented in the
//! [`mechanics`](super::mechanics) submodule and called from the methods here.

use super::{
    mechanics, Ability, ArmorType, Character, CharacterClass, CharacterId, ClassLevel, CombatState,
    Feature, FeatureUses, GameTime, HitPoints, Location, LocationId, LocationType,
    ProficiencyLevel, Quest, RechargeType, Skill, NPC,
};
use crate::dice::DieType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Game World
// ============================================================================

/// Current game mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GameMode {
    #[default]
    Exploration,
    Combat,
    Dialogue,
    Rest,
}

/// Entry in the narrative history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntry {
    pub content: String,
    pub entry_type: NarrativeType,
    pub game_time: GameTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NarrativeType {
    DmNarration,
    PlayerAction,
    NpcDialogue,
    Combat,
    System,
}

/// The complete game world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWorld {
    pub session_id: Uuid,
    pub campaign_name: String,

    // Player character
    pub player_character: Character,

    // NPCs
    pub npcs: HashMap<CharacterId, NPC>,

    // Current state
    pub mode: GameMode,
    pub combat: Option<CombatState>,
    pub current_location: Location,
    pub game_time: GameTime,

    // Locations
    pub known_locations: HashMap<LocationId, Location>,

    // Campaign progress
    pub quests: Vec<Quest>,
    pub narrative_history: Vec<NarrativeEntry>,
}

impl GameWorld {
    pub fn new(campaign_name: impl Into<String>, player_character: Character) -> Self {
        let starting_location = Location::new("Starting Location", LocationType::Town)
            .with_description("A quiet place where your adventure begins.");

        let mut known_locations = HashMap::new();
        known_locations.insert(starting_location.id, starting_location.clone());

        Self {
            session_id: Uuid::new_v4(),
            campaign_name: campaign_name.into(),
            player_character,
            npcs: HashMap::new(),
            mode: GameMode::Exploration,
            combat: None,
            current_location: starting_location,
            game_time: GameTime::default(),
            known_locations,
            quests: Vec::new(),
            narrative_history: Vec::new(),
        }
    }

    /// Start combat, transitioning to combat mode.
    ///
    /// Returns a mutable reference to the newly created combat state.
    pub fn start_combat(&mut self) -> &mut CombatState {
        mechanics::start_combat(self)
    }

    /// End combat, transitioning back to exploration mode.
    pub fn end_combat(&mut self) {
        mechanics::end_combat(self)
    }

    /// Advance to the next turn in combat.
    ///
    /// Returns the index of the new current combatant, or None if not in combat.
    pub fn next_turn(&mut self) -> Option<usize> {
        mechanics::next_turn(self)
    }

    /// Take a short rest (1 hour).
    ///
    /// - Warlocks recover all spell slots (Pact Magic)
    /// - Features that recharge on short rest are restored
    /// - Class-specific resources that recharge on short rest are restored
    pub fn short_rest(&mut self) {
        self.game_time.advance_hours(1);
        mechanics::apply_short_rest(&mut self.player_character);
    }

    /// Take a long rest (8 hours).
    ///
    /// - Full HP recovery
    /// - Remove Unconscious condition
    /// - Reduce exhaustion by 1 level
    /// - Recover half of total hit dice
    /// - Recover all spell slots
    /// - All features that recharge on short or long rest are restored
    /// - Class-specific resources that recharge on long rest are restored
    pub fn long_rest(&mut self) {
        self.game_time.advance_hours(8);
        mechanics::apply_long_rest(&mut self.player_character);
    }

    pub fn add_narrative(&mut self, content: String, entry_type: NarrativeType) {
        self.narrative_history.push(NarrativeEntry {
            content,
            entry_type,
            game_time: self.game_time.clone(),
        });
    }

    pub fn recent_narrative(&self, count: usize) -> Vec<&NarrativeEntry> {
        self.narrative_history.iter().rev().take(count).collect()
    }
}

/// Create a sample fighter character for testing.
pub fn create_sample_fighter(name: &str) -> Character {
    use super::{AbilityScores, ArmorClass};

    let mut character = Character::new(name);

    character.ability_scores = AbilityScores::new(16, 14, 14, 10, 12, 8);
    character.level = 3;
    character.hit_points = HitPoints::new(28);
    character.hit_dice.add(DieType::D10, 3);

    character.classes.push(ClassLevel {
        class: CharacterClass::Fighter,
        level: 3,
        subclass: Some("Champion".to_string()),
    });

    character
        .saving_throw_proficiencies
        .insert(Ability::Strength);
    character
        .saving_throw_proficiencies
        .insert(Ability::Constitution);

    character
        .skill_proficiencies
        .insert(Skill::Athletics, ProficiencyLevel::Proficient);
    character
        .skill_proficiencies
        .insert(Skill::Perception, ProficiencyLevel::Proficient);
    character
        .skill_proficiencies
        .insert(Skill::Intimidation, ProficiencyLevel::Proficient);

    character.armor_class = ArmorClass {
        base: 16,
        armor_type: Some(ArmorType::Heavy),
        shield_bonus: 2,
    };

    character.features.push(Feature {
        name: "Second Wind".to_string(),
        description: "Regain 1d10 + fighter level HP as bonus action".to_string(),
        source: "Fighter".to_string(),
        uses: Some(FeatureUses {
            current: 1,
            maximum: 1,
            recharge: RechargeType::ShortRest,
        }),
    });

    character.features.push(Feature {
        name: "Action Surge".to_string(),
        description: "Take one additional action on your turn".to_string(),
        source: "Fighter".to_string(),
        uses: Some(FeatureUses {
            current: 1,
            maximum: 1,
            recharge: RechargeType::ShortRest,
        }),
    });

    character
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{
        AbilityScores, ArmorItem, ArmorType, Inventory, Item, ItemType, WeaponDamageType,
        WeaponItem, WeaponProperty,
    };

    #[test]
    fn test_ability_modifier() {
        let scores = AbilityScores::new(16, 14, 12, 10, 8, 6);
        assert_eq!(scores.modifier(Ability::Strength), 3);
        assert_eq!(scores.modifier(Ability::Dexterity), 2);
        assert_eq!(scores.modifier(Ability::Constitution), 1);
        assert_eq!(scores.modifier(Ability::Intelligence), 0);
        assert_eq!(scores.modifier(Ability::Wisdom), -1);
        assert_eq!(scores.modifier(Ability::Charisma), -2);

        // Test odd scores below 10 (edge case for floor division)
        let odd_scores = AbilityScores::new(9, 7, 5, 11, 13, 15);
        assert_eq!(odd_scores.modifier(Ability::Strength), -1); // 9 -> -1
        assert_eq!(odd_scores.modifier(Ability::Dexterity), -2); // 7 -> -2
        assert_eq!(odd_scores.modifier(Ability::Constitution), -3); // 5 -> -3
        assert_eq!(odd_scores.modifier(Ability::Intelligence), 0); // 11 -> 0
        assert_eq!(odd_scores.modifier(Ability::Wisdom), 1); // 13 -> +1
        assert_eq!(odd_scores.modifier(Ability::Charisma), 2); // 15 -> +2
    }

    #[test]
    fn test_hit_points() {
        let mut hp = HitPoints::new(20);
        assert_eq!(hp.current, 20);

        hp.take_damage(5);
        assert_eq!(hp.current, 15);

        hp.heal(10);
        assert_eq!(hp.current, 20); // Capped at max

        hp.add_temp_hp(5);
        hp.take_damage(7);
        assert_eq!(hp.temporary, 0);
        assert_eq!(hp.current, 18);
    }

    #[test]
    fn test_character_proficiency() {
        let mut char = Character::new("Test");
        assert_eq!(char.proficiency_bonus(), 2);

        char.level = 5;
        assert_eq!(char.proficiency_bonus(), 3);

        char.level = 17;
        assert_eq!(char.proficiency_bonus(), 6);
    }

    #[test]
    fn test_sample_fighter() {
        let fighter = create_sample_fighter("Roland");
        assert_eq!(fighter.name, "Roland");
        assert_eq!(fighter.level, 3);
        assert_eq!(fighter.current_ac(), 18); // 16 base + 2 shield, no dex for heavy
    }

    #[test]
    fn test_game_world() {
        let character = create_sample_fighter("Test");
        let world = GameWorld::new("Test Campaign", character);
        assert_eq!(world.campaign_name, "Test Campaign");
        assert!(matches!(world.mode, GameMode::Exploration));
    }

    #[test]
    fn test_inventory_add_item() {
        let mut inventory = Inventory::default();
        assert!(inventory.items.is_empty());

        // Weapons don't stack - each is a separate item
        let sword = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        inventory.add_item(sword);

        assert_eq!(inventory.items.len(), 1);
        assert_eq!(inventory.find_item("Longsword").unwrap().quantity, 1);

        // Adding another weapon creates a second entry (weapons don't stack)
        let sword2 = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        inventory.add_item(sword2);

        assert_eq!(inventory.items.len(), 2); // Two separate swords

        // Stackable items (like potions) DO stack
        let potion1 = Item {
            name: "Healing Potion".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion1);
        assert_eq!(inventory.items.len(), 3);

        let potion2 = Item {
            name: "Healing Potion".to_string(),
            quantity: 2,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion2);
        assert_eq!(inventory.items.len(), 3); // Still 3 - potions stacked
        assert_eq!(inventory.find_item("Healing Potion").unwrap().quantity, 3);
    }

    #[test]
    fn test_inventory_remove_item() {
        let mut inventory = Inventory::default();
        let potion = Item {
            name: "Healing Potion".to_string(),
            quantity: 3,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        inventory.add_item(potion);

        assert!(inventory.remove_item("Healing Potion", 1));
        assert_eq!(inventory.find_item("Healing Potion").unwrap().quantity, 2);

        assert!(inventory.remove_item("Healing Potion", 2));
        assert!(inventory.find_item("Healing Potion").is_none());

        // Can't remove what you don't have
        assert!(!inventory.remove_item("Healing Potion", 1));
    }

    #[test]
    fn test_inventory_gold() {
        let mut inventory = Inventory {
            gold: 100,
            silver: 0,
            ..Default::default()
        };

        assert!(inventory.adjust_gold(50).is_ok());
        assert_eq!(inventory.gold, 150);

        assert!(inventory.adjust_gold(-100).is_ok());
        assert_eq!(inventory.gold, 50);

        // Can't go negative
        assert!(inventory.adjust_gold(-100).is_err());
        assert_eq!(inventory.gold, 50);
    }

    #[test]
    fn test_inventory_silver() {
        let mut inventory = Inventory {
            gold: 0,
            silver: 50,
            ..Default::default()
        };

        assert!(inventory.adjust_silver(25).is_ok());
        assert_eq!(inventory.silver, 75);

        assert!(inventory.adjust_silver(-50).is_ok());
        assert_eq!(inventory.silver, 25);

        // Can't go negative
        assert!(inventory.adjust_silver(-50).is_err());
        assert_eq!(inventory.silver, 25);
    }

    #[test]
    fn test_equipment_ac_calculation() {
        let mut character = Character::new("Test");
        character.ability_scores.dexterity = 16; // +3 DEX mod

        // Unarmored: 10 + DEX
        character.equipment.shield = Some(Item {
            name: "Shield".to_string(),
            quantity: 1,
            weight: 6.0,
            value_gp: 10.0,
            description: None,
            item_type: ItemType::Shield,
            magical: false,
        });
        // With shield but no armor: 10 + 3 + 2 = 15
        assert_eq!(character.current_ac(), 15);

        // Light armor: base + full DEX
        character.equipment.armor = Some(ArmorItem::new("Studded Leather", ArmorType::Light, 12));
        // 12 + 3 + 2 = 17
        assert_eq!(character.current_ac(), 17);

        // Medium armor: base + DEX (max 2)
        character.equipment.armor = Some(ArmorItem::new("Breastplate", ArmorType::Medium, 14));
        // 14 + 2 (capped) + 2 = 18
        assert_eq!(character.current_ac(), 18);

        // Heavy armor: base only
        character.equipment.armor = Some(ArmorItem::new("Plate Armor", ArmorType::Heavy, 18));
        // 18 + 0 + 2 = 20
        assert_eq!(character.current_ac(), 20);

        // Remove shield
        character.equipment.shield = None;
        assert_eq!(character.current_ac(), 18);
    }

    #[test]
    fn test_weapon_item() {
        let sword = WeaponItem::new("Longsword", "1d8", WeaponDamageType::Slashing)
            .with_properties(vec![WeaponProperty::Versatile("1d10".to_string())]);

        assert_eq!(sword.damage_dice, "1d8");
        assert!(!sword.is_finesse());
        assert!(!sword.is_two_handed());
        assert_eq!(sword.versatile_damage(), Some("1d10"));

        let rapier = WeaponItem::new("Rapier", "1d8", WeaponDamageType::Piercing)
            .with_properties(vec![WeaponProperty::Finesse]);
        assert!(rapier.is_finesse());

        // Two-handed weapons
        let greatsword = WeaponItem::new("Greatsword", "2d6", WeaponDamageType::Slashing)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded]);
        assert!(greatsword.is_two_handed());
    }

    #[test]
    fn test_item_stackability() {
        // Weapons don't stack
        let sword = Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        };
        assert!(!sword.is_stackable());

        // Armor doesn't stack
        let armor = Item {
            name: "Chain Mail".to_string(),
            quantity: 1,
            weight: 55.0,
            value_gp: 75.0,
            description: None,
            item_type: ItemType::Armor,
            magical: false,
        };
        assert!(!armor.is_stackable());

        // Potions stack
        let potion = Item {
            name: "Healing Potion".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        };
        assert!(potion.is_stackable());

        // Adventuring gear stacks
        let rope = Item {
            name: "Rope".to_string(),
            quantity: 1,
            weight: 10.0,
            value_gp: 1.0,
            description: None,
            item_type: ItemType::Adventuring,
            magical: false,
        };
        assert!(rope.is_stackable());
    }

    #[test]
    fn test_character_backstory() {
        // New character should have no backstory
        let character = Character::new("Test");
        assert!(character.backstory.is_none());

        // Can set backstory
        let mut character = Character::new("Test");
        character.backstory = Some("A wandering adventurer seeking glory.".to_string());
        assert!(character.backstory.is_some());
        assert_eq!(
            character.backstory.as_ref().unwrap(),
            "A wandering adventurer seeking glory."
        );
    }
}
