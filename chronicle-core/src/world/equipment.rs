//! D&D 5e equipment system.
//!
//! This module contains types for inventory items, weapons, armor,
//! consumables, and the equipment slot system.

use serde::{Deserialize, Serialize};

use super::conditions::Condition;
use super::defense::ArmorType;

// ============================================================================
// Equipment
// ============================================================================

/// Inventory item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quantity: u32,
    pub weight: f32,
    pub value_gp: f32,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub magical: bool,
}

impl Item {
    /// Returns true if this item type can stack in inventory.
    /// Weapons, armor, and shields don't stack (each is a distinct item).
    /// Consumables and gear can stack.
    pub fn is_stackable(&self) -> bool {
        match self.item_type {
            ItemType::Weapon | ItemType::Armor | ItemType::Shield => false,
            ItemType::Wand | ItemType::Ring | ItemType::Wondrous => false, // Unique items
            ItemType::Potion
            | ItemType::Scroll
            | ItemType::Adventuring
            | ItemType::Tool
            | ItemType::Other => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Potion,
    Scroll,
    Wand,
    Ring,
    Wondrous,
    Adventuring,
    Tool,
    Other,
}

/// Character inventory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub items: Vec<Item>,
    /// Gold pieces (1 gp = 10 sp)
    pub gold: i32,
    /// Silver pieces (10 sp = 1 gp)
    pub silver: i32,
}

// ============================================================================
// Equipment System
// ============================================================================

/// Equipment slots for what's actively equipped.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub armor: Option<ArmorItem>,
    pub shield: Option<Item>,
    pub main_hand: Option<WeaponItem>,
    pub off_hand: Option<Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Armor with D&D 5e properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorItem {
    pub base: Item,
    pub armor_type: ArmorType,
    pub base_ac: u8,
    pub strength_requirement: Option<u8>,
    pub stealth_disadvantage: bool,
}

impl ArmorItem {
    pub fn new(name: impl Into<String>, armor_type: ArmorType, base_ac: u8) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.0,
                value_gp: 0.0,
                description: None,
                item_type: ItemType::Armor,
                magical: false,
            },
            armor_type,
            base_ac,
            strength_requirement: None,
            stealth_disadvantage: false,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.base.weight = weight;
        self
    }

    pub fn with_value(mut self, value_gp: f32) -> Self {
        self.base.value_gp = value_gp;
        self
    }

    pub fn with_strength_requirement(mut self, str_req: u8) -> Self {
        self.strength_requirement = Some(str_req);
        self
    }

    pub fn with_stealth_disadvantage(mut self) -> Self {
        self.stealth_disadvantage = true;
        self
    }

    pub fn magical(mut self) -> Self {
        self.base.magical = true;
        self
    }
}

/// Weapons with D&D 5e properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponItem {
    pub base: Item,
    pub damage_dice: String,
    pub damage_type: WeaponDamageType,
    pub properties: Vec<WeaponProperty>,
    pub range: Option<(u32, u32)>,
}

impl WeaponItem {
    pub fn new(
        name: impl Into<String>,
        damage_dice: impl Into<String>,
        damage_type: WeaponDamageType,
    ) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.0,
                value_gp: 0.0,
                description: None,
                item_type: ItemType::Weapon,
                magical: false,
            },
            damage_dice: damage_dice.into(),
            damage_type,
            properties: Vec::new(),
            range: None,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.base.weight = weight;
        self
    }

    pub fn with_value(mut self, value_gp: f32) -> Self {
        self.base.value_gp = value_gp;
        self
    }

    pub fn with_properties(mut self, properties: Vec<WeaponProperty>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_range(mut self, normal: u32, long: u32) -> Self {
        self.range = Some((normal, long));
        self
    }

    pub fn magical(mut self) -> Self {
        self.base.magical = true;
        self
    }

    pub fn is_finesse(&self) -> bool {
        self.properties.contains(&WeaponProperty::Finesse)
    }

    pub fn is_ranged(&self) -> bool {
        self.range.is_some() || self.properties.contains(&WeaponProperty::Thrown)
    }

    pub fn is_two_handed(&self) -> bool {
        self.properties.contains(&WeaponProperty::TwoHanded)
    }

    pub fn versatile_damage(&self) -> Option<&str> {
        for prop in &self.properties {
            if let WeaponProperty::Versatile(dice) = prop {
                return Some(dice);
            }
        }
        None
    }
}

/// Weapon damage type (separate from spell/effect damage types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponDamageType {
    Slashing,
    Piercing,
    Bludgeoning,
}

impl WeaponDamageType {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponDamageType::Slashing => "slashing",
            WeaponDamageType::Piercing => "piercing",
            WeaponDamageType::Bludgeoning => "bludgeoning",
        }
    }
}

