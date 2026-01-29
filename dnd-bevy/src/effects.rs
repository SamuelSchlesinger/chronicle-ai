//! Effect-to-animation mapping.
//!
//! This module translates game Effects into visual animations
//! and UI state updates.

use bevy::prelude::*;
use dnd_core::rules::Effect;
use dnd_core::world::NarrativeType;

use crate::animations::{
    self,
    dice::{parse_dice_type, DiceType},
    effects::EffectType,
};
use crate::state::AppState;

/// Process a game effect and trigger appropriate animations.
pub fn process_effect(
    app_state: &mut AppState,
    effect: &Effect,
    commands: &mut Commands,
    time: f64,
) {
    match effect {
        Effect::DiceRolled { roll, purpose } => {
            // Spawn dice animation
            let dice_type = parse_dice_type(&roll.expression.original);
            animations::spawn_dice_animation(
                commands,
                roll.total,
                dice_type,
                purpose.clone(),
                Vec2::new(400.0, 300.0), // Center-ish position
            );

            // Add to narrative
            let result_text = format!("{}: {} = {}", purpose, roll.expression, roll.total);
            app_state.add_narrative(result_text, NarrativeType::System, time);
        }

        Effect::AttackHit {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
            is_critical,
        } => {
            // Screen shake on hit
            let intensity = if *is_critical { 1.0 } else { 0.5 };
            let effect_type = if *is_critical {
                EffectType::CriticalHit
            } else {
                EffectType::ScreenShake
            };
            animations::spawn_combat_effect(commands, effect_type, Vec2::ZERO, intensity);

            // Narrative
            if *is_critical {
                app_state.add_narrative(
                    format!(
                        "CRITICAL HIT! {} rolls {} vs AC {} and strikes {}!",
                        attacker_name, attack_roll, target_ac, target_name
                    ),
                    NarrativeType::Combat,
                    time,
                );
            } else {
                app_state.add_narrative(
                    format!(
                        "{} rolls {} vs AC {} and hits {}!",
                        attacker_name, attack_roll, target_ac, target_name
                    ),
                    NarrativeType::Combat,
                    time,
                );
            }
        }

        Effect::AttackMissed {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
        } => {
            animations::spawn_combat_effect(commands, EffectType::Miss, Vec2::ZERO, 0.3);
            app_state.add_narrative(
                format!(
                    "{} rolls {} vs AC {} and misses {}!",
                    attacker_name, attack_roll, target_ac, target_name
                ),
                NarrativeType::Combat,
                time,
            );
        }

        Effect::HpChanged {
            amount,
            new_current,
            dropped_to_zero,
            ..
        } => {
            // Spawn floating damage/healing number
            let is_healing = *amount > 0;
            let is_critical = amount.abs() >= 20; // Arbitrary threshold for "big" damage

            animations::spawn_damage_number(
                commands,
                *amount,
                is_healing,
                is_critical,
                Vec2::new(500.0, 400.0),
            );

            // Flash effect
            let effect_type = if is_healing {
                EffectType::Heal
            } else {
                EffectType::DamageFlash
            };
            animations::spawn_combat_effect(commands, effect_type, Vec2::ZERO, 0.5);

            // Narrative
            if *amount < 0 {
                app_state.add_narrative(
                    format!("Takes {} damage! (HP: {})", -amount, new_current),
                    NarrativeType::Combat,
                    time,
                );
            } else if *amount > 0 {
                app_state.add_narrative(
                    format!("Heals {} HP! (HP: {})", amount, new_current),
                    NarrativeType::System,
                    time,
                );
            }

            if *dropped_to_zero {
                animations::spawn_combat_effect(commands, EffectType::Death, Vec2::ZERO, 1.0);
                app_state.set_status("You fall unconscious!", time);
            }
        }

        Effect::ConditionApplied {
            condition, source, ..
        } => {
            app_state.add_narrative(
                format!("Now {} from {}!", condition, source),
                NarrativeType::Combat,
                time,
            );
        }

        Effect::ConditionRemoved { condition, .. } => {
            app_state.add_narrative(
                format!("No longer {}.", condition),
                NarrativeType::System,
                time,
            );
        }

        Effect::CombatStarted => {
            animations::spawn_combat_effect(commands, EffectType::ScreenShake, Vec2::ZERO, 0.3);
            app_state.add_narrative("Combat begins!".to_string(), NarrativeType::Combat, time);
            app_state.set_status("Roll for initiative!", time);
        }

        Effect::CombatEnded => {
            app_state.add_narrative("Combat ends.".to_string(), NarrativeType::System, time);
        }

        Effect::TurnAdvanced {
            round,
            current_combatant,
        } => {
            app_state.add_narrative(
                format!("Round {} - {}'s turn.", round, current_combatant),
                NarrativeType::Combat,
                time,
            );
        }

        Effect::InitiativeRolled {
            name, roll, total, ..
        } => {
            // Small dice animation for initiative
            animations::spawn_dice_animation(
                commands,
                *total,
                DiceType::D20,
                format!("{}'s initiative", name),
                Vec2::new(300.0, 400.0),
            );
            app_state.add_narrative(
                format!("{} rolls {} for initiative (total: {})", name, roll, total),
                NarrativeType::System,
                time,
            );
        }

        Effect::CombatantAdded {
            name, initiative, ..
        } => {
            app_state.add_narrative(
                format!("{} enters combat with initiative {}.", name, initiative),
                NarrativeType::Combat,
                time,
            );
        }

        Effect::TimeAdvanced { minutes } => {
            if *minutes >= 60 {
                let hours = minutes / 60;
                let mins = minutes % 60;
                if mins > 0 {
                    app_state.add_narrative(
                        format!("{} hours and {} minutes pass.", hours, mins),
                        NarrativeType::System,
                        time,
                    );
                } else {
                    app_state.add_narrative(
                        format!("{} hours pass.", hours),
                        NarrativeType::System,
                        time,
                    );
                }
            } else {
                app_state.add_narrative(
                    format!("{} minutes pass.", minutes),
                    NarrativeType::System,
                    time,
                );
            }
        }

        Effect::ExperienceGained { amount, new_total } => {
            app_state.add_narrative(
                format!("Gained {} XP! (Total: {} XP)", amount, new_total),
                NarrativeType::System,
                time,
            );
        }

        Effect::LevelUp { new_level } => {
            animations::spawn_combat_effect(commands, EffectType::LevelUp, Vec2::ZERO, 1.0);
            app_state.add_narrative(
                format!("LEVEL UP! You are now level {}!", new_level),
                NarrativeType::System,
                time,
            );
            app_state.set_status(format!("Level up! Now level {}!", new_level), time);
        }

        Effect::FeatureUsed {
            feature_name,
            uses_remaining,
        } => {
            app_state.add_narrative(
                format!("Used {}. ({} uses remaining)", feature_name, uses_remaining),
                NarrativeType::System,
                time,
            );
        }

        Effect::SpellSlotUsed { level, remaining } => {
            animations::spawn_combat_effect(commands, EffectType::SpellCast, Vec2::ZERO, 0.5);
            app_state.add_narrative(
                format!("Used a level {} spell slot. ({} remaining)", level, remaining),
                NarrativeType::System,
                time,
            );
        }

        Effect::RestCompleted { rest_type } => {
            let rest_name = match rest_type {
                dnd_core::rules::RestType::Short => "short",
                dnd_core::rules::RestType::Long => "long",
            };
            animations::spawn_combat_effect(commands, EffectType::Heal, Vec2::ZERO, 0.5);
            app_state.add_narrative(
                format!("Completed a {} rest.", rest_name),
                NarrativeType::System,
                time,
            );
        }

        Effect::CheckSucceeded {
            check_type,
            roll,
            dc,
        } => {
            app_state.add_narrative(
                format!("{} check succeeded! ({} vs DC {})", check_type, roll, dc),
                NarrativeType::System,
                time,
            );
        }

        Effect::CheckFailed {
            check_type,
            roll,
            dc,
        } => {
            app_state.add_narrative(
                format!("{} check failed. ({} vs DC {})", check_type, roll, dc),
                NarrativeType::System,
                time,
            );
        }

        Effect::FactRemembered { .. } => {
            // Internal - no UI effect
        }

        Effect::ItemAdded {
            item_name,
            quantity,
            new_total,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{} x ", quantity)
            } else {
                String::new()
            };
            app_state.add_narrative(
                format!("Received {}{}! (now have {})", qty_str, item_name, new_total),
                NarrativeType::System,
                time,
            );
        }

        Effect::ItemRemoved {
            item_name,
            quantity,
            remaining,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{} x ", quantity)
            } else {
                String::new()
            };
            if *remaining > 0 {
                app_state.add_narrative(
                    format!("Lost {}{}. ({} remaining)", qty_str, item_name, remaining),
                    NarrativeType::System,
                    time,
                );
            } else {
                app_state.add_narrative(
                    format!("Lost {}{}.", qty_str, item_name),
                    NarrativeType::System,
                    time,
                );
            }
        }

        Effect::ItemEquipped { item_name, slot } => {
            app_state.add_narrative(
                format!("Equipped {} in {} slot.", item_name, slot),
                NarrativeType::System,
                time,
            );
        }

        Effect::ItemUnequipped { item_name, slot } => {
            app_state.add_narrative(
                format!("Unequipped {} from {} slot.", item_name, slot),
                NarrativeType::System,
                time,
            );
        }

        Effect::ItemUsed { item_name, result } => {
            app_state.add_narrative(
                format!("Used {}. {}", item_name, result),
                NarrativeType::System,
                time,
            );
        }

        Effect::GoldChanged {
            amount,
            new_total,
            reason,
        } => {
            let action = if *amount >= 0.0 { "Gained" } else { "Spent" };
            app_state.add_narrative(
                format!(
                    "{} {:.0} gp ({}). Total: {:.0} gp",
                    action,
                    amount.abs(),
                    reason,
                    new_total
                ),
                NarrativeType::System,
                time,
            );
        }

        Effect::AcChanged { new_ac, source } => {
            app_state.add_narrative(
                format!("AC changed to {} ({})", new_ac, source),
                NarrativeType::System,
                time,
            );
        }

        Effect::DeathSaveFailure {
            total_failures,
            source,
            ..
        } => {
            animations::spawn_combat_effect(commands, EffectType::DamageFlash, Vec2::ZERO, 0.8);
            app_state.add_narrative(
                format!(
                    "DEATH SAVE FAILURE from {}! ({}/3 failures)",
                    source, total_failures
                ),
                NarrativeType::Combat,
                time,
            );
            if *total_failures >= 3 {
                animations::spawn_combat_effect(commands, EffectType::Death, Vec2::ZERO, 1.0);
                app_state.set_status("You have died!", time);
            } else {
                app_state.set_status(format!("Death saves: {}/3 failures", total_failures), time);
            }
        }

        Effect::DeathSavesReset { .. } => {
            animations::spawn_combat_effect(commands, EffectType::Heal, Vec2::ZERO, 0.5);
            app_state.add_narrative(
                "Death saves reset - you're stable!".to_string(),
                NarrativeType::System,
                time,
            );
        }

        Effect::CharacterDied { cause, .. } => {
            animations::spawn_combat_effect(commands, EffectType::Death, Vec2::ZERO, 1.0);
            app_state.add_narrative(
                format!("YOU HAVE DIED! Cause: {}", cause),
                NarrativeType::Combat,
                time,
            );
            app_state.set_status("GAME OVER - Your character has died.", time);
        }

        Effect::DeathSaveSuccess {
            roll,
            total_successes,
            ..
        } => {
            animations::spawn_dice_animation(
                commands,
                *roll,
                DiceType::D20,
                "Death Save".to_string(),
                Vec2::new(400.0, 300.0),
            );
            app_state.add_narrative(
                format!(
                    "Death save SUCCESS! Rolled {} ({}/3 successes)",
                    roll, total_successes
                ),
                NarrativeType::Combat,
                time,
            );
            app_state.set_status(format!("Death saves: {}/3 successes", total_successes), time);
        }

        Effect::Stabilized { .. } => {
            animations::spawn_combat_effect(commands, EffectType::Heal, Vec2::ZERO, 0.7);
            app_state.add_narrative(
                "You have stabilized! No longer dying.".to_string(),
                NarrativeType::Combat,
                time,
            );
            app_state.set_status("Stabilized - unconscious but stable", time);
        }

        Effect::ConcentrationBroken {
            spell_name,
            damage_taken,
            roll,
            dc,
            ..
        } => {
            animations::spawn_combat_effect(commands, EffectType::DamageFlash, Vec2::ZERO, 0.6);
            app_state.add_narrative(
                format!(
                    "CONCENTRATION BROKEN! Took {} damage, rolled {} vs DC {} - {} ends!",
                    damage_taken, roll, dc, spell_name
                ),
                NarrativeType::Combat,
                time,
            );
            app_state.set_status(format!("Lost concentration on {}!", spell_name), time);
        }

        Effect::ConcentrationMaintained {
            spell_name,
            roll,
            dc,
            ..
        } => {
            app_state.add_narrative(
                format!(
                    "Concentration maintained! Rolled {} vs DC {} - {} continues.",
                    roll, dc, spell_name
                ),
                NarrativeType::System,
                time,
            );
        }

        Effect::LocationChanged {
            previous_location,
            new_location,
        } => {
            app_state.add_narrative(
                format!("You travel from {} to {}.", previous_location, new_location),
                NarrativeType::System,
                time,
            );
            app_state.set_status(format!("Now at: {}", new_location), time);
        }
    }
}
