//! Skill checks, ability checks, saving throws, and dice rolls.

use crate::dice::{self, Advantage, DiceExpression};
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{Ability, CharacterId, Condition, GameWorld, Skill};

impl RulesEngine {
    pub(crate) fn resolve_skill_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious) {
            let ability = skill.ability();
            if matches!(ability, Ability::Strength | Ability::Dexterity) {
                return Resolution::new(format!(
                    "{} is unconscious and automatically fails the {} check!",
                    character.name,
                    skill.name()
                ))
                .with_effect(Effect::CheckFailed {
                    check_type: skill.name().to_string(),
                    roll: 0,
                    dc,
                });
            }
        }

        let modifier = character.skill_modifier(skill);

        // Check for armor-imposed stealth disadvantage
        let effective_advantage = if skill == Skill::Stealth {
            if let Some(ref armor) = character.equipment.armor {
                if armor.stealth_disadvantage {
                    // Armor imposes disadvantage on Stealth
                    advantage.combine(Advantage::Disadvantage)
                } else {
                    advantage
                }
            } else {
                advantage
            }
        } else {
            advantage
        };

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(effective_advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        // Note if stealth disadvantage was applied
        let disadvantage_note = if skill == Skill::Stealth
            && effective_advantage != advantage
            && matches!(effective_advantage, Advantage::Disadvantage)
        {
            " [armor disadvantage]"
        } else {
            ""
        };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {}){}",
            character.name,
            result_str,
            skill.name(),
            roll.total,
            dc,
            disadvantage_note
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", skill.name(), description),
        });

        if success {
            resolution = resolution.with_effect(Effect::CheckSucceeded {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        } else {
            resolution = resolution.with_effect(Effect::CheckFailed {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        }

        resolution
    }

    pub(crate) fn resolve_ability_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} check!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} check", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.ability_scores.modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", ability.abbreviation(), description),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        }
    }

    pub(crate) fn resolve_saving_throw(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity saving throws
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} saving throw!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.saving_throw_modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} on {} saving throw ({} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} save vs {}", ability.abbreviation(), source),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        }
    }

    pub(crate) fn resolve_roll_dice(&self, notation: &str, purpose: &str) -> Resolution {
        match dice::roll(notation) {
            Ok(roll) => Resolution::new(format!("Rolling {notation} for {purpose}: {roll}"))
                .with_effect(Effect::DiceRolled {
                    roll,
                    purpose: purpose.to_string(),
                }),
            Err(e) => Resolution::new(format!("Failed to roll {notation}: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::world::create_sample_fighter;

    // ========== Skill Check Tests ==========

    #[test]
    fn test_skill_check_produces_effects() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_skill_check(
            &world,
            world.player_character.id,
            Skill::Athletics,
            10,
            Advantage::Normal,
            "climbing a wall",
        );

        // Should have a dice roll effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));

        // Should have either success or failure effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::CheckSucceeded { .. } | Effect::CheckFailed { .. }
        )));
    }

    #[test]
    fn test_skill_check_unconscious_fails_str_dex() {
        let mut character = create_sample_fighter("Roland");
        character.add_condition(Condition::Unconscious, "test");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Athletics (STR) should auto-fail
        let resolution = engine.resolve_skill_check(
            &world,
            world.player_character.id,
            Skill::Athletics,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution.narrative.contains("automatically fails"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));

        // Acrobatics (DEX) should also auto-fail
        let resolution = engine.resolve_skill_check(
            &world,
            world.player_character.id,
            Skill::Acrobatics,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));
    }

    #[test]
    fn test_skill_check_unconscious_can_pass_mental() {
        let mut character = create_sample_fighter("Roland");
        character.add_condition(Condition::Unconscious, "test");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Perception (WIS) should NOT auto-fail (though mechanically odd, rules allow it)
        let resolution = engine.resolve_skill_check(
            &world,
            world.player_character.id,
            Skill::Perception,
            1, // Very low DC to ensure success
            Advantage::Normal,
            "test",
        );
        // Should roll normally
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_skill_check_with_proficiency() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Fighter has Athletics proficiency
        let resolution = engine.resolve_skill_check(
            &world,
            world.player_character.id,
            Skill::Athletics,
            15,
            Advantage::Normal,
            "test",
        );

        // Should include dice roll (can't verify proficiency bonus easily, but we know it was used)
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    // ========== Ability Check Tests ==========

    #[test]
    fn test_ability_check_produces_effects() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_ability_check(
            &world,
            world.player_character.id,
            Ability::Strength,
            15,
            Advantage::Normal,
            "lifting a boulder",
        );

        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::CheckSucceeded { .. } | Effect::CheckFailed { .. }
        )));
    }

    #[test]
    fn test_ability_check_unconscious_fails_str_dex() {
        let mut character = create_sample_fighter("Roland");
        character.add_condition(Condition::Unconscious, "test");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // STR check should auto-fail
        let resolution = engine.resolve_ability_check(
            &world,
            world.player_character.id,
            Ability::Strength,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));

        // DEX check should auto-fail
        let resolution = engine.resolve_ability_check(
            &world,
            world.player_character.id,
            Ability::Dexterity,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));
    }

    #[test]
    fn test_ability_check_with_advantage() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Just verify it runs without error with advantage
        let resolution = engine.resolve_ability_check(
            &world,
            world.player_character.id,
            Ability::Intelligence,
            10,
            Advantage::Advantage,
            "test",
        );
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    // ========== Saving Throw Tests ==========

    #[test]
    fn test_saving_throw_produces_effects() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_saving_throw(
            &world,
            world.player_character.id,
            Ability::Constitution,
            15,
            Advantage::Normal,
            "poison",
        );

        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::CheckSucceeded { .. } | Effect::CheckFailed { .. }
        )));
    }

    #[test]
    fn test_saving_throw_with_proficiency() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Fighter has STR and CON save proficiency
        let resolution = engine.resolve_saving_throw(
            &world,
            world.player_character.id,
            Ability::Constitution,
            1, // Very low DC to likely pass
            Advantage::Normal,
            "test",
        );

        assert!(resolution.narrative.contains("saving throw"));
    }

    #[test]
    fn test_saving_throw_unconscious_fails_str_dex() {
        let mut character = create_sample_fighter("Roland");
        character.add_condition(Condition::Unconscious, "test");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // STR save should auto-fail
        let resolution = engine.resolve_saving_throw(
            &world,
            world.player_character.id,
            Ability::Strength,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));

        // DEX save should auto-fail
        let resolution = engine.resolve_saving_throw(
            &world,
            world.player_character.id,
            Ability::Dexterity,
            10,
            Advantage::Normal,
            "test",
        );
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CheckFailed { roll: 0, .. })));
    }

    // ========== Dice Roll Tests ==========

    #[test]
    fn test_roll_dice_valid() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_roll_dice("2d6+5", "damage");

        assert!(resolution.narrative.contains("2d6+5"));
        assert!(resolution.narrative.contains("damage"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_roll_dice_invalid() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_roll_dice("invalid_notation", "test");

        assert!(resolution.narrative.contains("Failed to roll"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_roll_dice_complex_notation() {
        let engine = RulesEngine::new();

        // Keep highest 3 of 4d6
        let resolution = engine.resolve_roll_dice("4d6kh3", "ability score");

        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }
}
