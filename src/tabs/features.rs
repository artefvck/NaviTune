use eframe::egui;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_features_tab(&mut self, ui: &mut egui::Ui) {
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let loaded = !self.raw_buffer.is_empty();
        let gen = ui.ctx().data(|d| d.get_temp::<u64>(egui::Id::new("ppt_baseline_gen")).unwrap_or(0));

        if arch.has_od8_and_smu_masks() {
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new("PPTable Features").min_size(egui::vec2(180.0, 28.0));
                if ui.add(btn).clicked() {
                    self.show_smu_features_window = !self.show_smu_features_window;
                }
            });
            ui.add_space(6.0);
        }

        let card_title = if arch.is_gcn_subtable() {
            "OverDrive & Platform Capabilities (0x013)"
        } else {
            "OverDrive Features"
        };

        crate::card_layout::draw_card(ui, card_title, |ui| {
            ui.add_enabled_ui(loaded, |ui| {
                ui.columns(2, |cols| {
                    cols[0].spacing_mut().item_spacing.y = 4.0;
                    cols[1].spacing_mut().item_spacing.y = 4.0;
                    let left_items: [(&mut bool, &str, &str, bool); 8] = [
                        (&mut self.gpu_clock_limit, "GPU Clock Limit", "od_feat_0", true),
                        (&mut self.gpu_clock_curve, "GPU Clock Curve", "od_feat_1", !arch.is_gcn_subtable()),
                        (&mut self.memory_clock_limit, "Memory Clock Limit", "od_feat_2", true),
                        (&mut self.zero_rpm, "Zero RPM", "od_feat_3", arch != crate::ppt_arch::PptArch::Polaris),
                        (&mut self.min_fan_speed, "Minimum Fan Speed", "od_feat_4", true),
                        (&mut self.fan_target_temp_limit, "Fan Target Temperature Limit", "od_feat_5", true),
                        (&mut self.operating_temp_limit, "Operating Temperature Limit", "od_feat_6", true),
                        (&mut self.fan_acoustic_rpm, "Fan Acoustic RPM", "od_feat_7", !arch.is_gcn_subtable()),
                    ];
                    for (flag, lbl, tag, supported) in left_items {
                        if !supported {
                            continue;
                        }
                        let base_id = egui::Id::new(("od_feat_base", gen, tag));
                        let base_val = ui.ctx().data_mut(|d| {
                            if let Some(v) = d.get_temp::<bool>(base_id) {
                                v
                            } else {
                                d.insert_temp(base_id, *flag);
                                *flag
                            }
                        });
                        cols[0].horizontal(|ui| {
                            let resp = crate::ui_theme::green_checkbox(ui, flag, lbl);
                            if resp.changed() {
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            if *flag != base_val {
                                ui.colored_label(egui::Color32::from_rgb(240, 70, 90), "●");
                            }
                            if resp.hovered() || resp.clicked() {
                                let state = if *flag { "Enabled" } else { "Disabled" };
                                let def_str = if base_val { "Enabled" } else { "Disabled" };
                                let info = if *flag != base_val {
                                    format!("{} › {} ({}) · {} (Factory: {})", card_title, lbl, tag, state, def_str)
                                } else {
                                    format!("{} › {} ({}) · {}", card_title, lbl, tag, state)
                                };
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                                resp.on_hover_text(info);
                            }
                        });
                    }
                    let right_items: [(&mut bool, &str, &str, bool); 8] = [
                        (&mut self.fan_curve, "Fan Curve", "od_feat_8", !arch.is_gcn_subtable()),
                        (&mut self.power_limit_feat, "Power Limit", "od_feat_9", true),
                        (&mut self.timing_tuning, "Timing Tuning", "od_feat_10", !arch.is_gcn_subtable()),
                        (&mut self.power_mode, "Power Mode", "od_feat_11", arch.has_multi_power_modes()),
                        (&mut self.auto_undervolt_gpu, "Auto Undervolt GPU", "od_feat_12", !arch.is_gcn_subtable()),
                        (&mut self.auto_overclock_gpu, "Auto Overclock GPU", "od_feat_13", !arch.is_gcn_subtable()),
                        (&mut self.auto_overclock_mem, "Auto Overclock Memory", "od_feat_14", !arch.is_gcn_subtable()),
                        (&mut self.auto_fan_acoustic_limit, "Auto Fan Acoustic Limit", "od_feat_15", !arch.is_gcn_subtable()),
                    ];
                    for (flag, lbl, tag, supported) in right_items {
                        if !supported {
                            continue;
                        }
                        let base_id = egui::Id::new(("od_feat_base", gen, tag));
                        let base_val = ui.ctx().data_mut(|d| {
                            if let Some(v) = d.get_temp::<bool>(base_id) {
                                v
                            } else {
                                d.insert_temp(base_id, *flag);
                                *flag
                            }
                        });
                        cols[1].horizontal(|ui| {
                            let resp = crate::ui_theme::green_checkbox(ui, flag, lbl);
                            if resp.changed() {
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            if *flag != base_val {
                                ui.colored_label(egui::Color32::from_rgb(240, 70, 90), "●");
                            }
                            if resp.hovered() || resp.clicked() {
                                let state = if *flag { "Enabled" } else { "Disabled" };
                                let def_str = if base_val { "Enabled" } else { "Disabled" };
                                let info = if *flag != base_val {
                                    format!("{} › {} ({}) · {} (Factory: {})", card_title, lbl, tag, state, def_str)
                                } else {
                                    format!("{} › {} ({}) · {}", card_title, lbl, tag, state)
                                };
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                                resp.on_hover_text(info);
                            }
                        });
                    }
                });
            });
        });
    }
}
