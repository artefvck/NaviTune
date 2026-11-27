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
                    let card_id = detect_linux_card(&sel_lbl);
                    ui.label("sysfs Target");
                    ui.monospace(&sysfs_target);
                    ui.colored_label(
                        if sysfs_ok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) },
                        if sysfs_ok { "READY" } else { "SET PPFEATUREMASK" },
                    );
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
                    let (profiles, prof_detail) = read_linux_power_profiles(&sel_lbl);
                    ui.label("Power Profile");
                    if profiles.is_empty() {
                        ui.monospace(&prof_detail);
                        ui.colored_label(egui::Color32::from_rgb(150, 160, 180), "N/A");
                    } else {
                        let cur_idx = profiles.iter().find(|(_, _, act)| *act).map(|(i, _, _)| *i).unwrap_or(0);
                        let cur_lbl = profiles.iter().find(|(_, _, act)| *act)
                            .map(|(i, n, _)| format!("{}: {}", i, n))
                            .unwrap_or_else(|| prof_detail.clone());
                        let mut picked: Option<(u32, String)> = None;
                        egui::ComboBox::from_id_salt("linux_power_profile_combo")
                            .width(220.0)
                            .selected_text(cur_lbl)
                            .show_ui(ui, |ui| {
                                for (idx, name, act) in &profiles {
                                    let item_lbl = if *act { format!("{}: {} *", idx, name) } else { format!("{}: {}", idx, name) };
                                    if ui.selectable_label(*act, item_lbl).clicked() && *idx != cur_idx {
                                        picked = Some((*idx, name.clone()));
                                    }
                                }
                            });
                        if let Some((new_idx, new_name)) = picked {
                            match set_linux_power_profile(&sel_lbl, new_idx, &new_name) {
                                Ok(msg) => {
                                    crate::ui_theme::log_debug("LINUX_PROFILE", &msg);
                                    self.status_msg = msg;
                                }
                                Err(err) => self.show_error("Power Profile Error", &err),
                            }
                        }
                        ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "ACTIVE");
                    }
                    ui.end_row();
                });
            });

            draw_card(ui, "Linux sysfs Export", |ui| {
                let active_card = detect_linux_card(&sel_lbl);
                ui.label(format!("Export pp_table binary or copy sysfs command for /sys/class/drm/{}/device/pp_table:", active_card));
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Import Tuning Values Only...").on_hover_text("Copy only clocks, voltages, power limits & fan curve from a donor .mpt/.rom/.plist onto the currently loaded native GPU table without overwriting binary subtable structures").clicked() {
                    if let Some(path) = crate::io::pick_file_dialog(
                        "Select Donor Profile (.mpt, .rom, .bin, .plist)",
                        &[("PowerPlay / VBIOS", &["mpt", "rom", "bin", "ppt", "plist", "reg"])],
                    ) {
                        match crate::io::read_bounded(&path) {
                            Ok(bytes) => match self.apply_tuning_values_only(&path, &bytes) {
                                Ok(msg) => {
                                    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                                    self.status_msg = msg;
                                }
                                Err(e) => {
                                    self.show_error("Import Tuning Values Error", &e);
                                }
                            },
                            Err(e) => {
                                self.status_msg = format!("Failed reading donor file: {}", e);
                            }
                        }
                    }
                }
                if ui.button("Install").clicked() {
                        self.sync_fields_to_buffer();
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match install_linux_ppt(self, &sel_lbl) {
                            Ok(msg) => {
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                                if prev < self.ioreg_entries.len() { self.selected_gpu = prev; }
                            }
                            Err(err) => self.show_error("Linux Install Error", &err),
                        }
                    }
                    if ui.button("Uninstall").clicked() {
                        let prev = self.selected_gpu;
                        let lbl = self.gpus.get(prev).cloned().unwrap_or_default();
                        match uninstall_linux_ppt(&lbl) {
                            Ok(msg) => {
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                                if prev < self.ioreg_entries.len() { self.selected_gpu = prev; }
                            }
                            Err(err) => self.show_error("Linux Uninstall Error", &err),
                        }
                    }
                    if ui.button("Export pp_table (.bin)").clicked() {
                        self.save_ppt_file_with_default("pp_table.bin");
                    }
                    if ui.button("Write to GPU (sysfs)")
                        .on_hover_text("Write synced PowerPlayTable directly to /sys/class/drm/cardX/device/pp_table (temporary until reboot)")
                        .clicked()
                    {
                        self.sync_fields_to_buffer();
                        let sel_lbl = self.gpus.get(self.selected_gpu).cloned().unwrap_or_default();
                        match write_linux_sysfs_ppt(&self.raw_buffer, &sel_lbl, self.freq_gfx) {
                            Ok(msg) => {
                                crate::ui_theme::log_debug("LINUX_SYSFS", &msg);
                                self.status_msg = msg;
                                self.refresh_ioreg_gpus();
                            }
                            Err(err) => self.show_error("Linux sysfs Write Error", &err),
                        }
                    }
                });
            });
    }
}
