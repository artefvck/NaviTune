use eframe::egui;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_oc_limits_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);

        let mut issues: Vec<&str> = Vec::new();
        if arch.has_od8_and_smu_masks() && !arch.has_dynamic_power_modes_only() {
            if self.oc_gfx_clock[0] > self.oc_gfx_clock[1] { issues.push("GFX Clock Min > Max"); }
            if self.oc_mem_clock[0] > self.oc_mem_clock[1] { issues.push("Memory Clock Min > Max"); }
            if self.oc_power_limit[0] > self.oc_power_limit[1] || self.oc_power_limit[1] > 50 { issues.push("Power Limit (%) inverted or > +50%"); }
            if self.oc_fan_rpm[0] > self.oc_fan_rpm[1] { issues.push("Fan RPM Min > Max"); }
            if self.oc_fan_acoustic_rpm[0] > self.oc_fan_acoustic_rpm[1] { issues.push("Fan Acoustic RPM Min > Max"); }
        } else if self.oc_power_limit[1] > 50 {
            issues.push("Power Limit Max > +50%");
        }
        if !issues.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("OverDrive Check: {}", issues.join(" | ")));
            ui.add_space(4.0);
        }

        if !arch.supports_card(crate::ppt_arch::PptCard::OverDrive) && !self.is_card_usable(crate::ppt_arch::PptCard::PowerModes) && !self.is_card_usable(crate::ppt_arch::PptCard::PowerModeAcoustic) {
            crate::card_layout::draw_arch_note_card(ui, &format!("{} — OverDrive Architecture", arch.label()), "This PPTable revision does not expose OverDrive ceiling sliders or multi-preset Power Modes.");
            return;
        }
        let show_powermodes = self.is_card_usable(crate::ppt_arch::PptCard::PowerModes);
        let show_powermodeacoustic = self.is_card_usable(crate::ppt_arch::PptCard::PowerModeAcoustic);
        two_col_cards_adaptive(ui, "oc_cols", true, show_powermodes || show_powermodeacoustic, |ui| {
            draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::OverDrive), |ui| {
                let (gfx_tag, mem_tag, pwr_tag, fan_tag, ac_tag) = arch.od_offset_labels();
                if !arch.has_od8_and_smu_masks() {
                    kv_row_1(ui, "GFX Clock Max (MHz)", gfx_tag, &mut self.oc_gfx_clock[1], 62.0);
                    kv_row_1(ui, "Memory Clock Max (MHz)", mem_tag, &mut self.oc_mem_clock[1], 62.0);
                    kv_row_1(ui, "Power Limit (+%)", pwr_tag, &mut self.oc_power_limit[1], 62.0);
                } else {
                    kv_header_2(ui, "Min", "Max", 62.0);
                    kv_row_2(ui, "GFX Clock (MHz)", gfx_tag, &mut self.oc_gfx_clock, 62.0);
                    kv_row_2(ui, "Memory Clock (MHz)", mem_tag, &mut self.oc_mem_clock, 62.0);
                    kv_row_2(ui, "Power Limit (%)", pwr_tag, &mut self.oc_power_limit, 62.0);
                    if arch.supports_card_in_buf(crate::ppt_arch::PptCard::FanLimits, &self.raw_buffer) {
                        kv_row_2(ui, "Fan Max RPM", fan_tag, &mut self.oc_fan_rpm, 62.0);
                        kv_row_2(ui, "Fan Acoustic RPM", ac_tag, &mut self.oc_fan_acoustic_rpm, 62.0);
                    }
                    if arch.has_od_ctrl_toggles() {
                        ui.add_space(4.0);
                        let (mt_tag, pm_tag, zr_tag) = arch.od_ctrl_offset_labels();
                        kv_row_1(ui, "Mem Timing Ctrl", mt_tag, &mut self.oc_mem_timing_ctrl, 62.0);
                        kv_row_1(ui, "Power Mode Ctrl", pm_tag, &mut self.oc_power_mode_ctrl, 62.0);
                        kv_row_1(ui, "Zero RPM Ctrl", zr_tag, &mut self.oc_zero_rpm_ctrl, 62.0);
                    }
                }
            });
        }, |ui| {
            if show_powermodes {
                if arch.has_dynamic_power_modes_only() {
                    draw_card(ui, "Power Modes", |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(160, 175, 195),
                            "RDNA 3 (SMU 13.0) manages Quiet, Balanced, and Rage presets dynamically in SMU firmware; static PPTable slots are unused.",
                        );
                    });
                } else {
                    let compact_smu11 = arch.has_compact_power_modes(&self.raw_buffer);
                    draw_card(ui, "Power Modes", |ui| {
                        if compact_smu11 {
                            ui.colored_label(egui::Color32::from_rgb(160, 175, 195), "Compact VBIOS PPTable omits static Power Mode preset slots (available on 1924 B V4_5 / RDNA 2).");
                            ui.add_space(4.0);
                        }
                        ui.add_enabled_ui(!compact_smu11, |ui| {
                            kv_header_2(ui, "Limit (%)", "Temp (°C)", 66.0);
                            kv_row_2(ui, "Quiet", "pm_quiet_limit/temp (0x2DA)", &mut self.pm_quiet, 66.0);
                            kv_row_2(ui, "Balanced", "pm_balanced_limit/temp (0x2DE)", &mut self.pm_balanced, 66.0);
                            kv_row_2(ui, "Turbo", "pm_turbo_limit/temp (0x2E2)", &mut self.pm_turbo, 66.0);
                            kv_row_2(ui, "Rage", "pm_rage_limit/temp (0x2E6)", &mut self.pm_rage, 66.0);
                        });
                    });
                }
            }
            if show_powermodeacoustic {
                let compact_smu11 = arch.has_compact_power_modes(&self.raw_buffer);
                draw_card(ui, "Power Mode Acoustic", |ui| {
                    ui.add_enabled_ui(!compact_smu11, |ui| {
                        kv_header_2(ui, "Acoustic (RPM)", "Target (°C)", 66.0);
                        kv_row_2(ui, "Quiet", "ac_quiet (0x2F2)", &mut self.ac_quiet, 66.0);
                        kv_row_2(ui, "Balanced", "ac_balanced (0x2F6)", &mut self.ac_balanced, 66.0);
                        kv_row_2(ui, "Turbo", "ac_turbo (0x2FA)", &mut self.ac_turbo, 66.0);
                        kv_row_2(ui, "Rage", "ac_rage (0x2FE)", &mut self.ac_rage, 66.0);
                    });
                });
            }
        });
    }
}
