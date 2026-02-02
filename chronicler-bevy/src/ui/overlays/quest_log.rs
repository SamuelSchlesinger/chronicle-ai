//! Quest log overlay.

use bevy_egui::egui;
use chronicler_core::world::QuestStatus;

use crate::state::AppState;

/// Render the quest log overlay.
pub fn render_quest_log(ctx: &egui::Context, app_state: &AppState) {
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).clamp(280.0, 450.0);
    let height = (screen.height() * 0.7).clamp(300.0, 450.0);

    egui::Window::new("Quest Log")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 600.0])
        .show(ctx, |ui| {
            if app_state.world.quests.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No quests yet.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Your adventure awaits...");
                });
            } else {
                // Active quests
                let active_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Active)
                    .collect();

                if !active_quests.is_empty() {
                    ui.heading(egui::RichText::new("Active Quests").color(egui::Color32::YELLOW));
                    ui.separator();

                    for quest in active_quests {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(&quest.name).strong());
                            ui.label(&quest.description);

                            // Objectives
                            if !quest.objectives.is_empty() {
                                ui.add_space(4.0);
                                for obj in &quest.objectives {
                                    let marker = if obj.completed { "[X]" } else { "[ ]" };
                                    let color = if obj.completed {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::WHITE
                                    };
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} {}",
                                            marker, obj.description
                                        ))
                                        .color(color),
                                    );
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                }

                // Completed quests
                let completed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Completed)
                    .collect();

                if !completed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(egui::RichText::new("Completed Quests").color(egui::Color32::GREEN));
                    ui.separator();

                    for quest in completed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Done] {}", quest.name))
                                .color(egui::Color32::from_rgb(100, 180, 100)),
                        );
                    }
                }

                // Failed quests
                let failed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| {
                        q.status == QuestStatus::Failed || q.status == QuestStatus::Abandoned
                    })
                    .collect();

                if !failed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(egui::RichText::new("Failed Quests").color(egui::Color32::RED));
                    ui.separator();

                    for quest in failed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Failed] {}", quest.name))
                                .color(egui::Color32::from_rgb(180, 100, 100)),
                        );
                    }
                }
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Shift+Q or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}
