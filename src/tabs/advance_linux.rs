use crate::advance_os::*;
use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn draw_advance_linux(&mut self, ui: &mut egui::Ui) {
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let is_gcn = arch.is_gcn_subtable();
        let plat_caps = if is_gcn && self.raw_buffer.len() >= 0x17 {
            u32::from_le_bytes([self.raw_buffer[0x13], self.raw_buffer[0x14], self.raw_buffer[0x15], self.raw_buffer[0x16]])
        } else {
            0
        };
        let (dpm_gfx, gfxoff, baco) = if is_gcn {
            ((plat_caps & 1) != 0, false, (plat_caps & (1 << 21)) != 0)
        } else {
            (self.smu_features[1], self.smu_features[17], self.smu_features[18])
        };
        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();

                    draw_card(ui, "Linux Status", |ui| {
                egui::Grid::new("advance_linux_grid").num_columns(3).min_col_width(((ui.available_width() - 32.0) / 3.0).max(110.0)).striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Check"); ui.strong("State"); ui.strong("Linux Status"); ui.end_row();
                    ui.label(if is_gcn { "POWER_CONTAINMENT" } else { "DPM_GFXCLK" });
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
                    let card_id = sel_lbl.split("(").nth(1).and_then(|s| s.split_whitespace().next()).filter(|s| s.starts_with("card")).unwrap_or("card0");
                    ui.label("sysfs Target");
                    ui.monospace(format!("{}/device/pp_table", card_id));
                    ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "READY");
                    ui.end_row();
                    let (mask_detail, mask_badge, mask_color) = check_linux_ppfeaturemask();
                    ui.label("ppfeaturemask");
                    ui.monospace(mask_detail);
                    ui.colored_label(mask_color, mask_badge);
                    ui.end_row();
                    let (svc_detail, svc_badge, svc_color) = check_linux_systemd_service();
                    ui.label("Boot Service");
                    ui.monospace(svc_detail);
                    ui.colored_label(svc_color, svc_badge);
                    ui.end_row();
                    let (sync_detail, sync_badge, sync_color) = check_linux_ppt_sync(&sel_lbl);
                    ui.label("Config Sync");
                    ui.monospace(sync_detail);
                    ui.colored_label(sync_color, sync_badge);
                    ui.end_row();
                });
            });
        
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
        
    }
}
