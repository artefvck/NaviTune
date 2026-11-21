use crate::advance_os::*;
use crate::card_layout::{draw_card, two_col_cards};
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn draw_advance_linux(&mut self, ui: &mut egui::Ui) {
        let (dpm_gfx, gfxoff, baco) = (self.smu_features[1], self.smu_features[20], self.smu_features[21]);
        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();

        two_col_cards(ui, "advance_linux_cols", |ui| {
            draw_card(ui, "Linux sysfs Export", |ui| {
                ui.label("Export pp_table binary or copy sysfs command for /sys/class/drm/card0/device/pp_table:");
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Install").clicked() {
                        self.sync_fields_to_buffer();
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match install_linux_ppt(&self.raw_buffer, &sel_lbl) {
                            Ok(msg) => {
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                            }
                            Err(err) => self.show_error("Linux Install Error", &err),
                        }
                    }
                    if ui.button("Uninstall").clicked() {
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match uninstall_linux_ppt(&sel_lbl) {
                            Ok(msg) => {
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                            }
                            Err(err) => self.show_error("Linux Uninstall Error", &err),
                        }
                    }
                    if ui.button("Export pp_table (.bin)").clicked() {
                        self.save_ppt_file_with_default("pp_table.bin");
                    }
                    if ui.button("Copy sysfs Apply Command").clicked() {
                        ui.ctx().copy_text("sudo cp pp_table.bin /sys/class/drm/card0/device/pp_table".to_string());
                        self.status_msg = "Copied Linux sysfs pp_table apply command to clipboard".into();
                    }
                });
            });
        }, |ui| {
            draw_card(ui, "Linux Status", |ui| {
                egui::Grid::new("advance_linux_grid").num_columns(3).min_col_width(((ui.available_width() - 32.0) / 3.0).max(110.0)).striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Check"); ui.strong("State"); ui.strong("Linux Status"); ui.end_row();
                    ui.label("DPM_GFXCLK");
                    ui.monospace(if dpm_gfx { "Enabled (1)" } else { "Disabled (0)" });
                    ui.colored_label(
                        if dpm_gfx { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) },
                        if dpm_gfx { "OK" } else { "DISABLED" },
                    );
                    ui.end_row();
                    ui.label("GFXOFF / BACO");
                    ui.monospace(format!("{} / {}", gfxoff as u8, baco as u8));
                    ui.colored_label(egui::Color32::from_rgb(130, 180, 255), if gfxoff { "Power Saving" } else { "Compute" });
                    ui.end_row();
                    ui.label("sysfs Target");
                    ui.monospace("card0/device/pp_table");
                    ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "READY");
                    ui.end_row();
                    let (sync_detail, sync_badge, sync_color) = check_linux_ppt_sync(&sel_lbl);
                    ui.label("Config Sync");
                    ui.monospace(sync_detail);
                    ui.colored_label(sync_color, sync_badge);
                    ui.end_row();
                });
            });
        });
    }
}
