use crate::advance_os::*;
use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn draw_advance_windows(&mut self, ui: &mut egui::Ui) {
        let dpm_gfx = self.smu_features[1];
        let (pwr_w, tdc_g, vg1) = (self.power_limit_w, self.tdc_gfx, self.volt_gfx[1]);
        let od8_ok = self.gpu_clock_limit && self.power_limit_feat && self.fan_curve;
        let (clk_en, pwr_en, fan_en) = (self.gpu_clock_limit, self.power_limit_feat, self.fan_curve);

                    draw_card(ui, "Windows Status", |ui| {
                egui::Grid::new("advance_windows_grid").num_columns(3).min_col_width(((ui.available_width() - 32.0) / 3.0).max(110.0)).striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Check"); ui.strong("State"); ui.strong("Windows Status"); ui.end_row();
                    ui.label("DPM_GFXCLK");
                    ui.monospace(if dpm_gfx { "Enabled (1)" } else { "Disabled (0)" });
                    ui.colored_label(
                        if dpm_gfx { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) },
                        if dpm_gfx { "OK" } else { "DISABLED" },
                    );
                    ui.end_row();
                    ui.label("WattMan OD8");
                    ui.monospace(format!("CLK={} PWR={} FAN={}", clk_en as u8, pwr_en as u8, fan_en as u8));
                    ui.colored_label(
                        if od8_ok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) },
                        if od8_ok { "UNLOCKED" } else { "PARTIAL" },
                    );
                    ui.end_row();
                    ui.label("Registry Override");
                    let reg_active = self.ioreg_entries.get(self.selected_gpu).and_then(|e| e.ppt_bytes.as_ref()).map(|b| !b.is_empty()).unwrap_or(false) && cfg!(target_os = "windows");
                    ui.monospace(if reg_active { format!("PP_PhmSoftPowerPlayTable ({} B)", self.raw_buffer.len()) } else { "Factory (No Override)".to_string() });
                    ui.colored_label(
                        if reg_active { egui::Color32::from_rgb(245, 185, 65) } else { egui::Color32::from_rgb(80, 210, 120) },
                        if reg_active { "PATCHED" } else { "FACTORY" },
                    );
                    ui.end_row();
                    ui.label("PPT / TDC / Vmax");
                    ui.monospace(format!("{}W / {}A / {}mV", pwr_w, tdc_g, vg1));
                    let ok = pwr_w > 0 && tdc_g > 0 && vg1 <= 1250; ui.colored_label(if ok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if ok { "OK" } else { "CHECK" });
                    ui.end_row();
                });
            });
        
            draw_card(ui, "Windows MPT Export", |ui| {
                ui.label("Export or copy synced PP_PhmSoftPowerPlayTable for Windows Registry / MorePowerTool:");
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Write to Registry").clicked() {
                        self.sync_fields_to_buffer();
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match write_ppt_to_windows_registry(&self.raw_buffer, &sel_lbl) {
                            Ok(msg) => {
                                crate::ui_theme::log_debug("WIN_REG", &msg);
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                            }
                            Err(err) => self.show_error("Windows Registry Write Error", &err),
                        }
                    }
                    if ui.button("Delete Registry Key").clicked() {
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match delete_ppt_from_windows_registry(&sel_lbl) {
                            Ok(msg) => {
                                crate::ui_theme::log_debug("WIN_REG", &msg);
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                            }
                            Err(err) => self.show_error("Windows Registry Delete Error", &err),
                        }
                    }
                    if ui.button("Export Windows .reg / .mpt").clicked() {
                        self.save_ppt_file_with_default("custom_ppt.reg");
                    }
                    if ui.button("Copy Raw Hex (No Space)").clicked() {
                        self.sync_fields_to_buffer();
                        let hex_raw: String = self.raw_buffer.iter().map(|b| format!("{:02X}", b)).collect();
                        ui.ctx().copy_text(hex_raw);
                        self.status_msg = "Copied raw PP_PhmSoftPowerPlayTable Hex".into();
                    }
                });
            });
        
    }

}
