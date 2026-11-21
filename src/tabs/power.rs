use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_power_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let is_rdna3 = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer).is_some();
        
        let mut issues: Vec<&'static str> = Vec::new();
        if self.volt_gfx > self.volt_gfx {
            issues.push("GFX Vmin > Vmax");
        }
        if self.volt_soc > self.volt_soc {
            issues.push("Soc Vmin > Vmax");
        }
        if self.power_limit_w <= 0 || self.power_limit_w > 600 {
            issues.push("Power Limit (W) out of range");
        }
        if self.tdc_gfx <= 0 || self.tdc_soc <= 0 {
            issues.push("TDC GFX/Soc <= 0 A");
        }
        for i in 0..4 {
            if self.mem_dpm_volt[i] > self.mem_dpm_volt[i] {
                issues.push("Memory VDDCI > MVDD");
                break;
            }
        }


        two_col_cards(ui, "pwr_cols", |ui| {
            draw_card(ui, "Core Voltage (mV)", |ui| {
                kv_header_2(ui, "Vmin", "Vmax", 62.0);
                kv_row_2(ui, "GFX", "vmin/vmax_gfx (0x3A6 / 0x3B4)", &mut self.volt_gfx, 62.0);
                kv_row_2(ui, "Soc", "vmin/vmax_soc (0x3AA / 0x3B8)", &mut self.volt_soc, 62.0);
            });
            draw_card(ui, "Power & Current", |ui| {
                kv_row_1(ui, "Power Limit (W)", "socket_power_limit_ac (0x32E / 0x350)", &mut self.power_limit_w, 62.0);
                kv_row_1(ui, "TDC GFX (A)", "tdc_limit_gfx (0x34E / 0x366)", &mut self.tdc_gfx, 62.0);
                kv_row_1(ui, "TDC Soc (A)", "tdc_limit_soc (0x350 / 0x368)", &mut self.tdc_soc, 62.0);
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
            draw_card(ui, "Memory Voltage (mV)", |ui| {
                kv_header_2(ui, "VDDCI", "MVDD", 62.0);
                for i in 0..4 {
                    kv_row_2(ui, &format!("DPM {}", i), &format!("vddci/mvdd_{} (0x66A / 0x7D4)", i), &mut self.mem_dpm_volt[i], 62.0);
                }
            });
            if !is_rdna3 {
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

