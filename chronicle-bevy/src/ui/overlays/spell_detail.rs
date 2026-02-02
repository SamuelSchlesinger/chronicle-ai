//! Spell detail overlay.

use bevy_egui::egui;
use chronicle_core::spells::get_spell;

use crate::state::AppState;

/// Render the spell detail popup.
pub fn render_spell_detail(ctx: &egui::Context, app_state: &mut AppState) {
    let spell_name = match &app_state.viewing_spell {
        Some(name) => name.clone(),
        None => return,
    };

    let spell_data = get_spell(&spell_name);

    egui::Window::new("Spell Details")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([400.0, 350.0])
        .max_size([500.0, 500.0])
        .show(ctx, |ui| {
            match spell_data {
                Some(spell) => {
                    // Spell name and level
                    ui.heading(
                        egui::RichText::new(&spell.name)
                            .color(egui::Color32::from_rgb(100, 180, 255)),
                    );

                    let level_text = if spell.level == 0 {
                        format!("{} cantrip", spell.school.name())
                    } else {
                        let suffix = match spell.level {
                            1 => "st",
                            2 => "nd",
                            3 => "rd",
                            _ => "th",
                        };
                        format!("{}{}-level {}", spell.level, suffix, spell.school.name())
                    };

                    ui.label(
                        egui::RichText::new(level_text)
                            .italics()
                            .color(egui::Color32::LIGHT_GRAY),
                    );

                    if spell.ritual {
                        ui.label(
                            egui::RichText::new("(ritual)")
                                .italics()
                                .color(egui::Color32::from_rgb(180, 140, 255)),
                        );
                    }

                    ui.separator();

                    // Spell properties in a grid
                    egui::Grid::new("spell_properties")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Casting Time:").strong());
                            ui.label(spell.casting_time.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Range:").strong());
                            ui.label(spell.range.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Components:").strong());
                            ui.label(spell.components.description());
                            ui.end_row();

                            ui.label(egui::RichText::new("Duration:").strong());
                            let duration_text = if spell.concentration {
                                format!("Concentration, {}", spell.duration.description())
                            } else {
                                spell.duration.description().to_string()
                            };
                            ui.label(duration_text);
                            ui.end_row();
                        });

                    ui.separator();

                    // Description
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            ui.label(&spell.description);
                        });

                    // Combat info if applicable
                    if spell.damage_dice.is_some()
                        || spell.healing_dice.is_some()
                        || spell.save_type.is_some()
                    {
                        ui.separator();
                        ui.label(egui::RichText::new("Combat").strong());

                        if let Some(ref dice) = spell.damage_dice {
                            let damage_text = if let Some(ref dtype) = spell.damage_type {
                                format!("Damage: {} {}", dice, dtype.name())
                            } else {
                                format!("Damage: {}", dice)
                            };
                            ui.label(damage_text);
                        }

                        if let Some(ref dice) = spell.healing_dice {
                            ui.label(format!("Healing: {}", dice));
                        }

                        if let Some(ref save) = spell.save_type {
                            let save_text = if let Some(ref effect) = spell.save_effect {
                                format!("{} save: {}", save.abbreviation(), effect)
                            } else {
                                format!("{} save", save.abbreviation())
                            };
                            ui.label(save_text);
                        }

                        if let Some(ref attack) = spell.attack_type {
                            let attack_name = match attack {
                                chronicle_core::spells::SpellAttackType::Melee => "Melee",
                                chronicle_core::spells::SpellAttackType::Ranged => "Ranged",
                            };
                            ui.label(format!("Attack: {} spell attack", attack_name));
                        }
                    }
                }
                None => {
                    ui.heading(&spell_name);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Spell details not found in database.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        "This spell may be from a source not yet added to the SRD 5.2 database.",
                    );
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app_state.play_click();
                    app_state.viewing_spell = None;
                }
                ui.label(
                    egui::RichText::new("(or press Escape)")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
        });

    // Close on Escape key
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app_state.viewing_spell = None;
    }
}
