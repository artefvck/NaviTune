use eframe::egui;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_fan_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
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
        if arch.has_5point_fan_curve_slots() {
            for i in 1..5 {
                if self.fan_curve_temp[i] < self.fan_curve_temp[i - 1]
                    || self.fan_curve_pwm[i] < self.fan_curve_pwm[i - 1]
                {
                    issues.push("5-Point Fan Curve not ascending");
                    break;
                }
            }
        }
        if !issues.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Fan Check: {}", issues.join(" | ")));
            ui.add_space(4.0);
        }


        let show_zerorpm = self.is_card_usable(crate::ppt_arch::PptCard::ZeroRpm);
        let show_fancurve = self.is_card_usable(crate::ppt_arch::PptCard::FanCurve);
        let _show_thermallimits = self.is_card_usable(crate::ppt_arch::PptCard::ThermalLimits);
        if !arch.supports_card(crate::ppt_arch::PptCard::FanSettingsAndLimits) && !show_zerorpm && !self.is_card_usable(crate::ppt_arch::PptCard::FanCurve) && !self.is_card_usable(crate::ppt_arch::PptCard::ThermalLimits) {
            crate::card_layout::draw_arch_note_card(ui, &format!("{} — Thermal & Fan Architecture", arch.label()), "No discrete fan or thermal sub-table is present in this PPTable revision.");
            return;
        }
        let (pwm_t, ac_t, thr_t, max_t, tgt_t, gfx_t, stop_t, start_t) = arch.fan_offset_labels();
        two_col_cards_adaptive(ui, "fan_cols", true, show_zerorpm || show_fancurve, |ui| {
            draw_card(ui, "Fan Settings", |ui| {
                crate::card_layout::kv_header_1(ui, "Value", 74.0);
                kv_row_1(ui, "PWM Min (%)", pwm_t, &mut self.fan_pwm_min, 62.0);
                if arch.supports_card_in_buf(crate::ppt_arch::PptCard::FanLimits, &self.raw_buffer) {
                    kv_row_1(ui, "Acoustic Limit (RPM)", ac_t, &mut self.fan_acoustic_limit, 62.0);
                    kv_row_1(ui, "Throttling RPM", thr_t, &mut self.fan_throttling_rpm, 62.0);
                }
                kv_row_1(ui, "Max RPM", max_t, &mut self.fan_max_rpm, 62.0);
                kv_row_1(ui, "Target Temp (°C)", tgt_t, &mut self.fan_target_temp, 62.0);
                kv_row_1(ui, "Target GFX Clock", gfx_t, &mut self.fan_target_gfx_clock, 62.0);
                if arch.has_5point_fan_curve_slots() {
                    kv_row_1(ui, "Fan Mode", "fan_mode (0x724)", &mut self.fan_mode, 62.0);
                    kv_row_1(ui, "Hyst / Ctrl", "fan_hyst_ctrl (0x725)", &mut self.fan_hyst_ctrl, 62.0);
                }
            });
            if arch.supports_card_in_buf(crate::ppt_arch::PptCard::FanLimits, &self.raw_buffer) {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::FanLimits), |ui| {
                    kv_header_2(ui, "Min", "Max", 62.0);
                    let (_, _, _, fan_od_tag, ac_od_tag) = arch.od_offset_labels();
                    kv_row_2(ui, "Fan RPM", fan_od_tag, &mut self.oc_fan_rpm, 62.0);
                    kv_row_2(ui, "Acoustic RPM", ac_od_tag, &mut self.oc_fan_acoustic_rpm, 62.0);
                    if arch.has_5point_fan_curve_slots() {
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
                            let info = format!("Zero RPM › Enable Zero RPM ({}) · {}", arch.zero_rpm_enable_offset_label(), state);
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                            resp.on_hover_text(info);
                        }
                    });
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
                    kv_row_1(ui, "Stop Temp (°C)", stop_t, &mut self.fan_stop_temp, 62.0);
                    kv_row_1(ui, "Start Temp (°C)", start_t, &mut self.fan_start_temp, 62.0);
                });
            }
            if show_fancurve {
                draw_card(ui, "Fan Curve", |ui| {
                    if !arch.has_5point_fan_curve_slots() {
                        ui.colored_label(egui::Color32::from_rgb(160, 175, 195), arch.fan_curve_note());
                    } else {
                        let has_pts = self.fan_curve_temp.iter().any(|&t| t > 0) || self.fan_curve_pwm.iter().any(|&p| p > 0);
                        ui.add_enabled_ui(has_pts, |ui| {
                            kv_header_2(ui, "Temp (°C)", "PWM (%)", 62.0);
                            for i in 0..5 {
                                let mut pair = [self.fan_curve_temp[i], self.fan_curve_pwm[i]];
                                kv_row_2(ui, &format!("Point {}", i + 1), &format!("fan_pt_{}", i), &mut pair, 62.0);
                                self.fan_curve_temp[i] = pair[0];
                                self.fan_curve_pwm[i] = pair[1];
                            }
                        }).response.on_disabled_hover_text("Read-only: VBIOS ini menggunakan kurva kipas dinamis SMU (slot 5-titik bernilai 0).");
                    }
                });
            }
        });
    
        crate::card_layout::draw_card(ui, "Interactive Fan Curve (Drag Points)", |ui| {
            if self.raw_buffer.is_empty() { ui.disable(); }
            self.draw_fan_curve_plot(ui);
        });
}
}
