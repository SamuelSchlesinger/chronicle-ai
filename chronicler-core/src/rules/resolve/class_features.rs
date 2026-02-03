//! Class feature resolution methods.

use crate::rules::helpers::roll_with_fallback;
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{CharacterClass, CharacterId, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_use_rage(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if already raging
        if world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is already raging!", character.name));
        }

        // Check for rage uses remaining
        let rage_feature = character.features.iter().find(|f| f.name == "Rage");
        if let Some(feature) = rage_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no rage uses remaining! (Recovers on long rest)",
                        character.name
                    ));
                }
            }
        }

        // Determine rage damage bonus based on level
        let barbarian_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Barbarian)
            .map(|c| c.level)
            .unwrap_or(1);

        let rage_damage = match barbarian_level {
            1..=8 => 2,
            9..=15 => 3,
            _ => 4,
        };

        Resolution::new(format!(
            "{} enters a RAGE! Gains: advantage on STR checks/saves, +{} rage damage to melee attacks, resistance to bludgeoning/piercing/slashing damage. Cannot cast spells or concentrate while raging.",
            character.name, rage_damage
        ))
        .with_effect(Effect::RageStarted {
            character_id: world.player_character.id,
            damage_bonus: rage_damage,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Rage".to_string(),
            description: format!("Entered rage (1 minute, +{rage_damage} damage)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Rage".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_end_rage(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if !world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is not currently raging.", character.name));
        }

        let reason_text = match reason {
            "duration_expired" => "Rage ended (1 minute duration expired).",
            "unconscious" => "Rage ended (knocked unconscious).",
            "no_combat_action" => "Rage ended (turn ended without attacking or taking damage).",
            "voluntary" => "Rage ended voluntarily.",
            _ => "Rage ended.",
        };

        Resolution::new(format!("{}'s rage ends. {}", character.name, reason_text))
            .with_effect(Effect::RageEnded {
                character_id: world.player_character.id,
                reason: reason_text.to_string(),
            })
            .with_effect(Effect::ClassResourceUsed {
                character_name: character.name.clone(),
                resource_name: "Rage".to_string(),
                description: reason_text.to_string(),
            })
    }

    pub(crate) fn resolve_use_ki(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        ability: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        if resources.ki_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough ki points! Has {} but needs {}.",
                character.name, resources.ki_points, points
            ));
        }

        let ability_description = match ability {
            "flurry_of_blows" => "Flurry of Blows: Make two unarmed strikes as a bonus action.",
            "patient_defense" => "Patient Defense: Take the Dodge action as a bonus action.",
            "step_of_the_wind" => {
                "Step of the Wind: Disengage or Dash as a bonus action, jump distance doubled."
            }
            "stunning_strike" => "Stunning Strike: Target must make a CON save or be Stunned until the end of your next turn.",
            _ => ability,
        };

        Resolution::new(format!(
            "{} spends {} ki point{}. {}",
            character.name,
            points,
            if points == 1 { "" } else { "s" },
            ability_description
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Ki Points".to_string(),
            description: format!("Spent {points} ki for {ability}"),
        })
    }

    pub(crate) fn resolve_use_lay_on_hands(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        hp_amount: u32,
        cure_disease: bool,
        neutralize_poison: bool,
    ) -> Resolution {
        let character = &world.player_character;
        let pool = world.player_character.class_resources.lay_on_hands_pool;

        let total_cost =
            hp_amount + if cure_disease { 5 } else { 0 } + if neutralize_poison { 5 } else { 0 };

        if pool < total_cost {
            return Resolution::new(format!(
                "{} doesn't have enough in their Lay on Hands pool! Has {} HP but needs {}.",
                character.name, pool, total_cost
            ));
        }

        let mut effects_text = Vec::new();
        if hp_amount > 0 {
            effects_text.push(format!("restores {hp_amount} HP"));
        }
        if cure_disease {
            effects_text.push("cures one disease".to_string());
        }
        if neutralize_poison {
            effects_text.push("neutralizes one poison".to_string());
        }

        Resolution::new(format!(
            "{} uses Lay on Hands on {}: {}. ({} HP remaining in pool)",
            character.name,
            target_name,
            effects_text.join(", "),
            pool - total_cost
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Lay on Hands".to_string(),
            description: format!("Used {total_cost} points on {target_name}"),
        })
    }

    pub(crate) fn resolve_use_divine_smite(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        spell_slot_level: u8,
        target_is_undead_or_fiend: bool,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if they have spell slots available
        if let Some(ref spellcasting) = character.spellcasting {
            let slot_idx = spell_slot_level.saturating_sub(1) as usize;
            if slot_idx < 9 {
                let slot = &spellcasting.spell_slots.slots[slot_idx];
                if slot.available() == 0 {
                    return Resolution::new(format!(
                        "{} has no level {} spell slots remaining!",
                        character.name, spell_slot_level
                    ));
                }
            }
        }

        // Calculate damage dice
        // Base: 2d8, +1d8 per slot level above 1st, max 5d8
        // Extra 1d8 vs undead/fiends
        let base_dice = 2 + (spell_slot_level.saturating_sub(1)).min(3);
        let total_dice = if target_is_undead_or_fiend {
            (base_dice + 1).min(6)
        } else {
            base_dice.min(5)
        };

        let damage_roll = roll_with_fallback(&format!("{total_dice}d8"), "2d8");

        let extra_text = if target_is_undead_or_fiend {
            " (extra damage vs undead/fiend)"
        } else {
            ""
        };

        Resolution::new(format!(
            "{} channels divine power into their strike! Divine Smite deals {}d8 = {} radiant damage{}. (Level {} slot expended)",
            character.name, total_dice, damage_roll.total, extra_text, spell_slot_level
        ))
        .with_effect(Effect::DiceRolled {
            roll: damage_roll,
            purpose: "Divine Smite damage".to_string(),
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Divine Smite".to_string(),
            description: format!("Used level {spell_slot_level} slot for smite"),
        })
    }

    pub(crate) fn resolve_use_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        beast_form: &str,
        beast_hp: i32,
        _beast_ac: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if already in Wild Shape
        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_some()
        {
            return Resolution::new(format!("{} is already in Wild Shape form!", character.name));
        }

        // Find Wild Shape feature uses
        let wild_shape_feature = character.features.iter().find(|f| f.name == "Wild Shape");
        if let Some(feature) = wild_shape_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Wild Shape uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        // Calculate duration based on Druid level
        let druid_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Druid)
            .map(|c| c.level)
            .unwrap_or(2);
        let duration_hours = druid_level / 2;

        Resolution::new(format!(
            "{} transforms into a {}! Beast form has {} HP. Duration: {} hour{}. Mental stats, proficiencies, and features retained. Cannot cast spells but can maintain concentration.",
            character.name, beast_form, beast_hp, duration_hours,
            if duration_hours == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: format!("Transformed into {beast_form} ({beast_hp} HP)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Wild Shape".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_end_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
        excess_damage: i32,
    ) -> Resolution {
        let character = &world.player_character;

        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_none()
        {
            return Resolution::new(format!(
                "{} is not currently in Wild Shape form.",
                character.name
            ));
        }

        let reason_text = match reason {
            "duration_expired" => "Wild Shape ended (duration expired).",
            "hp_zero" => {
                if excess_damage > 0 {
                    &format!(
                        "Wild Shape ended (beast HP dropped to 0). {} excess damage carries over to normal form!",
                        excess_damage
                    )
                } else {
                    "Wild Shape ended (beast HP dropped to 0)."
                }
            }
            "voluntary" => "Wild Shape ended voluntarily as a bonus action.",
            "incapacitated" => "Wild Shape ended (druid became incapacitated).",
            _ => "Wild Shape ended.",
        };

        let mut resolution = Resolution::new(format!(
            "{} reverts to their normal form. {}",
            character.name, reason_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: reason_text.to_string(),
        });

        // Apply excess damage if any
        if excess_damage > 0 {
            resolution = resolution.with_effect(Effect::HpChanged {
                target_id: world.player_character.id,
                amount: -excess_damage,
                new_current: (character.hit_points.current - excess_damage).max(0),
                new_max: character.hit_points.maximum,
                dropped_to_zero: character.hit_points.current - excess_damage <= 0,
            });
        }

        resolution
    }

    pub(crate) fn resolve_use_channel_divinity(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        option: &str,
        targets: &[String],
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Channel Divinity uses
        let cd_feature = character
            .features
            .iter()
            .find(|f| f.name == "Channel Divinity");
        if let Some(feature) = cd_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Channel Divinity uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        let option_description = match option.to_lowercase().as_str() {
            "turn undead" => {
                "Turn Undead: Each undead within 30 feet must make a WIS save. On failure, they must spend their turns moving away and cannot take reactions for 1 minute."
            }
            "divine spark" => {
                "Divine Spark: Either deal 1d8 radiant damage to one creature within 30 feet (DEX save for half), or restore 1d8 HP to one creature within 30 feet."
            }
            "sacred weapon" => {
                "Sacred Weapon: Your weapon becomes magical for 1 minute, +CHA to attack rolls, and sheds bright light."
            }
            _ => option,
        };

        let targets_text = if targets.is_empty() {
            String::new()
        } else {
            format!(" Targets: {}.", targets.join(", "))
        };

        Resolution::new(format!(
            "{} uses Channel Divinity: {}.{}",
            character.name, option_description, targets_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Channel Divinity".to_string(),
            description: option.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Channel Divinity".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_bardic_inspiration(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        die_size: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Bardic Inspiration uses
        let bi_feature = character
            .features
            .iter()
            .find(|f| f.name == "Bardic Inspiration");
        if let Some(feature) = bi_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Bardic Inspiration uses remaining! (Recovers on long rest, or short rest at level 5+)",
                        character.name
                    ));
                }
            }
        }

        Resolution::new(format!(
            "{} inspires {} with a rousing performance! {} gains a {} Bardic Inspiration die they can add to one ability check, attack roll, or saving throw within the next 10 minutes.",
            character.name, target_name, target_name, die_size
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Bardic Inspiration".to_string(),
            description: format!("Inspired {target_name} with a {die_size}"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Bardic Inspiration".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_action_surge(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        action_taken: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.action_surge_used {
            return Resolution::new(format!(
                "{} has already used Action Surge! (Recovers on short/long rest)",
                character.name
            ));
        }

        Resolution::new(format!(
            "{} surges with renewed vigor! Takes an additional action this turn: {}",
            character.name, action_taken
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Action Surge".to_string(),
            description: action_taken.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Action Surge".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_second_wind(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
    ) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.second_wind_used {
            return Resolution::new(format!(
                "{} has already used Second Wind! (Recovers on short/long rest)",
                character.name
            ));
        }

        // Calculate healing: 1d10 + fighter level
        let fighter_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Fighter)
            .map(|c| c.level)
            .unwrap_or(1);

        let healing_roll = roll_with_fallback(&format!("1d10+{fighter_level}"), "1d10+1");
        let healing = healing_roll.total;

        let new_hp = (character.hit_points.current + healing).min(character.hit_points.maximum);

        Resolution::new(format!(
            "{} catches their breath with Second Wind! Regains 1d10+{} = {} HP. (Now at {}/{})",
            character.name, fighter_level, healing, new_hp, character.hit_points.maximum
        ))
        .with_effect(Effect::DiceRolled {
            roll: healing_roll,
            purpose: "Second Wind healing".to_string(),
        })
        .with_effect(Effect::HpChanged {
            target_id: world.player_character.id,
            amount: healing,
            new_current: new_hp,
            new_max: character.hit_points.maximum,
            dropped_to_zero: false,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Second Wind".to_string(),
            description: format!("Healed {healing} HP"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Second Wind".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_sorcery_points(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        metamagic: &str,
        spell_name: Option<&str>,
        slot_level: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        // Handle slot conversion separately
        if metamagic == "convert_to_slot" {
            if let Some(level) = slot_level {
                let cost = level; // Costs spell level points to create a slot
                if resources.sorcery_points < cost {
                    return Resolution::new(format!(
                        "{} doesn't have enough sorcery points! Has {} but needs {} to create a level {} slot.",
                        character.name, resources.sorcery_points, cost, level
                    ));
                }
                return Resolution::new(format!(
                    "{} converts {} sorcery points into a level {} spell slot.",
                    character.name, cost, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Created level {level} spell slot"),
                });
            }
        }

        if metamagic == "convert_from_slot" {
            if let Some(level) = slot_level {
                return Resolution::new(format!(
                    "{} converts a level {} spell slot into {} sorcery points.",
                    character.name, level, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Gained {level} points from slot"),
                });
            }
        }

        // Regular Metamagic usage
        if resources.sorcery_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough sorcery points! Has {} but needs {}.",
                character.name, resources.sorcery_points, points
            ));
        }

        let metamagic_description = match metamagic.to_lowercase().as_str() {
            "careful" => "Careful Spell: Protect allies from your spell's area effect.",
            "distant" => "Distant Spell: Double the spell's range (or 30 ft if touch).",
            "empowered" => "Empowered Spell: Reroll up to CHA mod damage dice.",
            "extended" => "Extended Spell: Double the spell's duration (max 24 hours).",
            "heightened" => "Heightened Spell: Target has disadvantage on first save.",
            "quickened" => "Quickened Spell: Cast as a bonus action instead of an action.",
            "subtle" => "Subtle Spell: Cast without verbal or somatic components.",
            "twinned" => "Twinned Spell: Target a second creature with a single-target spell.",
            _ => metamagic,
        };

        let spell_text = spell_name.map_or(String::new(), |s| format!(" on {}", s));

        Resolution::new(format!(
            "{} uses {}{} ({} sorcery point{}).",
            character.name,
            metamagic_description,
            spell_text,
            points,
            if points == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Sorcery Points".to_string(),
            description: format!("Used {points} for {metamagic}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::world::{
        create_sample_barbarian, create_sample_bard, create_sample_cleric, create_sample_druid,
        create_sample_fighter, create_sample_monk, create_sample_paladin, create_sample_sorcerer,
    };

    // ========== Rage Tests (Barbarian) ==========

    #[test]
    fn test_use_rage_success() {
        let character = create_sample_barbarian("Conan");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_rage(&world, world.player_character.id);

        assert!(resolution.narrative.contains("enters a RAGE"));
        assert!(resolution.narrative.contains("+2 rage damage"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::RageStarted {
                damage_bonus: 2,
                ..
            }
        )));
    }

    #[test]
    fn test_use_rage_already_raging() {
        let mut character = create_sample_barbarian("Conan");
        character.class_resources.rage_active = true;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_rage(&world, world.player_character.id);

        assert!(resolution.narrative.contains("already raging"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_rage_no_uses_remaining() {
        let mut character = create_sample_barbarian("Conan");
        // Set rage uses to 0
        for feature in &mut character.features {
            if feature.name == "Rage" {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = 0;
                }
            }
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_rage(&world, world.player_character.id);

        assert!(resolution.narrative.contains("no rage uses remaining"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_end_rage_success() {
        let mut character = create_sample_barbarian("Conan");
        character.class_resources.rage_active = true;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_end_rage(&world, world.player_character.id, "voluntary");

        assert!(resolution.narrative.contains("rage ends"));
        assert!(resolution.narrative.contains("voluntarily"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::RageEnded { .. })));
    }

    #[test]
    fn test_end_rage_not_raging() {
        let character = create_sample_barbarian("Conan");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_end_rage(&world, world.player_character.id, "voluntary");

        assert!(resolution.narrative.contains("not currently raging"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_end_rage_various_reasons() {
        let mut character = create_sample_barbarian("Conan");
        character.class_resources.rage_active = true;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let reasons = [
            ("duration_expired", "1 minute duration expired"),
            ("unconscious", "knocked unconscious"),
            ("no_combat_action", "without attacking or taking damage"),
        ];

        for (reason, expected_text) in reasons {
            let resolution = engine.resolve_end_rage(&world, world.player_character.id, reason);
            assert!(
                resolution.narrative.contains(expected_text),
                "Failed for reason: {}",
                reason
            );
        }
    }

    // ========== Ki Tests (Monk) ==========

    #[test]
    fn test_use_ki_success() {
        let character = create_sample_monk("Lee");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_ki(&world, world.player_character.id, 1, "flurry_of_blows");

        assert!(resolution.narrative.contains("spends 1 ki point"));
        assert!(resolution.narrative.contains("Flurry of Blows"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ClassResourceUsed { resource_name, .. } if resource_name == "Ki Points")));
    }

    #[test]
    fn test_use_ki_insufficient_points() {
        let mut character = create_sample_monk("Lee");
        character.class_resources.ki_points = 0;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_ki(&world, world.player_character.id, 1, "flurry_of_blows");

        assert!(resolution.narrative.contains("doesn't have enough ki"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_ki_various_abilities() {
        let character = create_sample_monk("Lee");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let abilities = [
            ("patient_defense", "Patient Defense"),
            ("step_of_the_wind", "Step of the Wind"),
            ("stunning_strike", "Stunning Strike"),
        ];

        for (ability, expected_text) in abilities {
            let resolution = engine.resolve_use_ki(&world, world.player_character.id, 1, ability);
            assert!(
                resolution.narrative.contains(expected_text),
                "Failed for ability: {}",
                ability
            );
        }
    }

    // ========== Lay on Hands Tests (Paladin) ==========

    #[test]
    fn test_use_lay_on_hands_heal_only() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_lay_on_hands(
            &world,
            world.player_character.id,
            "Wounded Ally",
            10,
            false,
            false,
        );

        assert!(resolution.narrative.contains("Lay on Hands"));
        assert!(resolution.narrative.contains("restores 10 HP"));
        assert!(resolution.narrative.contains("5 HP remaining"));
    }

    #[test]
    fn test_use_lay_on_hands_cure_disease() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_lay_on_hands(
            &world,
            world.player_character.id,
            "Diseased Ally",
            0,
            true,
            false,
        );

        assert!(resolution.narrative.contains("cures one disease"));
        assert!(resolution.narrative.contains("10 HP remaining")); // 15 - 5 = 10
    }

    #[test]
    fn test_use_lay_on_hands_neutralize_poison() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_lay_on_hands(
            &world,
            world.player_character.id,
            "Poisoned Ally",
            0,
            false,
            true,
        );

        assert!(resolution.narrative.contains("neutralizes one poison"));
    }

    #[test]
    fn test_use_lay_on_hands_insufficient_pool() {
        let mut character = create_sample_paladin("Arthur");
        character.class_resources.lay_on_hands_pool = 5;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_lay_on_hands(
            &world,
            world.player_character.id,
            "Ally",
            10,
            true,
            true,
        ); // 10 + 5 + 5 = 20 needed

        assert!(resolution.narrative.contains("doesn't have enough"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_lay_on_hands_combined() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_lay_on_hands(
            &world,
            world.player_character.id,
            "Ally",
            5,
            true,
            true,
        ); // 5 + 5 + 5 = 15 total

        assert!(resolution.narrative.contains("restores 5 HP"));
        assert!(resolution.narrative.contains("cures one disease"));
        assert!(resolution.narrative.contains("neutralizes one poison"));
        assert!(resolution.narrative.contains("0 HP remaining")); // 15 - 15 = 0
    }

    // ========== Divine Smite Tests (Paladin) ==========

    #[test]
    fn test_use_divine_smite_level_1_slot() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_divine_smite(&world, world.player_character.id, 1, false);

        assert!(resolution.narrative.contains("Divine Smite"));
        assert!(resolution.narrative.contains("2d8")); // Base is 2d8 at level 1
        assert!(resolution.narrative.contains("Level 1 slot expended"));
    }

    #[test]
    fn test_use_divine_smite_higher_slots() {
        let mut character = create_sample_paladin("Arthur");
        // Grant level 2 and 3 spell slots for testing
        if let Some(ref mut spellcasting) = character.spellcasting {
            spellcasting.spell_slots.slots[1] = crate::world::SlotInfo { total: 2, used: 0 }; // Level 2
            spellcasting.spell_slots.slots[2] = crate::world::SlotInfo { total: 2, used: 0 };
            // Level 3
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Level 2 slot = 3d8
        let resolution =
            engine.resolve_use_divine_smite(&world, world.player_character.id, 2, false);
        assert!(resolution.narrative.contains("3d8"));

        // Level 3 slot = 4d8
        let resolution =
            engine.resolve_use_divine_smite(&world, world.player_character.id, 3, false);
        assert!(resolution.narrative.contains("4d8"));
    }

    #[test]
    fn test_use_divine_smite_vs_undead() {
        let character = create_sample_paladin("Arthur");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_divine_smite(&world, world.player_character.id, 1, true);

        assert!(resolution.narrative.contains("3d8")); // 2d8 + 1d8 vs undead
        assert!(resolution.narrative.contains("extra damage vs undead"));
    }

    #[test]
    fn test_use_divine_smite_no_slots() {
        let mut character = create_sample_paladin("Arthur");
        // Use all spell slots
        if let Some(ref mut spellcasting) = character.spellcasting {
            spellcasting.spell_slots.slots[0].used = spellcasting.spell_slots.slots[0].total;
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_divine_smite(&world, world.player_character.id, 1, false);

        assert!(resolution.narrative.contains("no level 1 spell slots"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Wild Shape Tests (Druid) ==========

    #[test]
    fn test_use_wild_shape_success() {
        let character = create_sample_druid("Radagast");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_wild_shape(&world, world.player_character.id, "Wolf", 11, Some(13));

        assert!(resolution.narrative.contains("transforms into a Wolf"));
        assert!(resolution.narrative.contains("11 HP"));
        assert!(resolution.narrative.contains("1 hour")); // Level 3 druid = 1 hour
    }

    #[test]
    fn test_use_wild_shape_already_transformed() {
        let mut character = create_sample_druid("Radagast");
        character.class_resources.wild_shape_form = Some("Bear".to_string());
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_wild_shape(&world, world.player_character.id, "Wolf", 11, None);

        assert!(resolution.narrative.contains("already in Wild Shape"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_wild_shape_no_uses() {
        let mut character = create_sample_druid("Radagast");
        for feature in &mut character.features {
            if feature.name == "Wild Shape" {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = 0;
                }
            }
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_wild_shape(&world, world.player_character.id, "Wolf", 11, None);

        assert!(resolution
            .narrative
            .contains("no Wild Shape uses remaining"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_end_wild_shape_excess_damage() {
        let mut character = create_sample_druid("Radagast");
        character.class_resources.wild_shape_form = Some("Wolf".to_string());
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_end_wild_shape(&world, world.player_character.id, "hp_zero", 5);

        assert!(resolution
            .narrative
            .contains("reverts to their normal form"));
        assert!(resolution.narrative.contains("5 excess damage"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { amount: -5, .. })));
    }

    #[test]
    fn test_end_wild_shape_not_transformed() {
        let character = create_sample_druid("Radagast");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_end_wild_shape(&world, world.player_character.id, "voluntary", 0);

        assert!(resolution.narrative.contains("not currently in Wild Shape"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Channel Divinity Tests (Cleric/Paladin) ==========

    #[test]
    fn test_use_channel_divinity_turn_undead() {
        let character = create_sample_cleric("Brother Marcus");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_channel_divinity(
            &world,
            world.player_character.id,
            "Turn Undead",
            &["Zombie".to_string(), "Skeleton".to_string()],
        );

        assert!(resolution.narrative.contains("Turn Undead"));
        assert!(resolution.narrative.contains("Targets: Zombie, Skeleton"));
    }

    #[test]
    fn test_use_channel_divinity_divine_spark() {
        let character = create_sample_cleric("Brother Marcus");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_channel_divinity(
            &world,
            world.player_character.id,
            "Divine Spark",
            &[],
        );

        assert!(resolution.narrative.contains("Divine Spark"));
        assert!(resolution.narrative.contains("1d8"));
    }

    #[test]
    fn test_use_channel_divinity_no_uses() {
        let mut character = create_sample_cleric("Brother Marcus");
        for feature in &mut character.features {
            if feature.name == "Channel Divinity" {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = 0;
                }
            }
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_channel_divinity(
            &world,
            world.player_character.id,
            "Turn Undead",
            &[],
        );

        assert!(resolution
            .narrative
            .contains("no Channel Divinity uses remaining"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Bardic Inspiration Tests ==========

    #[test]
    fn test_use_bardic_inspiration_success() {
        let character = create_sample_bard("Dandelion");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_bardic_inspiration(&world, world.player_character.id, "Ally", "d6");

        assert!(resolution.narrative.contains("inspires Ally"));
        assert!(resolution.narrative.contains("d6 Bardic Inspiration"));
    }

    #[test]
    fn test_use_bardic_inspiration_no_uses() {
        let mut character = create_sample_bard("Dandelion");
        for feature in &mut character.features {
            if feature.name == "Bardic Inspiration" {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = 0;
                }
            }
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_bardic_inspiration(&world, world.player_character.id, "Ally", "d6");

        assert!(resolution
            .narrative
            .contains("no Bardic Inspiration uses remaining"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Action Surge Tests (Fighter) ==========

    #[test]
    fn test_use_action_surge_success() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_action_surge(&world, world.player_character.id, "Attack action");

        assert!(resolution.narrative.contains("surges with renewed vigor"));
        assert!(resolution.narrative.contains("Attack action"));
    }

    #[test]
    fn test_use_action_surge_already_used() {
        let mut character = create_sample_fighter("Roland");
        character.class_resources.action_surge_used = true;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_action_surge(&world, world.player_character.id, "Attack action");

        assert!(resolution.narrative.contains("already used Action Surge"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Second Wind Tests (Fighter) ==========

    #[test]
    fn test_use_second_wind_success() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_second_wind(&world, world.player_character.id);

        assert!(resolution.narrative.contains("Second Wind"));
        assert!(resolution.narrative.contains("1d10+3")); // Level 3 fighter
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { .. })));
    }

    #[test]
    fn test_use_second_wind_already_used() {
        let mut character = create_sample_fighter("Roland");
        character.class_resources.second_wind_used = true;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_second_wind(&world, world.player_character.id);

        assert!(resolution.narrative.contains("already used Second Wind"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Sorcery Points Tests (Sorcerer) ==========

    #[test]
    fn test_use_sorcery_points_metamagic() {
        let character = create_sample_sorcerer("Vex");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_sorcery_points(
            &world,
            world.player_character.id,
            2,
            "quickened",
            Some("Fireball"),
            None,
        );

        assert!(resolution.narrative.contains("Quickened Spell"));
        assert!(resolution.narrative.contains("on Fireball"));
        assert!(resolution.narrative.contains("2 sorcery points"));
    }

    #[test]
    fn test_use_sorcery_points_convert_to_slot() {
        let character = create_sample_sorcerer("Vex");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_sorcery_points(
            &world,
            world.player_character.id,
            2,
            "convert_to_slot",
            None,
            Some(2),
        );

        assert!(resolution.narrative.contains("converts 2 sorcery points"));
        assert!(resolution.narrative.contains("level 2 spell slot"));
    }

    #[test]
    fn test_use_sorcery_points_convert_from_slot() {
        let character = create_sample_sorcerer("Vex");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_sorcery_points(
            &world,
            world.player_character.id,
            0,
            "convert_from_slot",
            None,
            Some(2),
        );

        assert!(resolution
            .narrative
            .contains("converts a level 2 spell slot"));
        assert!(resolution.narrative.contains("2 sorcery points"));
    }

    #[test]
    fn test_use_sorcery_points_insufficient() {
        let mut character = create_sample_sorcerer("Vex");
        character.class_resources.sorcery_points = 0;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_use_sorcery_points(
            &world,
            world.player_character.id,
            2,
            "quickened",
            Some("Fireball"),
            None,
        );

        assert!(resolution
            .narrative
            .contains("doesn't have enough sorcery points"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_sorcery_points_various_metamagic() {
        let character = create_sample_sorcerer("Vex");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let metamagics = [
            ("careful", "Careful Spell"),
            ("distant", "Distant Spell"),
            ("empowered", "Empowered Spell"),
            ("extended", "Extended Spell"),
            ("heightened", "Heightened Spell"),
            ("subtle", "Subtle Spell"),
            ("twinned", "Twinned Spell"),
        ];

        for (metamagic, expected) in metamagics {
            let resolution = engine.resolve_use_sorcery_points(
                &world,
                world.player_character.id,
                1,
                metamagic,
                None,
                None,
            );
            assert!(
                resolution.narrative.contains(expected),
                "Failed for metamagic: {}",
                metamagic
            );
        }
    }
}