/// Weapon properties per D&D 5e.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponProperty {
    Finesse,
    Light,
    Heavy,
    TwoHanded,
    Versatile(String),
    Thrown,
    Ammunition,
    Loading,
    Reach,
}

/// Consumable item effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    /// Healing potion - roll dice and add bonus
    Healing { dice: String, bonus: i32 },
    /// Restore a spell slot of the given level
    RestoreSpellSlot { level: u8 },
    /// Remove a condition
    RemoveCondition { condition: Condition },
    /// Grant a condition for a duration
    GrantCondition {
        condition: Condition,
        duration_rounds: u32,
    },
    /// Cast a spell from a scroll
    CastSpell { spell_name: String, level: u8 },
    /// Grant temporary hit points
    TemporaryHitPoints { amount: i32 },
    /// Grant advantage on a type of roll for duration
    GrantAdvantage {
        roll_type: String,
        duration_rounds: u32,
    },
}

/// A consumable item with its effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableItem {
    pub base: Item,
    pub effect: ConsumableEffect,
}

impl ConsumableItem {
    pub fn healing_potion(
        name: impl Into<String>,
        dice: impl Into<String>,
        bonus: i32,
        value_gp: f32,
    ) -> Self {
        Self {
            base: Item {
                name: name.into(),
                quantity: 1,
                weight: 0.5,
                value_gp,
                description: Some(
                    "A magical potion that restores health when consumed.".to_string(),
                ),
                item_type: ItemType::Potion,
                magical: true,
            },
            effect: ConsumableEffect::Healing {
                dice: dice.into(),
                bonus,
            },
        }
    }

    pub fn spell_scroll(spell_name: impl Into<String>, level: u8, value_gp: f32) -> Self {
        let spell_name_str: String = spell_name.into();
        let name = format!("Scroll of {spell_name_str}");
        Self {
            base: Item {
                name,
                quantity: 1,
                weight: 0.0,
                value_gp,
                description: Some("A magical scroll containing a spell.".to_string()),
                item_type: ItemType::Scroll,
                magical: true,
            },
            effect: ConsumableEffect::CastSpell {
                spell_name: spell_name_str,
                level,
            },
        }
    }
}

impl Inventory {
    pub fn total_weight(&self) -> f32 {
        self.items
            .iter()
            .map(|i| i.weight * i.quantity as f32)
            .sum()
    }

    /// Add an item to the inventory.
    /// Stackable items (potions, scrolls, adventuring gear, etc.) stack with existing items.
    /// Non-stackable items (weapons, armor, shields) are added as separate entries.
    pub fn add_item(&mut self, item: Item) {
        // Only stack stackable item types
        if item.is_stackable() {
            if let Some(existing) = self.items.iter_mut().find(|i| i.name == item.name) {
                existing.quantity += item.quantity;
                return;
            }
        }
        self.items.push(item);
    }

    /// Remove an item from the inventory. Returns true if successful.
    /// Name matching is case-insensitive.
    pub fn remove_item(&mut self, name: &str, quantity: u32) -> bool {
        let name_lower = name.to_lowercase();
        if let Some(idx) = self
            .items
            .iter()
            .position(|i| i.name.to_lowercase() == name_lower)
        {
            if self.items[idx].quantity >= quantity {
                self.items[idx].quantity -= quantity;
                if self.items[idx].quantity == 0 {
                    self.items.remove(idx);
                }
                return true;
            }
        }
        false
    }

    /// Find an item by name.
    pub fn find_item(&self, name: &str) -> Option<&Item> {
        self.items
            .iter()
            .find(|i| i.name.to_lowercase() == name.to_lowercase())
    }

    /// Find an item by name (mutable).
    pub fn find_item_mut(&mut self, name: &str) -> Option<&mut Item> {
        self.items
            .iter_mut()
            .find(|i| i.name.to_lowercase() == name.to_lowercase())
    }

    /// Check if the inventory contains an item.
    pub fn has_item(&self, name: &str) -> bool {
        self.find_item(name).is_some()
    }

    /// Adjust gold amount. Returns new total or error if insufficient funds.
    pub fn adjust_gold(&mut self, amount: i32) -> Result<i32, &'static str> {
        let new_total = self.gold + amount;
        if new_total < 0 {
            Err("Insufficient gold")
        } else {
            self.gold = new_total;
            Ok(self.gold)
        }
    }

    /// Adjust silver amount. Returns new total or error if insufficient funds.
    pub fn adjust_silver(&mut self, amount: i32) -> Result<i32, &'static str> {
        let new_total = self.silver + amount;
        if new_total < 0 {
            Err("Insufficient silver")
        } else {
            self.silver = new_total;
            Ok(self.silver)
        }
    }
}
