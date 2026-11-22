use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_frequency_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let is_rdna3 = arch.is_rdna3();
        let is_rdna2 = arch.is_rdna2();
        let is_rdna1 = arch.is_rdna1() || arch.is_vega() || arch.is_gcn_subtable();
        if let Some(hint) = arch.gcn_subtable_hint("freq") {
            ui.colored_label(egui::Color32::from_rgb(235, 190, 95), hint);
            ui.add_space(6.0);
        }
        let mut issues: Vec<&'static str> = Vec::new();
        if arch.has_hard_limit_subtable() {
            if self.freq_gfx[1] <= 0 { issues.push("HardLimit GFXCLK Max <= 0 MHz"); }
            if self.mem_dpm_freq[3][0] <= 0 { issues.push("HardLimit MCLK Max <= 0 MHz"); }
        } else {
            if self.freq_gfx[0] > self.freq_gfx[1] { issues.push("GFX Min > Max"); }
            if self.freq_soc[0] > self.freq_soc[1] { issues.push("Soc Min > Max"); }
            if self.freq_fclk[0] > self.freq_fclk[1] { issues.push("FCLK Min > Max"); }
        }
        if arch.has_pcie_and_lclk_limits() {
        for i in 1..4 {
            if self.mem_dpm_freq[i][0] > 0 && self.mem_dpm_freq[i][0] < self.mem_dpm_freq[i - 1][0] {
                issues.push("Memory UCLK DPM states not ascending");
                break;
            }
        }
        }
        if self.pcie_gen_speed[0] > self.pcie_gen_speed[1]
            || self.pcie_lane_count[0] > self.pcie_lane_count[1]
            || self.lclk_mhz[0] > self.lclk_mhz[1]
        {
            issues.push("PCIe / LCLK Min > Max");
        }


        let show_clocklimits = self.is_card_usable(crate::ppt_arch::PptCard::ClockLimits);
        let show_memorydpm = self.is_card_usable(crate::ppt_arch::PptCard::MemoryDpm);
        if !show_clocklimits && !show_memorydpm && !self.is_card_usable(crate::ppt_arch::PptCard::PcieAndLinkClocks) {
            crate::card_layout::draw_arch_note_card(ui, &format!("{} — Clock Architecture", arch.label()), "No standalone clock limit or PCIe link sub-tables are present in this PPTable revision.");
            return;
        }
        two_col_cards_adaptive(ui, "freq_cols", show_clocklimits || show_memorydpm, show_pcieandlinkclocks, |ui| {
            if show_clocklimits {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::ClockLimits), |ui| {
                    if arch.has_hard_limit_subtable() {
                        kv_row_1(ui, "GFXCLK Max (HardLimit)", "hard_limit_table_offset GFXCLK (MHz)", &mut self.freq_gfx[1], 62.0);
                        if arch == crate::ppt_arch::PptArch::Vega10 {
                            kv_row_1(ui, "SOCCLK Max (HardLimit)", "hard_limit_table_offset SOCCLK (MHz)", &mut self.freq_soc[1], 62.0);
                        }
                    } else {
                        let (gfx_ctag, soc_ctag, fclk_ctag) = arch.clock_limit_offset_labels();
                    let _ = (&gfx_ctag, &soc_ctag, &fclk_ctag);
                    kv_header_2(ui, "Min", "Max", 62.0);
                        kv_row_2(ui, "GFX", match arch { crate::ppt_arch::PptArch::Vega20 => "freq_table_gfx (0x185)", crate::ppt_arch::PptArch::Rdna1 => "freq_table_gfx (0x11F)", _ => "freq_gfx (0x52E / 0x5EC)" }, &mut self.freq_gfx, 62.0);
                        kv_row_2(ui, "Soc", match arch { crate::ppt_arch::PptArch::Vega20 => "freq_table_socclk (0x195)", crate::ppt_arch::PptArch::Rdna1 => "freq_table_socclk (0x12F)", _ => "freq_soc (0x56E / 0x60E)" }, &mut self.freq_soc, 62.0);
                        kv_row_2(ui, "FCLK", match arch { crate::ppt_arch::PptArch::Vega20 => "freq_table_fclk (0x1C5)", crate::ppt_arch::PptArch::Rdna1 => "freq_table_fclk (0x15F)", _ => "freq_fclk (0x5D6 / 0x62C)" }, &mut self.freq_fclk, 62.0);
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
                    }
                });
            }
        }, |ui| {
            if show_memorydpm {
                draw_card(ui, arch.card_title(crate::ppt_arch::PptCard::MemoryDpm), |ui| {
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
            }
            if show_pcieandlinkclocks {
                draw_card(ui, "PCIe & LCLK", |ui| {
                    kv_header_2(ui, "Min", "Max", 62.0);
                    kv_row_2(ui, "PCIe Gen", "pcie_gen_speed (0x6F6 / 0x810)", &mut self.pcie_gen_speed, 62.0);
                    kv_row_2(ui, "PCIe Lanes", "pcie_lane_count (0x6F8 / 0x813)", &mut self.pcie_lane_count, 62.0);
                    kv_row_2(ui, "LCLK (MHz)", "lclk_freq (0x6FA / 0x816)", &mut self.lclk_mhz, 62.0);
                });
            }
        });

    }
}

