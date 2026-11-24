use eframe::egui;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_power_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let is_rdna3 = arch.is_rdna3();
        let _is_rdna2 = arch.is_rdna2();
        let is_rdna1 = arch.is_rdna1() || arch.is_vega() || arch.is_gcn_subtable();
        if let Some(hint) = arch.gcn_subtable_hint("power") {
            ui.colored_label(egui::Color32::from_rgb(235, 190, 95), hint);
            ui.add_space(6.0);
        }
        let mut issues: Vec<&str> = Vec::new();
        if self.volt_gfx[1] > 0 && self.volt_gfx[0] > self.volt_gfx[1] { issues.push("GFX Vmin > Vmax"); }
        if self.volt_soc[0] > self.volt_soc[1] { issues.push("Soc Vmin > Vmax"); }
        if arch.is_gcn_subtable() {
            if self.volt_gfx[1] <= 0 || self.volt_gfx[1] > 1350 { issues.push("HardLimit VDDC Max out of 1..1350 mV"); }
        } else {
            if self.power_limit_w <= 0 || self.power_limit_w > 600 { issues.push("Power Limit (W) out of 1..600 W"); }
            if self.tdc_gfx <= 0 || self.tdc_soc <= 0 { issues.push("TDC GFX/Soc <= 0 A"); }
        }
        let dpm_range = if arch.is_gcn_subtable() { 3..4 } else { 0..4 };
        for i in dpm_range {
            if self.mem_dpm_volt[i][0] > self.mem_dpm_volt[i][1] { issues.push("Memory VDDCI > MVDD"); break; }
        }
        if !issues.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Power Check: {}", issues.join(" | ")));
            ui.add_space(4.0);
        }

        let (_vg_tag, _vs_tag, ppt_tag, tdcg_tag, tdcs_tag, _memv_tag) = match arch {
            crate::ppt_arch::PptArch::Vega20 => (
                "vmin/vmax_gfx (0x159 / 0x15D)",
                "vmin/vmax_soc (0x15B / 0x15F)",
                "socket_power_limit_ac (0x11D)",
                "tdc_limit_gfx (0x13D)",
                "tdc_limit_soc (0x13F)",
                "vddci_max / mvdd_max (0x145 / 0x147)",
            ),
            crate::ppt_arch::PptArch::Rdna1 => (
                "vmin/vmax_gfx (0x0F3 / 0x0F7)",
                "vmin/vmax_soc (0x0F5 / 0x0F9)",
                "socket_power_limit_ac (0x0B7)",
                "tdc_limit_gfx (0x0D7)",
                "tdc_limit_soc (0x0D9)",
                if arch == crate::ppt_arch::PptArch::Vega20 { "vddci_max / mvdd_max (0x145 / 0x147)" } else { "vddci_max / mvdd_max (0x0DF / 0x0E1)" },
            ),
            _ => (
                "vmin/vmax_gfx (0x3A6 / 0x3B4)",
                "vmin/vmax_soc (0x3AA / 0x3B8)",
                "socket_power_limit_ac (0x32E / 0x350)",
                "tdc_limit_gfx (0x34E / 0x366)",
                "tdc_limit_soc (0x350 / 0x368)",
                "vddci_max / mvdd_max (0x0DF / 0x0E1)",
            ),
        };
        let show_corevoltage = self.is_card_usable(crate::ppt_arch::PptCard::CoreVoltage);
        let show_memoryvoltage = self.is_card_usable(crate::ppt_arch::PptCard::MemoryVoltage);
        let show_powerandcurrent = self.is_card_usable(crate::ppt_arch::PptCard::PowerAndCurrent);
        let show_thermalvmin = self.is_card_usable(crate::ppt_arch::PptCard::ThermalVmin);
        let show_ulvvoltage = self.is_card_usable(crate::ppt_arch::PptCard::UlvVoltage);
        if !show_corevoltage && !show_memoryvoltage && !show_powerandcurrent && !show_thermalvmin && !show_ulvvoltage {
            crate::card_layout::draw_arch_note_card(ui, &format!("{} — Power & Voltage Architecture", arch.label()), "This silicon revision stores power & voltage limits inside its DPM dependency tables (Frequency tab) rather than standalone HardLimit or SMU 11/13 voltage cards.");
            return;
        }
        let pwr_left_vis = show_corevoltage || show_powerandcurrent || show_ulvvoltage;
        let pwr_right_vis = (!arch.is_gcn_subtable() && show_memoryvoltage) || show_thermalvmin;
        two_col_cards_adaptive(ui, "pwr_cols", pwr_left_vis, pwr_right_vis, |ui| {
            if show_corevoltage {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::CoreVoltage), |ui| {
                    let (gfx_vtag, soc_vtag) = arch.core_voltage_offset_labels();
                    if arch.has_hard_limit_subtable() {
                        kv_row_1(ui, "VDDC Max (HardLimit)", gfx_vtag, &mut self.volt_gfx[1], 62.0);
                    } else {
                        kv_header_2(ui, "Vmin", "Vmax", 62.0);
                        kv_row_2(ui, "GFX", gfx_vtag, &mut self.volt_gfx, 62.0);
                        kv_row_2(ui, "Soc", soc_vtag, &mut self.volt_soc, 62.0);
                    }
                });
            }
            if show_powerandcurrent {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::PowerAndCurrent), |ui| {
                    kv_row_1(ui, "Power Limit (W)", ppt_tag, &mut self.power_limit_w, 62.0);
                    kv_row_1(ui, "TDC GFX (A)", tdcg_tag, &mut self.tdc_gfx, 62.0);
                    if !arch.is_gcn_subtable() {
                        kv_row_1(ui, "TDC Soc (A)", tdcs_tag, &mut self.tdc_soc, 62.0);
                    }
                });
            }
            if show_ulvvoltage {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::UlvVoltage), |ui| {
                    if is_rdna3 {
                        kv_row_1(ui, "GFX Vmin", "ulv_voltage_vmin_gfx (0x3C4)", &mut self.ulv_gfx[0], 62.0);
                        kv_row_1(ui, "Soc Vmin", "ulv_voltage_vmin_soc (0x3C6)", &mut self.ulv_soc[0], 62.0);
                    } else {
                        kv_header_2(ui, "Vmin", "Vmax", 62.0);
                        kv_row_2(ui, "GFX", match arch { crate::ppt_arch::PptArch::Vega20 => "min_voltage_ulv_gfx / ulv_voltage_offset_gfx (0x137 / 0x131)", crate::ppt_arch::PptArch::Rdna1 => "min_voltage_ulv_gfx / ulv_voltage_offset_gfx (0x0D1 / 0x0CB)", _ => "ulv_voltage_vmin_gfx / vmax_gfx (0x380 / 0x384)" }, &mut self.ulv_gfx, 62.0);
                        kv_row_2(ui, "Soc", match arch { crate::ppt_arch::PptArch::Vega20 => "min_voltage_ulv_soc / ulv_voltage_offset_soc (0x139 / 0x12F)", crate::ppt_arch::PptArch::Rdna1 => "min_voltage_ulv_soc / ulv_voltage_offset_soc (0x0D3 / 0x0C9)", _ => "ulv_voltage_vmin_soc / vmax_soc (0x382 / 0x386)" }, &mut self.ulv_soc, 62.0);
                    }
                });
            }
        }, |ui| {
            if !arch.is_gcn_subtable()
            && show_memoryvoltage {
                    draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::MemoryVoltage), |ui| {
                        kv_header_2(ui, "VDDCI", "MVDD", 62.0);
                        if is_rdna1 {
                            kv_row_2(ui, "Max (VDDCI/MVDD)", "vddci_max / mvdd_max (0x0DF / 0x0E1)", &mut self.mem_dpm_volt[3], 62.0);
                            for i in 0..3 {
                                self.mem_dpm_volt[i] = self.mem_dpm_volt[3];
                            }
                        } else {
                            for (i, volt) in self.mem_dpm_volt.iter_mut().enumerate().take(4) {
                                kv_row_2(ui, &format!("DPM {}", i), &format!("vddci/mvdd_{} (0x66A / 0x7D4)", i), volt, 62.0);
                            }
                        }
                    });
            }
            if show_thermalvmin {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::ThermalVmin), |ui| {
                    kv_header_2(ui, "TVmin (°C)", "Hyst (°C)", 62.0);
                    kv_row_2(ui, "GFX", "tvmin/hyst_gfx (0x3B2)", &mut self.tvmin_gfx, 62.0);
                    kv_row_2(ui, "Soc", "tvmin/hyst_soc (0x3B4)", &mut self.tvmin_soc, 62.0);
                    ui.add_space(4.0);
                    kv_header_2(ui, "Low", "High", 62.0);
                    kv_row_2(ui, "GFX", "vmin_low/high_gfx (0x3B6)", &mut self.vmin_lh_gfx, 62.0);
                    kv_row_2(ui, "Soc", "vmin_low/high_soc (0x3BA)", &mut self.vmin_lh_soc, 62.0);
                });
            }
        });

    }
}

