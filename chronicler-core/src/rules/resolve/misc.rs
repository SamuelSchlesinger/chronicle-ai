//! Miscellaneous resolution methods (experience, features, facts, consequences, ability scores).

use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{Ability, CharacterId, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_gain_experience(&self, world: &GameWorld, amount: u32) -> Resolution {
        let new_total = world.player_character.experience + amount;
        let current_level = world.player_character.level;

        // XP thresholds for levels 1-20
        let xp_thresholds = [
            0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000,
            140000, 165000, 195000, 225000, 265000, 305000, 355000,
        ];

        let new_level = xp_thresholds
            .iter()
            .rposition(|&threshold| new_total >= threshold)
            .map(|idx| (idx + 1) as u8)
            .unwrap_or(1);

        let mut resolution = Resolution::new(format!(
            "Gained {amount} experience points (Total: {new_total})"
        ));

        resolution = resolution.with_effect(Effect::ExperienceGained { amount, new_total });

        if new_level > current_level {
            resolution = resolution.with_effect(Effect::LevelUp { new_level });
        }

        resolution
    }

    pub(crate) fn resolve_use_feature(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        feature_name: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if let Some(feature) = character.features.iter().find(|f| f.name == feature_name) {
            if let Some(ref uses) = feature.uses {
                if uses.current > 0 {
                    Resolution::new(format!(
                        "{} uses {} ({} uses remaining)",
                        character.name,
                        feature_name,
                        uses.current - 1
                    ))
                    .with_effect(Effect::FeatureUsed {
                        feature_name: feature_name.to_string(),
                        uses_remaining: uses.current - 1,
                    })
                } else {
                    Resolution::new(format!(
                        "{} has no uses of {} remaining",
                        character.name, feature_name
                    ))
                }
            } else {
                Resolution::new(format!("{} uses {}", character.name, feature_name))
            }
        } else {
            Resolution::new(format!(
                "{} does not have the feature {}",
                character.name, feature_name
            ))
        }
    }

    pub(crate) fn resolve_remember_fact(
        &self,
        subject_name: &str,
        subject_type: &str,
        fact: &str,
        category: &str,
        related_entities: &[String],
        importance: f32,
    ) -> Resolution {
        // The actual storage is handled by the DM agent, not the rules engine.
        // We return a confirmation message and an effect that signals what to store.
        let related_str = if related_entities.is_empty() {
            String::new()
        } else {
            format!(" (related: {})", related_entities.join(", "))
        };

        Resolution::new(format!(
            "Noted: {subject_name} ({subject_type}) - {fact}{related_str}"
        ))
        .with_effect(Effect::FactRemembered {
            subject_name: subject_name.to_string(),
            subject_type: subject_type.to_string(),
            fact: fact.to_string(),
            category: category.to_string(),
            related_entities: related_entities.to_vec(),
            importance,
        })
    }

    pub(crate) fn resolve_change_location(
        &self,
        world: &GameWorld,
        new_location: &str,
        _location_type: Option<String>,
        _description: Option<String>,
    ) -> Resolution {
        let previous_location = world.current_location.name.clone();

        Resolution::new(format!(
            "You travel from {previous_location} to {new_location}."
        ))
        .with_effect(Effect::LocationChanged {
            previous_location,
            new_location: new_location.to_string(),
        })
    }

    pub(crate) fn resolve_register_consequence(
        &self,
        trigger_description: &str,
        consequence_description: &str,
        severity: &str,
        _related_entities: &[String],
        importance: f32,
        expires_in_turns: Option<u32>,
    ) -> Resolution {
        // Generate a unique ID for this consequence
        let consequence_id = uuid::Uuid::new_v4().to_string();

        let severity_display = match severity.to_lowercase().as_str() {
            "minor" => "minor",
            "moderate" => "moderate",
            "major" => "major",
            "critical" => "critical",
            _ => "moderate",
        };

        let expiry_note = match expires_in_turns {
            Some(turns) => format!(" (expires in {turns} turns)"),
            None => String::new(),
        };

        Resolution::new(format!(
            "Consequence registered: If {trigger_description}, then {consequence_description} ({severity_display} severity, importance {importance:.1}){expiry_note}"
        ))
        .with_effect(Effect::ConsequenceRegistered {
            consequence_id,
            trigger_description: trigger_description.to_string(),
            consequence_description: consequence_description.to_string(),
            severity: severity_display.to_string(),
        })
    }

    pub(crate) fn resolve_modify_ability_score(
        &self,
        ability: Ability,
        modifier: i8,
        source: &str,
        duration: Option<&str>,
    ) -> Resolution {
        let modifier_text = if modifier >= 0 {
            format!("+{}", modifier)
        } else {
            format!("{}", modifier)
        };
        let duration_text = duration
            .map(|d| format!(" for {}", d))
            .unwrap_or_else(|| " permanently".to_string());

        Resolution::new(format!(
            "{} modified by {}{} from {}",
            ability.name(),
            modifier_text,
            duration_text,
            source
        ))
        .with_effect(Effect::AbilityScoreModified {
            ability,
            modifier,
            source: source.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::world::{create_sample_fighter, Feature, FeatureUses, GameWorld, RechargeType};

    // ========== Gain Experience Tests ==========

    #[test]
    fn test_gain_experience_no_level_up() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_gain_experience(&world, 100);

        assert!(resolution.narrative.contains("Gained 100"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ExperienceGained { amount: 100, .. })));
        // Should not level up (need 300 XP for level 2)
        assert!(!resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::LevelUp { .. })));
    }

    #[test]
    fn test_gain_experience_level_up() {
        let mut character = create_sample_fighter("Roland");
        // Set fighter to level 1 with 0 XP for this test
        character.level = 1;
        character.experience = 0;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // 300 XP is the threshold for level 2
        let resolution = engine.resolve_gain_experience(&world, 300);

        assert!(resolution.narrative.contains("Gained 300"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::ExperienceGained {
                amount: 300,
                new_total: 300
            }
        )));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::LevelUp { new_level: 2 })));
    }

    #[test]
    fn test_gain_experience_multiple_levels() {
        let mut character = create_sample_fighter("Roland");
        // Set fighter to level 1 with 0 XP for this test
        character.level = 1;
        character.experience = 0;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // 2700 XP should reach level 4
        let resolution = engine.resolve_gain_experience(&world, 2700);

        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::LevelUp { new_level: 4 })));
    }

    // ========== Use Feature Tests ==========

    #[test]
    fn test_use_feature_with_uses() {
        let mut character = create_sample_fighter("Roland");
        character.features.push(Feature {
            name: "Second Wind".to_string(),
            description: "Regain HP".to_string(),
            source: "Fighter".to_string(),
            uses: Some(FeatureUses {
                current: 1,
                maximum: 1,
                recharge: RechargeType::ShortRest,
            }),
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_feature(&world, world.player_character.id, "Second Wind");

        assert!(resolution.narrative.contains("uses Second Wind"));
        assert!(resolution.narrative.contains("0 uses remaining"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::FeatureUsed { feature_name, uses_remaining: 0 } if feature_name == "Second Wind")));
    }

    #[test]
    fn test_use_feature_no_uses_remaining() {
        let mut character = create_sample_fighter("Roland");
        // Use a unique feature name to avoid collision with sample fighter's Second Wind
        character.features.push(Feature {
            name: "Test Feature".to_string(),
            description: "A test feature".to_string(),
            source: "Test".to_string(),
            uses: Some(FeatureUses {
                current: 0,
                maximum: 1,
                recharge: RechargeType::ShortRest,
            }),
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_feature(&world, world.player_character.id, "Test Feature");

        // Narrative says "has no uses of X remaining"
        assert!(resolution.narrative.contains("no uses of"));
        assert!(resolution.narrative.contains("remaining"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_feature_unlimited_uses() {
        let mut character = create_sample_fighter("Roland");
        character.features.push(Feature {
            name: "Fighting Style".to_string(),
            description: "Defense".to_string(),
            source: "Fighter".to_string(),
            uses: None, // No limited uses
        });
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_feature(&world, world.player_character.id, "Fighting Style");

        assert!(resolution.narrative.contains("uses Fighting Style"));
        // No effect since it's unlimited
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_use_feature_not_found() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_use_feature(&world, world.player_character.id, "Nonexistent Feature");

        assert!(resolution.narrative.contains("does not have"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Remember Fact Tests ==========

    #[test]
    fn test_remember_fact_basic() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_remember_fact(
            "Goblin Chief",
            "NPC",
            "Is afraid of fire",
            "weakness",
            &[],
            0.8,
        );

        assert!(resolution.narrative.contains("Goblin Chief"));
        assert!(resolution.narrative.contains("afraid of fire"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::FactRemembered { subject_name, fact, .. }
                if subject_name == "Goblin Chief" && fact == "Is afraid of fire")
        ));
    }

    #[test]
    fn test_remember_fact_with_related_entities() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_remember_fact(
            "Dark Tower",
            "Location",
            "Contains the artifact",
            "lore",
            &["Artifact".to_string(), "Evil Wizard".to_string()],
            0.9,
        );

        assert!(resolution.narrative.contains("related:"));
        assert!(resolution.narrative.contains("Artifact"));
        assert!(resolution.narrative.contains("Evil Wizard"));
    }

    // ========== Change Location Tests ==========

    #[test]
    fn test_change_location() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_change_location(&world, "Dark Forest", Some("forest".to_string()), None);

        assert!(resolution.narrative.contains("travel from"));
        assert!(resolution.narrative.contains("Dark Forest"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::LocationChanged { new_location, .. } if new_location == "Dark Forest")
        ));
    }

    // ========== Register Consequence Tests ==========

    #[test]
    fn test_register_consequence_basic() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_register_consequence(
            "player enters the throne room",
            "guards attack",
            "major",
            &[],
            0.9,
            None,
        );

        assert!(resolution.narrative.contains("player enters"));
        assert!(resolution.narrative.contains("guards attack"));
        assert!(resolution.narrative.contains("major severity"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::ConsequenceRegistered { severity, .. } if severity == "major")
        ));
    }

    #[test]
    fn test_register_consequence_with_expiry() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_register_consequence(
            "alarm triggered",
            "reinforcements arrive",
            "moderate",
            &[],
            0.7,
            Some(5),
        );

        assert!(resolution.narrative.contains("expires in 5 turns"));
    }

    // ========== Modify Ability Score Tests ==========

    #[test]
    fn test_modify_ability_score_positive() {
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_modify_ability_score(Ability::Strength, 2, "Bull's Strength", None);

        assert!(resolution.narrative.contains("Strength"));
        assert!(resolution.narrative.contains("+2"));
        assert!(resolution.narrative.contains("permanently"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::AbilityScoreModified { ability, modifier: 2, .. } if *ability == Ability::Strength)
        ));
    }

    #[test]
    fn test_modify_ability_score_negative() {
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_modify_ability_score(Ability::Dexterity, -2, "Curse", Some("1 hour"));

        assert!(resolution.narrative.contains("Dexterity"));
        assert!(resolution.narrative.contains("-2"));
        assert!(resolution.narrative.contains("for 1 hour"));
    }
}
