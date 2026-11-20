use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_power_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let is_rdna3 = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer).is_some();
        let is_rdna2 = !is_rdna3 && AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).is_some();

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

        ui.horizontal(|ui| {
            if is_rdna3 {
                ui.colored_label(
                    egui::Color32::from_rgb(95, 200, 145),
                    "RDNA 3 Power & Voltage (SMU 13.0 @ 0x340 — PPT, TDC, Vmin/Vmax & VDDCI/MVDD)",
                );
            } else if is_rdna2 {
                ui.colored_label(
                    egui::Color32::from_rgb(95, 200, 145),
                    "RDNA 2 Power & Voltage (SMU 11.0 @ 0x322 — Full Voltage, PPT, TDC & TVmin)",
                );
            } else {
                ui.weak("Load a PowerPlay table to verify Power & Voltage rails.");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.raw_buffer.is_empty() {
                    if issues.is_empty() {
                        ui.colored_label(
                            egui::Color32::from_rgb(80, 210, 120),
                            "✔ Power & Voltage Rails Verified",
                        );
                    } else {
                        if ui.button("Fix Inverted Bounds").clicked() {
                            if self.volt_gfx[0] > self.volt_gfx[1] {
                                self.volt_gfx[0] = self.volt_gfx[1];
                            }
                            if self.volt_soc[0] > self.volt_soc[1] {
                                self.volt_soc[0] = self.volt_soc[1];
                            }
                            for i in 0..4 {
                                if self.mem_dpm_volt[i][0] > self.mem_dpm_volt[i][1] {
                                    self.mem_dpm_volt[i][0] = self.mem_dpm_volt[i][1];
                                }
                            }
                        }
                        ui.colored_label(
                            egui::Color32::from_rgb(245, 185, 65),
                            format!("⚠ {}", issues.join(" | ")),
                        );
                    }
                }
            });
        });
        ui.add_space(4.0);

        two_col_cards(ui, "pwr_cols", |ui| {
            draw_card(ui, "Voltage Vmin / Vmax (mV)", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "GFX", "vmin/vmax_gfx (0x3A6 / 0x3B4)", &mut self.volt_gfx, 62.0);
                kv_row_2(ui, "Soc", "vmin/vmax_soc (0x3AA / 0x3B8)", &mut self.volt_soc, 62.0);
            });
            draw_card(ui, "Power & TDC Limits", |ui| {
                kv_row_1(ui, "Power Limit (W)", "socket_power_limit_ac (0x32E / 0x350)", &mut self.power_limit_w, 62.0);
                kv_row_1(ui, "TDC GFX (A)", "tdc_limit_gfx (0x34E / 0x366)", &mut self.tdc_gfx, 62.0);
                kv_row_1(ui, "TDC Soc (A)", "tdc_limit_soc (0x350 / 0x368)", &mut self.tdc_soc, 62.0);
            });
            draw_card(ui, "ULV Vmin / Offset (mV)", |ui| {
                kv_header_2(ui, "Vmin", "Offset", 62.0);
                kv_row_2(ui, "GFX", "ulv_voltage_vmin_gfx (0x380 / 0x3C4)", &mut self.ulv_gfx, 62.0);
                kv_row_2(ui, "Soc", "ulv_voltage_vmin_soc (0x382 / 0x3C6)", &mut self.ulv_soc, 62.0);
            });
        }, |ui| {
            draw_card(ui, "Memory VDDCI / MVDD (mV)", |ui| {
                kv_header_2(ui, "VDDCI", "MVDD", 62.0);
                for i in 0..4 {
                    kv_row_2(ui, &format!("DPM {}", i), &format!("vddci/mvdd_{} (0x66A / 0x7D4)", i), &mut self.mem_dpm_volt[i], 62.0);
                }
            });
            if is_rdna2 || !is_rdna3 {
                draw_card(ui, "Temperature Dependent Vmin", |ui| {
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
