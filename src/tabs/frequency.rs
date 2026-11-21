use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_frequency_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let is_rdna3 = arch.is_rdna3();
        if let Some(hint) = arch.gcn_subtable_hint("freq") {
            ui.colored_label(egui::Color32::from_rgb(235, 190, 95), hint);
            ui.add_space(6.0);
        }
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


        two_col_cards(ui, "freq_cols", |ui| {
            draw_card(ui, "Clock Limits", |ui| {
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
            draw_card(ui, "Memory DPM", |ui| {
                if is_rdna3 || is_rdna1 {
                    let off = if is_rdna3 { "0x63C" } else if arch == crate::ppt_arch::PptArch::Vega20 { "0x1B5" } else { "0x14F" };
                    for i in 0..4 {
                        kv_row_1(ui, &format!("DPM {} (MHz)", i), &format!("freq_uclk[{}] ({})", i, off), &mut self.mem_dpm_freq[i][0], 62.0);
                    }
                } else {
                    kv_header_2(ui, "Freq (MHz)", "Div", 62.0);
                    for i in 0..4 {
                        kv_row_2(ui, &format!("DPM {}", i), &format!("freq_uclk[{}] (0x57E)", i), &mut self.mem_dpm_freq[i], 62.0);
                    }
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

