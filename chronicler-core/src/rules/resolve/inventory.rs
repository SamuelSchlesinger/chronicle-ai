//! Inventory management resolution methods.

use crate::rules::helpers::roll_with_fallback;
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{CharacterId, Condition, GameWorld, ItemType};

impl RulesEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_add_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        quantity: u32,
        _item_type: Option<&str>,
        _description: Option<&str>,
        _magical: bool,
        _weight: Option<f32>,
        _value_gp: Option<f32>,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if item already exists
        let existing_qty = character
            .inventory
            .find_item(item_name)
            .map(|i| i.quantity)
            .unwrap_or(0);
        let new_total = existing_qty + quantity;

        // Note: item_type, description, magical, weight, value_gp are passed through
        // but the actual item creation happens in apply_effect or could be enhanced
        // to look up standard items from the items database.

        let qty_str = if quantity > 1 {
            format!("{quantity} x ")
        } else {
            String::new()
        };

        Resolution::new(format!(
            "{} receives {}{} (now has {} total)",
            character.name, qty_str, item_name, new_total
        ))
        .with_effect(Effect::ItemAdded {
            item_name: item_name.to_string(),
            quantity,
            new_total,
        })
    }

    pub(crate) fn resolve_remove_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        quantity: u32,
    ) -> Resolution {
        let character = &world.player_character;

        if let Some(item) = character.inventory.find_item(item_name) {
            if item.quantity >= quantity {
                let remaining = item.quantity - quantity;
                let qty_str = if quantity > 1 {
                    format!("{quantity} x ")
                } else {
                    String::new()
                };
                Resolution::new(format!(
                    "{} loses {}{} ({} remaining)",
                    character.name, qty_str, item_name, remaining
                ))
                .with_effect(Effect::ItemRemoved {
                    item_name: item_name.to_string(),
                    quantity,
                    remaining,
                })
            } else {
                Resolution::new(format!(
                    "{} doesn't have enough {} (has {}, needs {})",
                    character.name, item_name, item.quantity, quantity
                ))
            }
        } else {
            Resolution::new(format!("{} doesn't have any {}", character.name, item_name))
        }
    }

    pub(crate) fn resolve_equip_item(&self, world: &GameWorld, item_name: &str) -> Resolution {
        let character = &world.player_character;

        if let Some(item) = character.inventory.find_item(item_name) {
            let slot = match item.item_type {
                ItemType::Weapon => "main_hand",
                ItemType::Armor => "armor",
                ItemType::Shield => "shield",
                _ => {
                    return Resolution::new(format!(
                        "{item_name} cannot be equipped (not a weapon, armor, or shield)"
                    ));
                }
            };

            // Check for two-handed weapon + shield conflict
            if slot == "shield" {
                if let Some(ref weapon) = character.equipment.main_hand {
                    if weapon.is_two_handed() {
                        return Resolution::new(format!(
                            "Cannot equip {} - {} requires two hands",
                            item_name, weapon.base.name
                        ));
                    }
                }
            }

            // Check for shield + two-handed weapon conflict
            if slot == "main_hand" {
                if let Some(db_weapon) = crate::items::get_weapon(item_name) {
                    if db_weapon.is_two_handed() && character.equipment.shield.is_some() {
                        return Resolution::new(format!(
                            "Cannot equip {item_name} - it requires two hands but a shield is equipped. Unequip the shield first."
                        ));
                    }
                }
            }

            // Check strength requirement for heavy armor
            if slot == "armor" {
                if let Some(db_armor) = crate::items::get_armor(item_name) {
                    if let Some(str_req) = db_armor.strength_requirement {
                        let char_str = character.ability_scores.strength;
                        if char_str < str_req {
                            return Resolution::new(format!(
                                "{} equips {} but doesn't meet the Strength {} requirement (has {}). Movement speed reduced by 10 feet.",
                                character.name, item_name, str_req, char_str
                            ))
                            .with_effect(Effect::ItemEquipped {
                                item_name: item_name.to_string(),
                                slot: slot.to_string(),
                            });
                        }
                    }
                }
            }

            Resolution::new(format!(
                "{} equips {} in {} slot",
                character.name, item_name, slot
            ))
            .with_effect(Effect::ItemEquipped {
                item_name: item_name.to_string(),
                slot: slot.to_string(),
            })
        } else {
            Resolution::new(format!(
                "{} doesn't have {} in their inventory",
                character.name, item_name
            ))
        }
    }

    pub(crate) fn resolve_unequip_item(&self, world: &GameWorld, slot: &str) -> Resolution {
        let character = &world.player_character;

        let item_name = match slot.to_lowercase().as_str() {
            "armor" => character
                .equipment
                .armor
                .as_ref()
                .map(|a| a.base.name.clone()),
            "shield" => character.equipment.shield.as_ref().map(|s| s.name.clone()),
            "main_hand" | "weapon" => character
                .equipment
                .main_hand
                .as_ref()
                .map(|w| w.base.name.clone()),
            "off_hand" => character
                .equipment
                .off_hand
                .as_ref()
                .map(|i| i.name.clone()),
            _ => {
                return Resolution::new(format!(
                    "Unknown equipment slot: {slot}. Valid slots: armor, shield, main_hand, off_hand"
                ));
            }
        };

        if let Some(name) = item_name {
            Resolution::new(format!("{} unequips {}", character.name, name)).with_effect(
                Effect::ItemUnequipped {
                    item_name: name,
                    slot: slot.to_string(),
                },
            )
        } else {
            Resolution::new(format!("Nothing equipped in {slot} slot"))
        }
    }

    pub(crate) fn resolve_use_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        _target_id: Option<CharacterId>,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters cannot use items themselves
        if character.has_condition(Condition::Unconscious) {
            return Resolution::new(format!(
                "{} is unconscious and cannot use items!",
                character.name
            ));
        }

        if let Some(item) = character.inventory.find_item(item_name) {
            // Check if it's a consumable type
            match item.item_type {
                ItemType::Potion => {
                    // Look up proper healing amount from database, fall back to basic potion
                    let (dice_expr, bonus) =
                        if let Some(potion) = crate::items::get_potion(item_name) {
                            match potion.effect {
                                crate::world::ConsumableEffect::Healing { ref dice, bonus } => {
                                    (dice.clone(), bonus)
                                }
                                _ => ("2d4".to_string(), 2),
                            }
                        } else {
                            ("2d4".to_string(), 2) // Default healing potion
                        };

                    let heal_expr = if bonus != 0 {
                        format!("{dice_expr}+{bonus}")
                    } else {
                        dice_expr
                    };
                    let heal_roll = roll_with_fallback(&heal_expr, "1d4");

                    Resolution::new(format!(
                        "{} drinks {} and heals for {} HP",
                        character.name, item_name, heal_roll.total
                    ))
                    .with_effect(Effect::ItemUsed {
                        item_name: item_name.to_string(),
                        result: format!("Healed {} HP", heal_roll.total),
                    })
                    .with_effect(Effect::HpChanged {
                        target_id: character.id,
                        amount: heal_roll.total,
                        new_current: (character.hit_points.current + heal_roll.total)
                            .min(character.hit_points.maximum),
                        new_max: character.hit_points.maximum,
                        dropped_to_zero: false,
                    })
                    .with_effect(Effect::ItemRemoved {
                        item_name: item_name.to_string(),
                        quantity: 1,
                        remaining: item.quantity.saturating_sub(1),
                    })
                }
                ItemType::Scroll => Resolution::new(format!(
                    "{} reads {} and it crumbles to dust",
                    character.name, item_name
                ))
                .with_effect(Effect::ItemUsed {
                    item_name: item_name.to_string(),
                    result: "Scroll consumed".to_string(),
                })
                .with_effect(Effect::ItemRemoved {
                    item_name: item_name.to_string(),
                    quantity: 1,
                    remaining: item.quantity.saturating_sub(1),
                }),
                _ => Resolution::new(format!("{item_name} is not a consumable item")),
            }
        } else {
            Resolution::new(format!(
                "{} doesn't have {} in their inventory",
                character.name, item_name
            ))
        }
    }

    pub(crate) fn resolve_adjust_gold(
        &self,
        world: &GameWorld,
        amount: i32,
        reason: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let new_total = character.inventory.gold + amount;

        if new_total < 0 {
            Resolution::new(format!(
                "{} doesn't have enough gold (has {} gp, needs {} gp)",
                character.name, character.inventory.gold, -amount
            ))
        } else {
            let action = if amount >= 0 { "gains" } else { "spends" };
            Resolution::new(format!(
                "{} {} {} gp {} (now has {} gp)",
                character.name,
                action,
                amount.abs(),
                reason,
                new_total
            ))
            .with_effect(Effect::GoldChanged {
                amount,
                new_total,
                reason: reason.to_string(),
            })
        }
    }

    pub(crate) fn resolve_adjust_silver(
        &self,
        world: &GameWorld,
        amount: i32,
        reason: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let new_total = character.inventory.silver + amount;

        if new_total < 0 {
            Resolution::new(format!(
                "{} doesn't have enough silver (has {} sp, needs {} sp)",
                character.name, character.inventory.silver, -amount
            ))
        } else {
            let action = if amount >= 0 { "gains" } else { "spends" };
            Resolution::new(format!(
                "{} {} {} sp {} (now has {} sp)",
                character.name,
                action,
                amount.abs(),
                reason,
                new_total
            ))
            .with_effect(Effect::SilverChanged {
                amount,
                new_total,
                reason: reason.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::rules::RulesEngine;
    use crate::world::{create_sample_fighter, Item, ItemType};

    // ========== Add Item Tests ==========

    #[test]
    fn test_add_item_new_item() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_add_item(&world, "Longsword", 1, None, None, false, None, None);

        assert!(resolution.narrative.contains("receives"));
        assert!(resolution.narrative.contains("Longsword"));
        assert!(resolution.narrative.contains("1 total"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemAdded { item_name, quantity: 1, new_total: 1 } if item_name == "Longsword")));
    }

    #[test]
    fn test_add_item_existing_item_stacks() {
        let mut character = create_sample_fighter("Roland");
        // Add an existing item to inventory
        character.inventory.items.push(Item {
            name: "Healing Potion".to_string(),
            quantity: 2,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_add_item(&world, "Healing Potion", 3, None, None, true, None, None);

        assert!(resolution.narrative.contains("5 total")); // 2 + 3 = 5
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemAdded { new_total: 5, .. })));
    }

    #[test]
    fn test_add_item_multiple_quantity() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_add_item(&world, "Arrow", 20, None, None, false, None, None);

        assert!(resolution.narrative.contains("20 x Arrow"));
        assert!(resolution.narrative.contains("20 total"));
    }

    // ========== Remove Item Tests ==========

    #[test]
    fn test_remove_item_success() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Healing Potion".to_string(),
            quantity: 3,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_remove_item(&world, "Healing Potion", 1);

        assert!(resolution.narrative.contains("loses"));
        assert!(resolution.narrative.contains("2 remaining"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::ItemRemoved {
                quantity: 1,
                remaining: 2,
                ..
            }
        )));
    }

    #[test]
    fn test_remove_item_insufficient() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Healing Potion".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_remove_item(&world, "Healing Potion", 5);

        assert!(resolution.narrative.contains("doesn't have enough"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_remove_item_not_found() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_remove_item(&world, "Nonexistent Item", 1);

        assert!(resolution.narrative.contains("doesn't have any"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Equip Item Tests ==========

    #[test]
    fn test_equip_item_weapon() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_equip_item(&world, "Longsword");

        assert!(resolution.narrative.contains("equips Longsword"));
        assert!(resolution.narrative.contains("main_hand"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemEquipped { slot, .. } if slot == "main_hand")));
    }

    #[test]
    fn test_equip_item_armor() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Chain Mail".to_string(),
            quantity: 1,
            weight: 55.0,
            value_gp: 75.0,
            description: None,
            item_type: ItemType::Armor,
            magical: false,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_equip_item(&world, "Chain Mail");

        assert!(resolution.narrative.contains("armor slot"));
    }

    #[test]
    fn test_equip_item_shield() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Shield".to_string(),
            quantity: 1,
            weight: 6.0,
            value_gp: 10.0,
            description: None,
            item_type: ItemType::Shield,
            magical: false,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_equip_item(&world, "Shield");

        assert!(resolution.narrative.contains("shield slot"));
    }

    #[test]
    fn test_equip_item_non_equippable() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Rope".to_string(),
            quantity: 1,
            weight: 10.0,
            value_gp: 1.0,
            description: None,
            item_type: ItemType::Adventuring,
            magical: false,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_equip_item(&world, "Rope");

        assert!(resolution.narrative.contains("cannot be equipped"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_equip_item_not_in_inventory() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_equip_item(&world, "Longsword");

        assert!(resolution.narrative.contains("doesn't have"));
        assert!(resolution.narrative.contains("in their inventory"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Unequip Item Tests ==========

    #[test]
    fn test_unequip_item_armor() {
        let mut character = create_sample_fighter("Roland");
        character.equipment.armor = Some(crate::world::ArmorItem::new(
            "Chain Mail",
            crate::world::ArmorType::Heavy,
            16,
        ));
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_unequip_item(&world, "armor");

        assert!(resolution.narrative.contains("unequips Chain Mail"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemUnequipped { slot, .. } if slot == "armor")));
    }

    #[test]
    fn test_unequip_item_weapon() {
        let mut character = create_sample_fighter("Roland");
        character.equipment.main_hand = Some(crate::world::WeaponItem::new(
            "Longsword",
            "1d8",
            crate::world::WeaponDamageType::Slashing,
        ));
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_unequip_item(&world, "main_hand");

        assert!(resolution.narrative.contains("unequips Longsword"));
    }

    #[test]
    fn test_unequip_item_empty_slot() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_unequip_item(&world, "armor");

        assert!(resolution.narrative.contains("Nothing equipped"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_unequip_item_invalid_slot() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_unequip_item(&world, "invalid_slot");

        assert!(resolution.narrative.contains("Unknown equipment slot"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Use Item Tests ==========

    #[test]
    fn test_use_item_potion_healing() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Potion of Healing".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_item(&world, "Potion of Healing", None);

        assert!(resolution.narrative.contains("drinks"));
        assert!(resolution.narrative.contains("heals for"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemUsed { .. })));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { .. })));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemRemoved { quantity: 1, .. })));
    }

    #[test]
    fn test_use_item_scroll() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Scroll of Fireball".to_string(),
            quantity: 1,
            weight: 0.0,
            value_gp: 200.0,
            description: None,
            item_type: ItemType::Scroll,
            magical: true,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_item(&world, "Scroll of Fireball", None);

        assert!(resolution.narrative.contains("reads"));
        assert!(resolution.narrative.contains("crumbles to dust"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ItemRemoved { quantity: 1, .. })));
    }

    #[test]
    fn test_use_item_non_consumable() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Longsword".to_string(),
            quantity: 1,
            weight: 3.0,
            value_gp: 15.0,
            description: None,
            item_type: ItemType::Weapon,
            magical: false,
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_item(&world, "Longsword", None);

        assert!(resolution.narrative.contains("not a consumable"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_item_not_in_inventory() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_item(&world, "Potion of Healing", None);

        assert!(resolution.narrative.contains("doesn't have"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_item_unconscious() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.items.push(Item {
            name: "Potion of Healing".to_string(),
            quantity: 1,
            weight: 0.5,
            value_gp: 50.0,
            description: None,
            item_type: ItemType::Potion,
            magical: true,
        });
        character.add_condition(Condition::Unconscious, "test");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_item(&world, "Potion of Healing", None);

        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution.narrative.contains("cannot use items"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Gold Adjustment Tests ==========

    #[test]
    fn test_adjust_gold_gain() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.gold = 100;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_gold(&world, 50, "for quest reward");

        assert!(resolution.narrative.contains("gains 50 gp"));
        assert!(resolution.narrative.contains("now has 150 gp"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::GoldChanged {
                amount: 50,
                new_total: 150,
                ..
            }
        )));
    }

    #[test]
    fn test_adjust_gold_spend() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.gold = 100;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_gold(&world, -50, "for supplies");

        assert!(resolution.narrative.contains("spends 50 gp"));
        assert!(resolution.narrative.contains("now has 50 gp"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::GoldChanged {
                amount: -50,
                new_total: 50,
                ..
            }
        )));
    }

    #[test]
    fn test_adjust_gold_insufficient() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.gold = 30;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_gold(&world, -50, "for purchase");

        assert!(resolution.narrative.contains("doesn't have enough gold"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Silver Adjustment Tests ==========

    #[test]
    fn test_adjust_silver_gain() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.silver = 50;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_silver(&world, 25, "for tips");

        assert!(resolution.narrative.contains("gains 25 sp"));
        assert!(resolution.narrative.contains("now has 75 sp"));
    }

    #[test]
    fn test_adjust_silver_spend() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.silver = 50;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_silver(&world, -20, "for drinks");

        assert!(resolution.narrative.contains("spends 20 sp"));
        assert!(resolution.narrative.contains("now has 30 sp"));
    }

    #[test]
    fn test_adjust_silver_insufficient() {
        let mut character = create_sample_fighter("Roland");
        character.inventory.silver = 10;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_adjust_silver(&world, -50, "for purchase");

        assert!(resolution.narrative.contains("doesn't have enough silver"));
        assert!(resolution.effects.is_empty());
    }
}
