use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_fan_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let _fan_tags = arch.fan_offset_labels();
        let is_rdna3 = arch.is_rdna3();
        let is_rdna2 = arch.is_rdna2();
        let is_rdna1 = arch.is_rdna1() || arch.is_vega() || arch.is_gcn_subtable();
        if arch.is_gcn_subtable() && (self.fan_pwm_min < 0 || self.fan_pwm_min > 100) {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), "Fan Check: Fan PWM Min out of 0..100%");
            ui.add_space(4.0);
        }
        if let Some(hint) = arch.gcn_subtable_hint("fan") {
            ui.colored_label(egui::Color32::from_rgb(235, 190, 95), hint);
            ui.add_space(6.0);
        }
        let mut issues: Vec<&'static str> = Vec::new();
        if self.fan_pwm_min < 0 || self.fan_pwm_min > 100 {
            issues.push("PWM Min out of 0..100%");
        }
        if self.fan_max_rpm > 0 && self.fan_acoustic_limit > self.fan_max_rpm {
            issues.push("Acoustic Limit > Max RPM");
        }
        if self.fan_zero_rpm_enable && self.fan_stop_temp >= self.fan_start_temp {
            issues.push("Zero RPM Stop Temp >= Start Temp");
        }
        if self.fan_start_temp > 0 && self.fan_target_temp > 0 && self.fan_start_temp >= self.fan_target_temp {
            issues.push("Start Temp >= Target Temp");
        }
        if is_rdna2 {
            for i in 1..5 {
                if self.fan_curve_temp[i] < self.fan_curve_temp[i - 1]
                    || self.fan_curve_pwm[i] < self.fan_curve_pwm[i - 1]
                {
                    issues.push("5-Point Fan Curve not ascending");
                    break;
                }
            }
        }


        let show_zerorpm = self.is_card_usable(crate::ppt_arch::PptCard::ZeroRpm);
        let show_fancurve = self.is_card_usable(crate::ppt_arch::PptCard::FanCurve);
        let show_thermallimits = self.is_card_usable(crate::ppt_arch::PptCard::ThermalLimits);
        if !arch.supports_card(crate::ppt_arch::PptCard::FanSettingsAndLimits) && !show_zerorpm && !self.is_card_usable(crate::ppt_arch::PptCard::FanCurve) && !self.is_card_usable(crate::ppt_arch::PptCard::ThermalLimits) {
            crate::card_layout::draw_arch_note_card(ui, &format!("{} — Thermal & Fan Architecture", arch.label()), "No discrete fan or thermal sub-table is present in this PPTable revision.");
            return;
        }
        two_col_cards_adaptive(ui, "fan_cols", true, show_zerorpm || show_fancurve || show_thermallimits, |ui| {
            draw_card(ui, "Fan Settings", |ui| {
                kv_row_1(ui, "PWM Min (%)", "fan_pwm_min (0x716 / 0x86C)", &mut self.fan_pwm_min, 62.0);
                kv_row_1(ui, "Acoustic Limit (RPM)", "fan_acoustic_limit (0x718 / 0x86E)", &mut self.fan_acoustic_limit, 62.0);
                kv_row_1(ui, "Throttling RPM", "fan_throttling_rpm (0x71A / 0x86A)", &mut self.fan_throttling_rpm, 62.0);
                kv_row_1(ui, "Max RPM", "fan_max_rpm (0x71C / 0x870)", &mut self.fan_max_rpm, 62.0);
                kv_row_1(ui, "Target Temp (°C)", "fan_target_temp (0x720 / 0x878)", &mut self.fan_target_temp, 62.0);
                kv_row_1(ui, "Target GFX Clock", "fan_target_gfx_clock (0x722 / 0x876)", &mut self.fan_target_gfx_clock, 62.0);
                if is_rdna2 {
                    kv_row_1(ui, "Fan Mode", "fan_mode (0x71E)", &mut self.fan_mode, 62.0);
                    kv_row_1(ui, "Hyst / Ctrl", "fan_hyst_ctrl (0x71F)", &mut self.fan_hyst_ctrl, 62.0);
                }
            });
            if !arch.is_gcn_subtable() {
                draw_card(ui, "Fan Limits", |ui| {
                    kv_header_2(ui, "Min", "Max", 62.0);
                    kv_row_2(ui, "Fan RPM", "od_fan_min/max (0x106 / 0x113)", &mut self.oc_fan_rpm, 62.0);
                    kv_row_2(ui, "Acoustic RPM", "od_acoustic_min/max (0x10A / 0x117)", &mut self.oc_fan_acoustic_rpm, 62.0);
                    if is_rdna2 {
                        kv_row_2(ui, "OD Temp (°C)", "od_fan_curve_min/max", &mut self.od_fan_temp, 62.0);
                        kv_row_2(ui, "OD Speed (%)", "od_fan_curve_min/max", &mut self.od_fan_speed, 62.0);
                    }
                });
            }
            }, |ui| {
                if show_zerorpm {
                    draw_card(ui, "Zero RPM", |ui| {
                        ui.horizontal(|ui| {
                            let resp = crate::ui_theme::green_checkbox(ui, &mut self.fan_zero_rpm_enable, "Enable Zero RPM");
                            if resp.hovered() || resp.clicked() {
                                let state = if self.fan_zero_rpm_enable { "Enabled" } else { "Disabled" };
                                let info = format!("Zero RPM › Enable Zero RPM (0x726 / 0x87C) · {}", state);
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                                resp.on_hover_text(info);
                            }
                        });
                    }
                    if self.fan_zero_rpm_enable && !self.zero_rpm {
                        ui.horizontal_wrapped(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(245, 185, 65),
                                egui::RichText::new("⚠ OD8 Zero RPM feature flag is off").small(),
                            );
                            if ui.small_button("Enable in Features").clicked() {
                                self.zero_rpm = true;
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                        });
                    }
                    ui.add_space(4.0);
                    kv_row_1(ui, "Stop Temp (°C)", "fan_stop_temp (0x728 / 0x81E)", &mut self.fan_stop_temp, 62.0);
                    kv_row_1(ui, "Start Temp (°C)", "fan_start_temp (0x72A / 0x838)", &mut self.fan_start_temp, 62.0);
                });
            }
            if arch.has_fan_curve_points() {
                }
            if show_fancurve {
                draw_card(ui, "Fan Curve", |ui| {
                    if is_rdna3 || is_rdna1 {
                        ui.colored_label(
                            egui::Color32::from_rgb(160, 175, 195),
                            "RDNA 3 (SMU 13.0) computes dynamic fan PWM from Target Temp (101°C) & Acoustic Limit; 5-point curve slots are RDNA 2 only.",
                        );
                    } else {
                        kv_header_2(ui, "Temp (°C)", "PWM (%)", 62.0);
                        for i in 0..5 {
                            let mut pair = [self.fan_curve_temp[i], self.fan_curve_pwm[i]];
                            kv_row_2(ui, &format!("Point {}", i + 1), &format!("fan_pt_{}", i), &mut pair, 62.0);
                            self.fan_curve_temp[i] = pair[0];
                            self.fan_curve_pwm[i] = pair[1];
                        }
                    }
                });
            }
            }
        });

    }
}

