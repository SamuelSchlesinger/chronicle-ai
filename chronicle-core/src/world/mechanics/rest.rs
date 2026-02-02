//! Rest mechanics for D&D 5e.
//!
//! Implements short rest and long rest recovery rules including:
//! - Hit point recovery
//! - Spell slot recovery
//! - Hit dice recovery
//! - Feature use recovery
//! - Class-specific resource recovery
//! - Condition removal (exhaustion, unconscious)

use crate::world::{Character, CharacterClass, Condition, RechargeType};

/// Apply short rest recovery to a character.
///
/// Short rest (1 hour):
/// - Warlocks recover all spell slots (Pact Magic)
/// - Features that recharge on short rest are restored
/// - Class-specific resources that recharge on short rest are restored
pub fn apply_short_rest(character: &mut Character) {
    // Warlocks recover all spell slots on short rest (Pact Magic)
    let is_warlock = character
        .classes
        .iter()
        .any(|c| c.class == CharacterClass::Warlock);
    if is_warlock {
        if let Some(ref mut spellcasting) = character.spellcasting {
            spellcasting.spell_slots.recover_all();
        }
    }

    // Reset feature uses that recharge on short rest
    for feature in &mut character.features {
        if let Some(ref mut uses) = feature.uses {
            if matches!(uses.recharge, RechargeType::ShortRest) {
                uses.current = uses.maximum;
            }
        }
    }

    // Reset class-specific resources
    for class_level in &character.classes {
        character
            .class_resources
            .short_rest_recovery(class_level.class, class_level.level);
    }
}

/// Apply long rest recovery to a character.
///
/// Long rest (8 hours):
/// - Full HP recovery
/// - Remove Unconscious condition
/// - Reduce exhaustion by 1 level
/// - Recover half of total hit dice
/// - Recover all spell slots
/// - Features that recharge on short or long rest are restored
/// - Class-specific resources that recharge on long rest are restored
pub fn apply_long_rest(character: &mut Character) {
    // Full HP recovery
    let max_hp = character.hit_points.maximum;
    character.hit_points.current = max_hp;

    // Remove Unconscious condition if present (they're now healed)
    character
        .conditions
        .retain(|c| c.condition != Condition::Unconscious);

    // Reduce exhaustion by 1 level (if any)
    for condition in &mut character.conditions {
        if let Condition::Exhaustion(level) = &mut condition.condition {
            if *level > 0 {
                *level -= 1;
            }
        }
    }
    // Remove exhaustion if reduced to 0
    character
        .conditions
        .retain(|c| !matches!(c.condition, Condition::Exhaustion(0)));

    // Recover half hit dice
    character.hit_dice.recover_half();

    // Recover spell slots
    if let Some(ref mut spellcasting) = character.spellcasting {
        spellcasting.spell_slots.recover_all();
    }

    // Reset feature uses (both short rest and long rest features)
    for feature in &mut character.features {
        if let Some(ref mut uses) = feature.uses {
            if matches!(
                uses.recharge,
                RechargeType::LongRest | RechargeType::ShortRest
            ) {
                uses.current = uses.maximum;
            }
        }
    }

    // Reset class-specific resources
    let classes: Vec<_> = character
        .classes
        .iter()
        .map(|c| (c.class, c.level))
        .collect();
    for (class, level) in classes {
        character.class_resources.long_rest_recovery(class, level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::DieType;
    use crate::world::Ability;
    use crate::world::{
        AbilityScores, ArmorClass, ArmorType, ClassLevel, Feature, FeatureUses, HitPoints,
        ProficiencyLevel, Skill,
    };

    fn create_test_fighter() -> Character {
        let mut character = Character::new("Test Fighter");

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
                current: 0, // Start with 0 to test recovery
                maximum: 1,
                recharge: RechargeType::ShortRest,
            }),
        });

        character.features.push(Feature {
            name: "Action Surge".to_string(),
            description: "Take one additional action on your turn".to_string(),
            source: "Fighter".to_string(),
            uses: Some(FeatureUses {
                current: 0, // Start with 0 to test recovery
                maximum: 1,
                recharge: RechargeType::ShortRest,
            }),
        });

        character
    }

    #[test]
    fn test_short_rest_recovers_short_rest_features() {
        let mut character = create_test_fighter();

        // Verify features start at 0
        assert_eq!(character.features[0].uses.as_ref().unwrap().current, 0);
        assert_eq!(character.features[1].uses.as_ref().unwrap().current, 0);

        apply_short_rest(&mut character);

        // Features should be recovered
        assert_eq!(character.features[0].uses.as_ref().unwrap().current, 1);
        assert_eq!(character.features[1].uses.as_ref().unwrap().current, 1);
    }

    #[test]
    fn test_long_rest_recovers_hp() {
        let mut character = create_test_fighter();
        character.hit_points.current = 10; // Damage the character

        apply_long_rest(&mut character);

        assert_eq!(character.hit_points.current, character.hit_points.maximum);
    }

    #[test]
    fn test_long_rest_recovers_features() {
        let mut character = create_test_fighter();

        // Add a long rest feature
        character.features.push(Feature {
            name: "Long Rest Feature".to_string(),
            description: "Test feature".to_string(),
            source: "Test".to_string(),
            uses: Some(FeatureUses {
                current: 0,
                maximum: 2,
                recharge: RechargeType::LongRest,
            }),
        });

        apply_long_rest(&mut character);

        // All features should be recovered
        assert_eq!(character.features[0].uses.as_ref().unwrap().current, 1);
        assert_eq!(character.features[1].uses.as_ref().unwrap().current, 1);
        assert_eq!(character.features[2].uses.as_ref().unwrap().current, 2);
    }
}
