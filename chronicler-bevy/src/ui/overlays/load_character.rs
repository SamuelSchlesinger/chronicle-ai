//! Load character overlay.

use bevy_egui::egui;

use crate::state::{ActiveOverlay, AppState, CharacterSaveList};

/// Render the load character overlay.
pub fn render_load_character(
    ctx: &egui::Context,
    app_state: &mut AppState,
    save_list: &mut CharacterSaveList,
) -> Option<chronicler_core::world::Character> {
    let mut selected_character = None;

    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).clamp(300.0, 450.0);
    let height = (screen.height() * 0.65).clamp(280.0, 400.0);

    egui::Window::new("Load Character")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Saved Characters");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        app_state.play_click();
                        // Reset to trigger a reload
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
                    ui.label("Loading saved characters...");
                });
            } else if let Some(ref err) = save_list.error {
                ui.colored_label(egui::Color32::RED, err);
            } else if save_list.saves.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No saved characters found.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Create a character first!");
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (i, save) in save_list.saves.iter().enumerate() {
                            let is_selected = save_list.selected == Some(i);
                            let meta = &save.metadata;

                            let text = format!(
                                "{} - Level {} {} {}{}",
                                meta.name,
                                meta.level,
                                meta.race,
                                meta.class,
                                if meta.has_backstory {
                                    " (has backstory)"
                                } else {
                                    ""
                                }
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
                        .add_enabled(can_load, egui::Button::new("Load & Play"))
                        .clicked()
                    {
                        app_state.play_click();
                        if let Some(idx) = save_list.selected {
                            let path = save_list.saves[idx].path.clone();
                            // Load the character using the shared runtime
                            match crate::runtime::RUNTIME
                                .block_on(chronicler_core::SavedCharacter::load_json(&path))
                            {
                                Ok(saved) => {
                                    selected_character = Some(saved.character);
                                    app_state.overlay = ActiveOverlay::None;
                                }
                                Err(e) => {
                                    save_list.error = Some(format!("Failed to load: {e}"));
                                }
                            }
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

    selected_character
}
