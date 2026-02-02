//! Inventory overlay.

use bevy_egui::egui;

use crate::state::AppState;

/// Render the inventory overlay.
pub fn render_inventory(ctx: &egui::Context, app_state: &AppState) {
    // Use responsive sizing based on available screen
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).clamp(280.0, 400.0);
    let height = (screen.height() * 0.7).clamp(300.0, 450.0);

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([500.0, 600.0])
        .show(ctx, |ui| {
            // Currency
            ui.horizontal(|ui| {
                ui.label("Currency:");
                ui.label(
                    egui::RichText::new(format!("{} gp", app_state.world.gold))
                        .color(egui::Color32::from_rgb(218, 165, 32))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{} sp", app_state.world.silver))
                        .color(egui::Color32::from_rgb(192, 192, 192))
                        .strong(),
                );
            });

            ui.separator();

            // Equipped items
            ui.heading("Equipped");
            ui.indent("equipped", |ui| {
                if let Some(ref weapon) = app_state.world.equipped_weapon {
                    ui.label(format!("Main Hand: {weapon}"));
                } else {
                    ui.label("Main Hand: (empty)");
                }
                if let Some(ref armor) = app_state.world.equipped_armor {
                    ui.label(format!("Armor: {armor}"));
                } else {
                    ui.label("Armor: (none)");
                }
            });

            ui.separator();

            // Inventory items
            ui.heading("Items");

            if app_state.world.inventory_items.is_empty() {
                ui.label(egui::RichText::new("Your pack is empty.").italics());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for item in &app_state.world.inventory_items {
                            ui.horizontal(|ui| {
                                let name = if item.quantity > 1 {
                                    format!("{} x{}", item.name, item.quantity)
                                } else {
                                    item.name.clone()
                                };

                                let color = if item.magical {
                                    egui::Color32::from_rgb(138, 43, 226) // Purple for magical
                                } else {
                                    egui::Color32::WHITE
                                };

                                ui.label(egui::RichText::new(name).color(color));

                                if item.weight > 0.0 {
                                    ui.label(
                                        egui::RichText::new(format!("({:.1} lb)", item.weight))
                                            .color(egui::Color32::GRAY)
                                            .small(),
                                    );
                                }
                            });

                            if let Some(ref desc) = item.description {
                                ui.indent("item_desc", |ui| {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .color(egui::Color32::GRAY)
                                            .small()
                                            .italics(),
                                    );
                                });
                            }
                        }
                    });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press I or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}
