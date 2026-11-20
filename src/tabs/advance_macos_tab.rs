use crate::advance_os::*;
use crate::card_layout::{draw_card, two_col_cards};
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn draw_advance_macos(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let vcn_bits = [(21usize, "21: VCN_PG"), (22, "22: MM_DPM_PG"), (29, "29: DPM_VCLK"), (30, "30: DPM_DCLK")];
            let mut all_vcn = vcn_bits.iter().all(|(b, _)| self.smu_features[*b]);
            let panel_id = egui::Id::new("macos_vcn_panel_open");
            let mut open = ui.ctx().data(|d| d.get_temp::<bool>(panel_id).unwrap_or(false));

            if ui.button(if open { "Hide VCN (macOS)" } else { "Enable VCN (macOS)" }).clicked() {
                open = !open;
                ui.ctx().data_mut(|d| d.insert_temp(panel_id, open));
            }
            if crate::ui_theme::green_checkbox(ui, &mut all_vcn, "Set All VCN SMU Mask (21, 22, 29, 30)").changed() {
                for (b, _) in vcn_bits {
                    self.smu_features[b] = all_vcn;
                }
                self.sync_fields_to_buffer();
            }
            if open {
                ui.separator();
                for (bit, lbl) in vcn_bits {
                    if crate::ui_theme::green_checkbox(ui, &mut self.smu_features[bit], lbl).changed() {
                        self.sync_fields_to_buffer();
                    }
                }
            }
        });
        ui.add_space(4.0);
        let (vcn_pg, gfxoff, baco) = (self.smu_features[22], self.smu_features[20], self.smu_features[21]);
        let (fan_stop, fan_start, fan_pwm) = (self.fan_stop_temp, self.fan_start_temp, self.fan_pwm_min);
        let (vg0, vg1) = (self.volt_gfx[0], self.volt_gfx[1]);

        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
        two_col_cards(ui, "advance_macos_cols", |ui| {
            draw_card(ui, "macOS OpenCore Export", |ui| {
                ui.label("Export or copy synced PP_PhmSoftPowerPlayTable for OpenCore DeviceProperties:");
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Copy OpenCore Base64/Hex").clicked() {
                        self.sync_fields_to_buffer();
                        self.show_opencore_copy_modal = true;
                    }
                    if ui.button("Export .plist").clicked() {
                        self.save_ppt_file_with_default("DeviceProperties.plist");
                    }
                    if ui.button("Export Injector .kext").clicked() {
                        self.sync_fields_to_buffer();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("RadeonPowerPlayInjector.kext")
                            .save_file()
                        {
                            match export_macos_injector_kext(&self.raw_buffer, &path) {
                                Ok(msg) => self.status_msg = msg,
                                Err(err) => self.show_error("Injector .kext Export Error", &err),
                            }
                        }
                    }
                    let id_aml_modal = egui::Id::new("aml_export_modal_open");
                    if ui.button("Export ACPI .aml").clicked() {
                        self.sync_fields_to_buffer();
                        ui.ctx().data_mut(|d| d.insert_temp(id_aml_modal, true));
                    }
                    let mut aml_modal_open = ui.ctx().data(|d| d.get_temp::<bool>(id_aml_modal).unwrap_or(false));
                    if aml_modal_open {
                        let mut dis_pc = ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("aml_dis_pc")).unwrap_or(true));
                        let mut dis_ulps = ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("aml_dis_ulps")).unwrap_or(true));
                        let mut win_open = true;
                        egui::Window::new("Export ACPI .aml Options")
                            .open(&mut win_open)
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                            .show(ui.ctx(), |ui| {
                                ui.label("Select macOS ACPI _DSM properties to include:");
                                ui.add_space(6.0);
                                if crate::ui_theme::green_checkbox(ui, &mut dis_pc, "PP_DisablePowerContainment").changed() {
                                    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("aml_dis_pc"), dis_pc));
                                }
                                if crate::ui_theme::green_checkbox(ui, &mut dis_ulps, "PP_DisableULPS").changed() {
                                    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("aml_dis_ulps"), dis_ulps));
                                }
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    if ui.button("Save .aml / .dsl...").clicked() {
                                        aml_modal_open = false;
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_file_name("SSDT-sPPT.aml")
                                            .add_filter("Compiled ACPI Table (*.aml)", &["aml"])
                                            .add_filter("ACPI Source Language (*.dsl)", &["dsl"])
                                            .save_file()
                                        {
                                            let acpi_path = detect_live_acpi_gpu_path(&sel_lbl);
                                            let dsl = format_opencore_acpi_dsl(&self.raw_buffer, &acpi_path, dis_pc, dis_ulps);
                                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("aml").to_ascii_lowercase();
                                            if ext == "dsl" {
                                                match std::fs::write(&path, dsl) {
                                                    Ok(_) => self.status_msg = format!("Exported {} ({})", path.file_name().unwrap_or_default().to_string_lossy(), acpi_path),
                                                    Err(e) => self.show_error("ACPI .dsl Export Error", &e.to_string()),
                                                }
                                            } else {
                                                match compile_dsl_to_aml(&dsl, &path) {
                                                    Ok(msg) => self.status_msg = format!("{} [{}]", msg, acpi_path),
                                                    Err(err) => self.show_error("iasl Compiler Not Found / Failed", &err),
                                                }
                                            }
                                        }
                                    }
                                    if ui.button("Cancel").clicked() {
                                        aml_modal_open = false;
                                    }
                                });
                            });
                        if !win_open {
                            aml_modal_open = false;
                        }
                        ui.ctx().data_mut(|d| d.insert_temp(id_aml_modal, aml_modal_open));
                    }
                });
            });
        }, |ui| {
            draw_card(ui, "macOS Driver & Sleep/Wake Status", |ui| {
                egui::Grid::new("advance_macos_grid").num_columns(3).min_col_width(((ui.available_width() - 32.0) / 3.0).max(110.0)).striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Check"); ui.strong("State"); ui.strong("macOS Status"); ui.end_row();
                    ui.label("VCN_PG (Bit 22)");
                    ui.monospace(if vcn_pg { "Enabled (1)" } else { "Disabled (0)" });
                    ui.colored_label(
                        if !vcn_pg { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) },
                        if !vcn_pg { "OPTIMAL" } else { "WARN" },
                    );
                    ui.end_row();
                    ui.label("GFXOFF / BACO");
                    ui.monospace(format!("{} / {}", gfxoff as u8, baco as u8));
                    ui.colored_label(egui::Color32::from_rgb(80, 210, 120), if gfxoff && baco { "OK" } else { "WARN" });
                    ui.end_row();
                    ui.label("Fan Hysteresis");
                    ui.monospace(format!("{}°C / {}°C ({}%)", fan_stop, fan_start, fan_pwm));
                    ui.colored_label(egui::Color32::from_rgb(80, 210, 120), if fan_stop < fan_start { "OK" } else { "FIX" });
                    ui.end_row();
                    ui.label("GFX Voltage");
                    ui.monospace(format!("{}..{} mV", vg0, vg1));
                    let v_ok = vg0 > 0 && vg0 <= vg1 && vg1 <= 1250;
                    ui.colored_label(
                        if v_ok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) },
                        if v_ok { "OK" } else { "CHECK" },
                    );
                    ui.end_row();
                    let acpi_path = detect_live_acpi_gpu_path(&sel_lbl);
                    ui.label("Live ACPI Path");
                    ui.monospace(&acpi_path);
                    ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "DETECTED");
                    ui.end_row();
                });
            });
        });
    }

}
