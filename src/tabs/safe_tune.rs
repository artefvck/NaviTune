use eframe::egui;
use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, NaviRDNA2Chip};
use crate::pptable_rdna3::{AtomSmcDpmInfoV4_12, NaviRDNA3Chip};
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_safe_tune_tab(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No table loaded");
            });
            return;
        }

        let did = self
            .vbios_buffer
            .as_deref()
            .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
            .map(|vi| vi.device_id)
            .unwrap_or(0);

        let (
            chip_label, is_rdna3, max_gfx_v, max_soc_v, min_w, max_w,
            max_tdc_gfx, max_tdc_soc, safe_uclk_max, max_fan_tgt, min_fclk, min_soc, expected_lanes,
        ) = if let Some(t3) = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
            let chip3 = t3.detect_chip(did);
            match chip3 {
                NaviRDNA3Chip::Navi31PlumBonito => (chip3.display_name(), true, 1200, 1325, 260, 365, 460, 90, 1300, 105, 1400, 2200, 16),
                NaviRDNA3Chip::Navi32WheatNas => (chip3.display_name(), true, 1200, 1325, 200, 275, 340, 75, 1350, 105, 1400, 2000, 16),
                NaviRDNA3Chip::Navi33Hotbeige => (chip3.display_name(), true, 1200, 1325, 115, 200, 240, 60, 1150, 105, 1200, 1800, 8),
                NaviRDNA3Chip::UnknownRdna3 => (chip3.display_name(), true, 1200, 1325, 140, 365, 460, 90, 1300, 105, 1400, 1800, 16),
            }
        } else if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Rdna1 {
            let chip1 = if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&self.raw_buffer) {
                t1.detect_chip(did)
            } else if did != 0 {
                crate::pptable_rdna1::NaviRDNA1Chip::from_did_and_clock(did, self.oc_gfx_clock[1] as u32)
            } else if self.power_limit_w > 0 && self.power_limit_w <= 140 {
                crate::pptable_rdna1::NaviRDNA1Chip::Navi14
            } else {
                crate::pptable_rdna1::NaviRDNA1Chip::from_did_and_clock(did, self.oc_gfx_clock[1] as u32)
            };
            match chip1 {
                crate::pptable_rdna1::NaviRDNA1Chip::Navi10 => (chip1.display_name(), false, 1200, 1200, 150, 225, 210, 32, 950, 95, 1250, 1150, 16),
                crate::pptable_rdna1::NaviRDNA1Chip::Navi12 => (chip1.display_name(), false, 1150, 1150, 85, 150, 140, 28, 900, 95, 1400, 1100, 16),
                crate::pptable_rdna1::NaviRDNA1Chip::Navi14 => (chip1.display_name(), false, 1150, 1150, 50, 130, 125, 24, 900, 95, 1400, 1100, 8),
                crate::pptable_rdna1::NaviRDNA1Chip::UnknownRdna1 => (chip1.display_name(), false, 1200, 1200, 50, 225, 210, 32, 950, 95, 1400, 1100, 16),
            }
        } else if matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega10 | crate::ppt_arch::PptArch::Vega20) {
            let frev_v = self.raw_buffer.get(2).copied().unwrap_or(0);
            let vchip = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev_v);
            match vchip {
                crate::pptable_vega::VegaChip::Vega20 => (vchip.display_name(), false, 1150, 1250, 140, 320, 280, 180, 1150, 95, 1100, 1000, 16),
                _ => (vchip.display_name(), false, 1150, 1150, 120, 300, 260, 40, 1100, 90, 0, 0, 16),
            }
        } else if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Polaris {
            let pchip = crate::pptable_polaris::PolarisChip::from_did(did);
            match pchip {
                crate::pptable_polaris::PolarisChip::Baffin => (pchip.display_name(), false, 1150, 1000, 55, 135, 110, 25, 1750, 85, 0, 0, 16),
                crate::pptable_polaris::PolarisChip::Lexa => (pchip.display_name(), false, 1100, 950, 35, 95, 80, 20, 1500, 85, 0, 0, 16),
                _ => (pchip.display_name(), false, 1200, 1050, 120, 220, 180, 35, 2250, 85, 0, 0, 16),
            }
        } else {
            let chip2 = AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer)
                .map(|t| t.detect_chip(&self.raw_buffer))
                .unwrap_or(NaviRDNA2Chip::Navi23DimgreyCavefish);
            match chip2 {
                NaviRDNA2Chip::Navi24BeigeGoby => (chip2.display_name(), false, 1150, 1100, 25, 115, 95, 22, 1050, 95, 1350, 1000, 4),
                NaviRDNA2Chip::Navi23DimgreyCavefish => (chip2.display_name(), false, 1150, 1150, 80, 145, 135, 28, 1000, 95, 1550, 1150, 8),
                NaviRDNA2Chip::Navi22NavyFlounder => (chip2.display_name(), false, 1200, 1200, 175, 230, 186, 32, 1000, 95, 1550, 1150, 16),
                NaviRDNA2Chip::Navi21SiennaCichlid => (chip2.display_name(), false, 1200, 1200, 250, 300, 255, 40, 1000, 95, 1550, 1150, 16),
            }
        };

        let _is_gcn = crate::ppt_arch::PptArch::detect(&self.raw_buffer).is_gcn_subtable();
        let mut checks: Vec<(&str, &str, String, String, u8, &str)> = Vec::new();

        let gfx_ok = self.freq_gfx[0] > 0 && self.freq_gfx[0] < self.freq_gfx[1] && (self.oc_gfx_clock[1] == 0 || self.freq_gfx[1] <= self.oc_gfx_clock[1]);
        let gfx_sev = if self.freq_gfx[0] >= self.freq_gfx[1] { 2 } else if !gfx_ok { 1 } else { 0 };
        checks.push(("Clocks", "GFX Min / Max vs OD8", format!("{} / {} MHz (OD Max: {})", self.freq_gfx[0], self.freq_gfx[1], self.oc_gfx_clock[1]), "Min < Max <= OD Max".to_string(), gfx_sev, "Keep GFX Max within OverDrive Max to prevent driver clock clamping."));

        let uclk_dpm3 = self.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
        let uclk_sev = if uclk_dpm3 > safe_uclk_max + 50 { 2 } else if uclk_dpm3 > safe_uclk_max || (self.oc_mem_clock[1] > 0 && uclk_dpm3 > self.oc_mem_clock[1]) { 1 } else { 0 };
        checks.push(("Clocks", "UCLK DPM3 vs Safe Max", format!("{} MHz (OD Max: {} MHz)", uclk_dpm3, self.oc_mem_clock[1]), format!("<= {} MHz (OD <= {})", safe_uclk_max, self.oc_mem_clock[1]), uclk_sev, "GDDR6 strap timing loosens or throws ECC retries if UCLK exceeds silicon ceiling."));

        let is_gcn = crate::ppt_arch::PptArch::detect(&self.raw_buffer).is_gcn_subtable();
        let eff_vmax_gfx = (self.volt_gfx[1] as f32 + self.static_off_c[0] as f32 * 1000.0).round() as i32;
        let v_gfx_sev = if self.volt_gfx[1] > 0 && (self.volt_gfx[0] > self.volt_gfx[1] || eff_vmax_gfx > max_gfx_v + 50 || (!is_rdna3 && eff_vmax_gfx < 650)) { 2 } else if self.volt_gfx[0] < 675 || (self.volt_gfx[1] > 0 && eff_vmax_gfx > max_gfx_v) { 1 } else { 0 };
        checks.push(("Voltage", "GFX Vmin / Vmax (+Offset)", format!("{} / {} mV (Eff Max: {} mV)", self.volt_gfx[0], self.volt_gfx[1], eff_vmax_gfx), format!("700..{} mV (Min <= Max)", max_gfx_v), v_gfx_sev, "Effective Vmax includes Static Voltage Offset (c); keep within silicon safe bounds."));

        if !is_gcn && (self.volt_soc[0] > 0 || self.volt_soc[1] > 0) {
        let v_soc_sev = if self.volt_soc[0] > self.volt_soc[1] || self.volt_soc[1] > max_soc_v + 50 { 2 } else if self.volt_soc[0] < 675 || self.volt_soc[1] > max_soc_v { 1 } else { 0 };
            checks.push(("Voltage", "Soc Vmin / Vmax", format!("{} / {} mV", self.volt_soc[0], self.volt_soc[1]), format!("700..{} mV (Min <= Max)", max_soc_v), v_soc_sev, "Keep SoC voltage within silicon spec for Infinity Cache & memory controller stability."));
        }

        let pwr_sev = if self.power_limit_w <= 0 || self.tdc_gfx <= 0 || self.power_limit_w > max_w + 50 { 2 } else if self.power_limit_w < min_w || self.power_limit_w > max_w || self.tdc_gfx > max_tdc_gfx || self.tdc_soc > max_tdc_soc { 1 } else { 0 };
        checks.push(("Power", "PPT & TDC GFX / Soc", format!("{} W | {} A / {} A", self.power_limit_w, self.tdc_gfx, self.tdc_soc), format!("{}..{} W | TDC <= {}/{} A", min_w, max_w, max_tdc_gfx, max_tdc_soc), pwr_sev, "Calibrated to factory silicon VRM current & socket board power limits."));

        let mut mem_v_sev = 0u8;
        for volt in self.mem_dpm_volt.iter().take(4) {
            let vddci = volt[0];
            let mvdd = volt[1];
            if vddci > mvdd || mvdd > 1425 { mem_v_sev = 2; } else if (vddci > 900 || mvdd > 1365) && mem_v_sev < 1 { mem_v_sev = 1; }
        }
        checks.push(("Memory", "VDDCI / MVDD DPM 0..3", format!("DPM3: {} / {} mV", self.mem_dpm_volt.iter().map(|d| d[0]).max().unwrap_or(0), self.mem_dpm_volt.iter().map(|d| d[1]).max().unwrap_or(0)), "VDDCI <= MVDD <= 1365 mV".to_string(), mem_v_sev, "Factory DPM3 is 800..850 mV VDDCI / 1350 mV MVDD; keep MVDD <= 1365 mV on air."));

        let edge_lim = self.temp_limits[0];
        let hotspot_lim = self.temp_limits[1];
        let mem_lim = self.temp_limits[2];
        let has_zrpm = self.fan_stop_temp > 0 || self.fan_start_temp > 0;
        let rpm_inv = self.fan_max_rpm > 0 && (self.fan_throttling_rpm > self.fan_max_rpm || self.fan_acoustic_limit > self.fan_max_rpm);
        let rpm_warn = self.fan_throttling_rpm > 0 && self.fan_acoustic_limit > self.fan_throttling_rpm;
        let (safe_edge, safe_hot, safe_mem) = if is_rdna3 { (110, 110, 108) } else { (100, 110, 105) };
        let temp_risk = edge_lim > safe_edge + 5 || hotspot_lim > safe_hot + 5 || mem_lim > safe_mem + 5;
        let temp_warn = edge_lim > safe_edge || hotspot_lim > safe_hot || mem_lim > safe_mem;
        let fan_sev = if rpm_inv || (has_zrpm && (self.fan_stop_temp >= self.fan_start_temp || (self.fan_target_temp >= 45 && self.fan_start_temp >= self.fan_target_temp))) || temp_risk { 2 } else if rpm_warn || self.fan_target_temp > max_fan_tgt || self.fan_pwm_min < 10 || temp_warn { 1 } else { 0 };
        checks.push(("Thermal", "Fan & TempLimits (Edge/Hot/Mem)", format!("Tgt {}° ({}/{} RPM) | {}/{}/{}°C", self.fan_target_temp, self.fan_throttling_rpm, self.fan_max_rpm, edge_lim, hotspot_lim, mem_lim), format!("Tgt <= {}°, Throttle <= Max RPM | <= {}/{}/{}°C", max_fan_tgt, safe_edge, safe_hot, safe_mem), fan_sev, "Validates Zero-RPM hysteresis, Throttling/Max fan RPM ordering, and TempLimit[0..2] (Edge/Hotspot/Mem)."));

        let is_rdna1 = crate::ppt_arch::PptArch::detect(&self.raw_buffer).is_rdna1();
        let min_od_pwr = if is_rdna1 { -50 } else { -10 };
        let od_pwr_sev = if self.oc_power_limit[0] > self.oc_power_limit[1] || self.oc_power_limit[1] > 30 { 2 } else if self.oc_power_limit[0] < min_od_pwr || self.oc_power_limit[1] > 20 { 1 } else { 0 };
        checks.push(("OverDrive", "OD Power Slider (%)", format!("{}% .. +{}%", self.oc_power_limit[0], self.oc_power_limit[1]), format!("{}% .. +20% (Factory Range)", if is_rdna1 { -50 } else { -6 }), od_pwr_sev, "Standard AMD OverDrive power slider allows up to +15%..+20% around socket PPT."));

        let fclk_sev = if self.freq_fclk[0] > self.freq_fclk[1] || self.freq_soc[0] > self.freq_soc[1] { 2 } else if self.freq_fclk[1] < min_fclk || self.freq_soc[1] < min_soc { 1 } else { 0 };
        if !is_gcn {
        if self.freq_fclk[1] > 0 || self.freq_soc[1] > 0 {
        checks.push(("Fabric", "FCLK & SoC Max Coupling", format!("FCLK {} MHz | SoC {} MHz", self.freq_fclk[1], self.freq_soc[1]), format!("FCLK >= {} MHz | SoC >= {} MHz", min_fclk, min_soc), fclk_sev, "Factory RDNA 2 runs ~1800 MHz FCLK; factory RDNA 3 runs ~1500 MHz FCLK / 2400+ MHz SoC."));
        }

        let (disp_gen, disp_lanes, pcie_sev) = if is_rdna3 {
            let g = self.pcie_gen_speed[1] + 1;
            let l = match self.pcie_lane_count[1] { 4 => 8, 6 | 7 => 16, v => v };
            let sev = if g > 4 || l > expected_lanes { 2 } else if g < 4 || l < expected_lanes { 1 } else { 0 };
            (g, l, sev)
        } else {
            let sev = if self.pcie_gen_speed[1] > 4 || self.pcie_lane_count[1] > expected_lanes { 2 } else if self.pcie_gen_speed[1] < 4 || self.pcie_lane_count[1] < expected_lanes { 1 } else { 0 };
            (self.pcie_gen_speed[1], self.pcie_lane_count[1], sev)
        };
        if !is_gcn && (self.pcie_gen_speed[1] > 0 || self.pcie_lane_count[1] > 0) {
        checks.push(("PCIe Bus", "Link Gen & Lane Width", format!("Gen {} x{} lanes", disp_gen, disp_lanes), format!("Gen 4 x{} lanes (Silicon Native)", expected_lanes), pcie_sev, "Navi 21/22/31/32 use PCIe 4.0 x16; Navi 23/33 use x8; Navi 24 uses x4."));
        }

        let prot_ok = if is_rdna3 {
            self.smu_features.get(17).copied().unwrap_or(true) && self.smu_features.get(20).copied().unwrap_or(true)
        } else if is_rdna1 {
            self.smu_features.get(31).copied().unwrap_or(true) && self.smu_features.get(32).copied().unwrap_or(true) && self.smu_features.get(33).copied().unwrap_or(true)
        } else {
            self.smu_features.get(32).copied().unwrap_or(true) && self.smu_features.get(33).copied().unwrap_or(true) && self.smu_features.get(34).copied().unwrap_or(true)
        };
        if !is_gcn {
        checks.push(("SMU Safety", "PPT / TDC / Thermal Guards", if prot_ok { "Enabled (Hardware Guards ON)".to_string() } else { "DISABLED (Protection Bit Off!)".to_string() }, "PPT + TDC + Thermal ON".to_string(), if prot_ok { 0 } else { 2 }, "Never disable SMU PPT, TDC, or Thermal throttle bits on daily hardware."));
        }
        }

        let warn_cnt = checks.iter().filter(|c| c.4 == 1).count();
        let risk_cnt = checks.iter().filter(|c| c.4 == 2).count();
        ui.horizontal(|ui| {
            ui.strong(format!("Target Silicon: {}  |", chip_label));
            if risk_cnt == 0 && warn_cnt == 0 {
                ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "✔ All calibrated Safe Tune checks passed.");
            } else {
                ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Summary: {} Warning(s), {} Risk(s)", warn_cnt, risk_cnt));
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Quick Safe Presets:").strong().size(12.0));
            if ui.button("Quiet Undervolt").clicked() {
                self.apply_safe_preset(crate::safe_presets::SafePresetKind::QuietUndervolt);
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            }
            if ui.button("Balanced OC").clicked() {
                self.apply_safe_preset(crate::safe_presets::SafePresetKind::BalancedOc);
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            }
            if ui.button("Auto-Fix Risks").clicked() {
                self.apply_safe_preset(crate::safe_presets::SafePresetKind::AutoFixRisks);
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            }
        });
        ui.add_space(6.0);

        let avail_w = ui.available_width();
        let avail_h = ui.available_height();
        let col_w = ((avail_w - 60.0) / 5.0).max(90.0);
        egui::ScrollArea::both()
            .id_salt("safe_tune_full_scroll")
            .auto_shrink([false, false])
            .max_height(avail_h)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(avail_w, avail_h));
                egui::Grid::new("safe_tune_grid")
                    .num_columns(5)
                    .min_col_width(col_w)
                    .striped(true)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                ui.strong("Area"); ui.strong("Check"); ui.strong("Current"); ui.strong("Safe Guideline"); ui.strong("Status");
                ui.end_row();
                for (area, name, cur, guide, sev, tip) in checks {
                    ui.label(area);
                    ui.label(name).on_hover_text(tip);
                    ui.monospace(cur);
                    ui.label(guide);
                    match sev {
                        0 => ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "SAFE"),
                        1 => ui.colored_label(egui::Color32::from_rgb(245, 185, 65), "WARN").on_hover_text(tip),
                        _ => ui.colored_label(egui::Color32::from_rgb(245, 75, 95), "RISK").on_hover_text(tip),
                    };
                    ui.end_row();
                }
            });
        });
    }
}
