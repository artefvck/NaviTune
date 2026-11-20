use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_frequency_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let is_rdna3 = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer).is_some();
        let is_rdna2 = !is_rdna3 && AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).is_some();

        let mut issues: Vec<&'static str> = Vec::new();
        if self.freq_gfx[0] > self.freq_gfx[1] {
            issues.push("GFX Min > Max");
        }
        if self.freq_soc[0] > self.freq_soc[1] {
            issues.push("Soc Min > Max");
        }
        if self.freq_fclk[0] > self.freq_fclk[1] {
            issues.push("FCLK Min > Max");
        }
        for i in 1..4 {
            if self.mem_dpm_freq[i][0] > 0 && self.mem_dpm_freq[i][0] < self.mem_dpm_freq[i - 1][0] {
                issues.push("Memory UCLK DPM states not ascending");
                break;
            }
        }
        if self.pcie_gen_speed[0] > self.pcie_gen_speed[1]
            || self.pcie_lane_count[0] > self.pcie_lane_count[1]
            || self.lclk_mhz[0] > self.lclk_mhz[1]
        {
            issues.push("PCIe / LCLK Min > Max");
        }

        ui.horizontal(|ui| {
            if is_rdna3 {
                ui.colored_label(
                    egui::Color32::from_rgb(95, 200, 145),
                    "RDNA 3 Clocks & Fabric (SMU 13.0 — GFX @ 0x5EC, Soc @ 0x60E, FCLK @ 0x62C, UCLK @ 0x63C)",
                );
            } else if is_rdna2 {
                ui.colored_label(
                    egui::Color32::from_rgb(95, 200, 145),
                    "RDNA 2 Clocks & Fabric (SMU 11.0 — Full Display/Media, FCLK, UCLK & PCIe/LCLK)",
                );
            } else {
                ui.weak("Load a PowerPlay table to verify clock domains.");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.raw_buffer.is_empty() {
                    if issues.is_empty() {
                        ui.colored_label(
                            egui::Color32::from_rgb(80, 210, 120),
                            "✔ Clock Domains Verified",
                        );
                    } else {
                        if ui.button("Fix Inverted Bounds").clicked() {
                            if self.freq_gfx[0] > self.freq_gfx[1] {
                                self.freq_gfx[0] = self.freq_gfx[1];
                            }
                            if self.freq_soc[0] > self.freq_soc[1] {
                                self.freq_soc[0] = self.freq_soc[1];
                            }
                            if self.freq_fclk[0] > self.freq_fclk[1] {
                                self.freq_fclk[0] = self.freq_fclk[1];
                            }
                            if self.pcie_gen_speed[0] > self.pcie_gen_speed[1] {
                                self.pcie_gen_speed[0] = self.pcie_gen_speed[1];
                            }
                            if self.pcie_lane_count[0] > self.pcie_lane_count[1] {
                                self.pcie_lane_count[0] = self.pcie_lane_count[1];
                            }
                            if self.lclk_mhz[0] > self.lclk_mhz[1] {
                                self.lclk_mhz[0] = self.lclk_mhz[1];
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

        two_col_cards(ui, "freq_cols", |ui| {
            draw_card(ui, "Clock Limits (MHz)", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "GFX", "freq_gfx (0x52E / 0x5EC)", &mut self.freq_gfx, 62.0);
                kv_row_2(ui, "Soc", "freq_soc (0x56E / 0x60E)", &mut self.freq_soc, 62.0);
                kv_row_2(ui, "FCLK", "freq_fclk (0x5D6 / 0x62C)", &mut self.freq_fclk, 62.0);
                if is_rdna2 || !is_rdna3 {
                    ui.add_space(4.0);
                    for (lbl, tag, pair) in [
                        ("VCLK", "freq_vclk", &mut self.freq_vclk),
                        ("DCLK", "freq_dclk", &mut self.freq_dclk),
                        ("DCEFCLK", "freq_dcef", &mut self.freq_dcefclk),
                        ("DISPCLK", "freq_disp", &mut self.freq_dispclk),
                        ("PIXCLK", "freq_pix", &mut self.freq_pixclk),
                        ("PHYCLK", "freq_phy", &mut self.freq_phyclk),
                        ("DTBCLK", "freq_dtb", &mut self.freq_dtbclk),
                    ] {
                        kv_row_2(ui, lbl, tag, pair, 62.0);
                    }
                    kv_row_1(ui, "FCLK Boost", "fclk_boost", &mut self.fclk_boost_freq, 62.0);
                }
            });
        }, |ui| {
            draw_card(ui, "Memory DPM (MHz / Div)", |ui| {
                kv_header_2(ui, "Freq", "Div", 62.0);
                for i in 0..4 {
                    kv_row_2(ui, &format!("DPM {}", i), &format!("freq_uclk[{}] (0x57E / 0x63C)", i), &mut self.mem_dpm_freq[i], 62.0);
                }
            });
            draw_card(ui, "PCIe & LCLK", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "PCIe Gen", "pcie_gen_speed (0x6F6 / 0x810)", &mut self.pcie_gen_speed, 62.0);
                kv_row_2(ui, "PCIe Lanes", "pcie_lane_count (0x6F8 / 0x813)", &mut self.pcie_lane_count, 62.0);
                kv_row_2(ui, "LCLK (MHz)", "lclk_freq (0x6FA / 0x816)", &mut self.lclk_mhz, 62.0);
            });
        });
    }
}
