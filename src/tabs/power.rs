use eframe::egui;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_power_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let is_rdna3 = arch.is_rdna3();
        let is_rdna2 = arch.is_rdna2();
        let is_rdna1 = arch.is_rdna1() || arch.is_vega() || arch.is_gcn_subtable();
        if let Some(hint) = arch.gcn_subtable_hint("power") {
            ui.colored_label(egui::Color32::from_rgb(235, 190, 95), hint);
            ui.add_space(6.0);
        }
        let mut issues: Vec<&str> = Vec::new();
        if self.volt_gfx[0] > self.volt_gfx[1] { issues.push("GFX Vmin > Vmax"); }
        if self.volt_soc[0] > self.volt_soc[1] { issues.push("Soc Vmin > Vmax"); }
        if !arch.is_gcn_subtable() {
            if self.power_limit_w <= 0 || self.power_limit_w > 600 { issues.push("Power Limit (W) out of 1..600 W"); }
            if self.tdc_gfx <= 0 || self.tdc_soc <= 0 { issues.push("TDC GFX/Soc <= 0 A"); }
        }
        for i in 0..4 {
            if self.mem_dpm_volt[i][0] > self.mem_dpm_volt[i][1] { issues.push("Memory VDDCI > MVDD"); break; }
        }
        if !issues.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Power Check: {}", issues.join(" | ")));
            ui.add_space(4.0);
        }

        let (vg_tag, vs_tag, ppt_tag, tdcg_tag, tdcs_tag, memv_tag) = match arch {
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
        two_col_cards(ui, "pwr_cols", |ui| {
            draw_card(ui, "Core Voltage (mV)", |ui| {
                kv_header_2(ui, "Vmin", "Vmax", 62.0);
                kv_row_2(ui, "GFX", vg_tag, &mut self.volt_gfx, 62.0);
                kv_row_2(ui, "Soc", vs_tag, &mut self.volt_soc, 62.0);
            });
            draw_card(ui, "Power & Current", |ui| {
                kv_row_1(ui, "Power Limit (W)", ppt_tag, &mut self.power_limit_w, 62.0);
                kv_row_1(ui, "TDC GFX (A)", tdcg_tag, &mut self.tdc_gfx, 62.0);
                kv_row_1(ui, "TDC Soc (A)", tdcs_tag, &mut self.tdc_soc, 62.0);
            });
            draw_card(ui, "ULV Voltage (mV)", |ui| {
                if is_rdna3 {
                    kv_row_1(ui, "GFX Vmin", "ulv_voltage_vmin_gfx (0x3C4)", &mut self.ulv_gfx[0], 62.0);
                    kv_row_1(ui, "Soc Vmin", "ulv_voltage_vmin_soc (0x3C6)", &mut self.ulv_soc[0], 62.0);
                } else {
                    kv_header_2(ui, "Vmin", "Vmax", 62.0);
                    kv_row_2(ui, "GFX", "ulv_voltage_vmin_gfx (0x380)", &mut self.ulv_gfx, 62.0);
                    kv_row_2(ui, "Soc", "ulv_voltage_vmin_soc (0x382)", &mut self.ulv_soc, 62.0);
                }
            });
        }, |ui| {
            if !arch.is_gcn_subtable() {
                draw_card(ui, "Memory Voltage (mV)", |ui| {
                    kv_header_2(ui, "VDDCI", "MVDD", 62.0);
                    if is_rdna1 {
                        kv_row_2(ui, "Max (VDDCI/MVDD)", "vddci_max / mvdd_max (0x0DF / 0x0E1)", &mut self.mem_dpm_volt[3], 62.0);
                        for i in 0..3 {
                            self.mem_dpm_volt[i] = self.mem_dpm_volt[3];
                        }
                    } else {
                        for i in 0..4 {
                            kv_row_2(ui, &format!("DPM {}", i), &format!("vddci/mvdd_{} (0x66A / 0x7D4)", i), &mut self.mem_dpm_volt[i], 62.0);
                        }
                    }
    
            }
        });
            if !is_rdna3 && !is_rdna1 {
                draw_card(ui, "Thermal Vmin", |ui| {
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

