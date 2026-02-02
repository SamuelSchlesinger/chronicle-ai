//! Load game overlay.

use bevy_egui::egui;

use crate::state::{ActiveOverlay, AppState, GameSaveList};

/// Render the load game overlay. Returns the path to load if a game is selected.
pub fn render_load_game(
    ctx: &egui::Context,
    app_state: &mut AppState,
    save_list: &mut GameSaveList,
) -> Option<String> {
    let mut selected_path = None;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).clamp(300.0, 480.0);
    let height = (screen.height() * 0.65).clamp(280.0, 400.0);

    egui::Window::new("Load Game")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([600.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Saved Games");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        save_list.saves.clear();
                        save_list.loaded = false;
                        save_list.loading = false;
                        save_list.error = None;
                        save_list.selected = None;
                    }
                });
            });
            ui.separator();

            if save_list.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading saved games...");
                });
            } else if let Some(ref err) = save_list.error {
                ui.colored_label(egui::Color32::RED, err);
            } else if save_list.saves.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No saved games found.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Start a New Game and save your progress!");
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (i, save) in save_list.saves.iter().enumerate() {
                            let is_selected = save_list.selected == Some(i);

                            let text = format!(
                                "{} - {} (Level {})\nSaved: {}",
                                save.campaign_name,
                                save.character_name,
                                save.character_level,
                                save.saved_at
                            );

                            if ui.selectable_label(is_selected, text).clicked() {
                                save_list.selected = Some(i);
                            }
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    let can_load = save_list.selected.is_some();

                    if ui
                        .add_enabled(can_load, egui::Button::new("Load Game"))
                        .clicked()
                    {
                        app_state.play_click();
                        if let Some(idx) = save_list.selected {
                            selected_path = Some(save_list.saves[idx].path.clone());
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        app_state.play_click();
                        app_state.overlay = ActiveOverlay::None;
                    }
                });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });

    selected_path
}
