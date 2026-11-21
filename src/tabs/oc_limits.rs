use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_oc_limits_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::pptable_rdna1::PptArch::detect(&self.raw_buffer);
        let is_rdna3 = arch == crate::pptable_rdna1::PptArch::Rdna3;
        let is_rdna2 = arch == crate::pptable_rdna1::PptArch::Rdna2;
        let is_rdna1 = arch == crate::pptable_rdna1::PptArch::Rdna1;

        let mut issues: Vec<&str> = Vec::new();
        if !is_rdna3 {
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

        two_col_cards(ui, "oc_cols", |ui| {
            draw_card(ui, "OverDrive", |ui| {
                if is_rdna3 {
                    kv_row_1(ui, "GFX Clock Max (MHz)", "od_gfx_max (0x0FF)", &mut self.oc_gfx_clock[1], 66.0);
                    kv_row_1(ui, "Memory Clock Max (MHz)", "od_mem_max (0x107)", &mut self.oc_mem_clock[1], 66.0);
                    kv_row_1(ui, "Power Limit Max (%)", "od_pwr_max (0x10F)", &mut self.oc_power_limit[1], 66.0);
                    kv_row_1(ui, "Fan Speed Max (RPM)", "od_fan_max (0x113)", &mut self.oc_fan_rpm[1], 66.0);
                    kv_row_1(ui, "Fan Acoustic Max (RPM)", "od_acoustic_max (0x117)", &mut self.oc_fan_acoustic_rpm[1], 66.0);
                } else {
                    kv_header_2(ui, "Min", "Max", 66.0);
                    kv_row_2(ui, "GFX Clock (MHz)", "od_gfx_min/max (0x0E2)", &mut self.oc_gfx_clock, 66.0);
                    kv_row_2(ui, "Memory Clock (MHz)", "od_mem_min/max (0x0FA)", &mut self.oc_mem_clock, 66.0);
                    kv_row_2(ui, "Power Limit (%)", "od_pwr_min/max (0x102)", &mut self.oc_power_limit, 66.0);
                    kv_row_2(ui, "Fan Speed (RPM)", "od_fan_min/max (0x106)", &mut self.oc_fan_rpm, 66.0);
                    kv_row_2(ui, "Fan Acoustic (RPM)", "od_acoustic_min/max (0x10A)", &mut self.oc_fan_acoustic_rpm, 66.0);
                    if is_rdna2 {
                        ui.add_space(4.0);
                        kv_row_1(ui, "Mem Timing Ctrl", "od_mem_timing_ctrl (0x116)", &mut self.oc_mem_timing_ctrl, 66.0);
                        kv_row_1(ui, "Power Mode Ctrl", "od_pwr_mode_ctrl (0x11A)", &mut self.oc_power_mode_ctrl, 66.0);
                        kv_row_1(ui, "Zero RPM Ctrl", "od_zero_rpm_ctrl (0x11E)", &mut self.oc_zero_rpm_ctrl, 66.0);
                    }
                }
            });
        }, |ui| {
            if is_rdna3 {
                draw_card(ui, "Power Modes", |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 175, 195),
                        "RDNA 3 (SMU 13.0) manages Quiet, Balanced, and Rage presets dynamically in SMU firmware; static PPTable slots are unused.",
                    );
                });
            } else {
                draw_card(ui, "Power Modes", |ui| {
                    kv_header_2(ui, "Limit (%)", "Temp (°C)", 66.0);
                    kv_row_2(ui, "Quiet", "pm_quiet_limit/temp (0x2DA)", &mut self.pm_quiet, 66.0);
                    kv_row_2(ui, "Balanced", "pm_balanced_limit/temp (0x2DE)", &mut self.pm_balanced, 66.0);
                    kv_row_2(ui, "Turbo", "pm_turbo_limit/temp (0x2E2)", &mut self.pm_turbo, 66.0);
                    kv_row_2(ui, "Rage", "pm_rage_limit/temp (0x2E6)", &mut self.pm_rage, 66.0);
                });
                draw_card(ui, "Power Mode Acoustic", |ui| {
                    kv_header_2(ui, "Acoustic (RPM)", "Target (°C)", 66.0);
                    kv_row_2(ui, "Quiet", "ac_quiet (0x2F2)", &mut self.ac_quiet, 66.0);
                    kv_row_2(ui, "Balanced", "ac_balanced (0x2F6)", &mut self.ac_balanced, 66.0);
                    kv_row_2(ui, "Turbo", "ac_turbo (0x2FA)", &mut self.ac_turbo, 66.0);
                    kv_row_2(ui, "Rage", "ac_rage (0x2FE)", &mut self.ac_rage, 66.0);
                });
            }
        });
    }
}
