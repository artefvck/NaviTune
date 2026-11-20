use crate::PowerPlayApp;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
pub enum DebugSubTab {
    Audit,
    CaptureLog,
    SafeTune,
    NativeOffset,
}

impl PowerPlayApp {
    pub fn render_debug_more_tab(&mut self, ui: &mut egui::Ui) {
        self.render_more_popup_buttons(ui);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::Audit, "1. Offset Audit");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::CaptureLog, "2. Capture Log");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::SafeTune, "3. Safe Tune");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::NativeOffset, "4. Native Offset");
            ui.separator();
            if ui.button("Struct Window").clicked() {
                self.show_struct_inspector_window = !self.show_struct_inspector_window;
            }
            if ui.button("Dump Audit -> debug.log").clicked() {
                self.sync_fields_to_buffer();
                let mut dump = format!(
                    "=== FULL PPTABLE SNAPSHOT ({} bytes, source={}) ===\n",
                    self.raw_buffer.len(),
                    self.status_msg
                );
                dump.push_str(&format!(
                    "GFX Clock: {:?} MHz | MEM Clock: {:?} MHz | Power: {} W | TDC GFX/Soc: {}/{} A\n",
                    self.freq_gfx, self.freq_soc, self.power_limit_w, self.tdc_gfx, self.tdc_soc
                ));
                dump.push_str(&format!(
                    "Volt GFX: {:?} mV | Volt Soc: {:?} mV | TVmin GFX/Soc: {:?}/{:?} | Vmin LH GFX/Soc: {:?}/{:?}\n",
                    self.volt_gfx, self.volt_soc, self.tvmin_gfx, self.tvmin_soc, self.vmin_lh_gfx, self.vmin_lh_soc
                ));
                for (i, chunk) in self.raw_buffer.chunks(16).enumerate() {
                    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                    dump.push_str(&format!("0x{:04X}: {}\n", i * 16, hex.join(" ")));
                }
                crate::ui_theme::log_debug("SNAPSHOT", &dump);
                if let Some(lp) = crate::ui_theme::debug_log_path() {
                    self.status_msg = format!("Dumped full audit & hex snapshot to {}", lp.display());
                }
            }
        });
        ui.separator();

        match self.debug_subtab {
            DebugSubTab::Audit => self.draw_debug_audit(ui),
            DebugSubTab::CaptureLog => self.draw_debug_capture_log(ui),
            DebugSubTab::SafeTune => self.draw_debug_safe_tune(ui),
            DebugSubTab::NativeOffset => self.draw_debug_native_offset(ui),
        }
        self.render_struct_inspector_viewport(ui.ctx());
    }
    fn draw_debug_audit(&mut self, ui: &mut egui::Ui) {
        use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name};
        ui.checkbox(&mut self.debug_only_mismatch, "Show only mismatches (DIFF)");
        ui.add_space(4.0);

        let Some(t) = AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer) else {
            ui.label("Load a 2,470-byte RDNA 2 PPTable to run native struct audit.");
            return;
        };

        let (od_gfx_min, od_gfx_max) = ({t.od_gfx_min[0]}, {t.od_gfx_max[0]});
        let (od_mem_min, od_mem_max) = ({t.od_mem_min[0]}, {t.od_mem_max[0]});
        let (od_pwr_min, od_pwr_max) = ({t.od_pwr_min}, {t.od_pwr_max});
        let (od_fan_min, od_fan_max) = ({t.od_fan_min}, {t.od_fan_max});
        let (od_ac_min, od_ac_max) = ({t.od_acoustic_min}, {t.od_acoustic_max});
        let (pm_q_l, pm_q_t) = ({t.pm_quiet_limit}, {t.pm_quiet_temp});
        let (pm_b_l, pm_b_t) = ({t.pm_balanced_limit}, {t.pm_balanced_temp});
        let (pm_t_l, pm_t_t) = ({t.pm_turbo_limit}, {t.pm_turbo_temp});
        let (pm_r_l, pm_r_t) = ({t.pm_rage_limit}, {t.pm_rage_temp});
        let pwr_w = {t.socket_power_limit_ac[0]};
        let (tdc_g, tdc_s) = ({t.tdc_limit_gfx}, {t.tdc_limit_soc});
        let (vg_min, vg_max) = ({t.min_voltage_gfx} / 4, {t.max_voltage_gfx} / 4);
        let (vs_min, vs_max) = ({t.min_voltage_soc} / 4, {t.max_voltage_soc} / 4);
        let (ug_min, ug_off) = ({t.ulv_voltage_vmin_gfx} / 4, {t.ulv_voltage_offset_gfx} / 4);
        let (us_min, us_off) = ({t.ulv_voltage_vmin_soc} / 4, {t.ulv_voltage_offset_soc} / 4);
        let (vddci3, mvdd3) = ({t.mem_vddci[3]} / 4, {t.mem_mvdd[3]} / 4);
        let (fg_min, fg_max) = ({t.freq_gfx_min}, {t.freq_gfx_max});
        let (fs_min, fs_max) = ({t.freq_soc[0]}, {t.freq_soc[1]});
        let (ff_min, ff_max) = ({t.freq_fclk[0]}, {t.freq_fclk[1]});
        let (uclk3, div3) = ({t.freq_uclk[3]}, {t.mem_clk_div[3]});
        let (pcie_g0, pcie_g1) = ({t.pcie_gen_speed[0]}, {t.pcie_gen_speed[1]});
        let (pcie_l0, pcie_l1) = ({t.pcie_lane_count[0]}, {t.pcie_lane_count[1]});
        let (lclk0, lclk1) = ({t.lclk_freq[0]}, {t.lclk_freq[1]});
        let crv_c = {t.static_volt_gfx.c};
        let (dctol_g, dctol_s) = ({t.dctol_gfx} as i32, {t.dctol_soc} as i32);
        let (dcb_max_g, dcb_max_s) = ({t.dcbtc_max_gfx} as f64 / 4.0, {t.dcbtc_max_soc} as f64 / 4.0);
        let (dcb_gb_g, dcb_gb_s) = ({t.dcbtc_gb_gfx} as f64 / 4.0, {t.dcbtc_gb_soc} as f64 / 4.0);
        let (f_pwm, f_max) = ({t.fan_pwm_min}, {t.fan_max_rpm});
        let (f_stop, f_start, f_tgt) = ({t.fan_stop_temp}, {t.fan_start_temp}, {t.fan_target_temp});
        let (od_ft, od_fs) = ({t.od_fan_curve_min[0].temp}, {t.od_fan_curve_min[0].pwm});
        let rows: Vec<(&str, &str, usize, String, String)> = vec![
            ("OC Limits", "OD8 GFX Min / Max", field_offset_by_name("od_gfx"), format!("{} / {}", od_gfx_min, od_gfx_max), format!("{} / {}", self.oc_gfx_clock[0], self.oc_gfx_clock[1])),
            ("OC Limits", "OD8 Mem Min / Max", field_offset_by_name("od_mem"), format!("{} / {}", od_mem_min, od_mem_max), format!("{} / {}", self.oc_mem_clock[0], self.oc_mem_clock[1])),
            ("OC Limits", "OD8 Pwr Min / Max", field_offset_by_name("od_pwr"), format!("{} / {}", od_pwr_min, od_pwr_max), format!("{} / {}", self.oc_power_limit[0], self.oc_power_limit[1])),
            ("OC Limits", "OD8 Fan Min / Max", field_offset_by_name("od_fan"), format!("{} / {}", od_fan_min, od_fan_max), format!("{} / {}", self.oc_fan_rpm[0], self.oc_fan_rpm[1])),
            ("OC Limits", "OD8 Acoustic Min / Max", field_offset_by_name("od_acoustic"), format!("{} / {}", od_ac_min, od_ac_max), format!("{} / {}", self.oc_fan_acoustic_rpm[0], self.oc_fan_acoustic_rpm[1])),
            ("OC Limits", "PM Quiet Limit / Temp", field_offset_by_name("pm_quiet"), format!("{} / {}", pm_q_l, pm_q_t), format!("{} / {}", self.pm_quiet[0], self.pm_quiet[1])),
            ("OC Limits", "PM Balanced Limit / Temp", field_offset_by_name("pm_balanced"), format!("{} / {}", pm_b_l, pm_b_t), format!("{} / {}", self.pm_balanced[0], self.pm_balanced[1])),
            ("OC Limits", "PM Turbo Limit / Temp", field_offset_by_name("pm_turbo"), format!("{} / {}", pm_t_l, pm_t_t), format!("{} / {}", self.pm_turbo[0], self.pm_turbo[1])),
            ("OC Limits", "PM Rage Limit / Temp", field_offset_by_name("pm_rage"), format!("{} / {}", pm_r_l, pm_r_t), format!("{} / {}", self.pm_rage[0], self.pm_rage[1])),
            ("Power", "Power Limit (W)", field_offset_by_name("pwr_limit_w"), format!("{}", pwr_w), format!("{}", self.power_limit_w)),
            ("Power", "TDC GFX / Soc (A)", field_offset_by_name("tdc_gfx"), format!("{} / {}", tdc_g, tdc_s), format!("{} / {}", self.tdc_gfx, self.tdc_soc)),
            ("Power", "GFX Vmin / Vmax (mV)", field_offset_by_name("volt_gfx"), format!("{} / {}", vg_min, vg_max), format!("{} / {}", self.volt_gfx[0], self.volt_gfx[1])),
            ("Power", "Soc Vmin / Vmax (mV)", field_offset_by_name("volt_soc"), format!("{} / {}", vs_min, vs_max), format!("{} / {}", self.volt_soc[0], self.volt_soc[1])),
            ("Power", "ULV GFX Vmin / Offset", field_offset_by_name("ulv_gfx"), format!("{} / {}", ug_min, ug_off), format!("{} / {}", self.ulv_gfx[0], self.ulv_gfx[1])),
            ("Power", "ULV Soc Vmin / Offset", field_offset_by_name("ulv_soc"), format!("{} / {}", us_min, us_off), format!("{} / {}", self.ulv_soc[0], self.ulv_soc[1])),
            ("Power", "Mem DPM3 VDDCI / MVDD", field_offset_by_name("mem_dpm_volt"), format!("{} / {}", vddci3, mvdd3), format!("{} / {}", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1])),
            ("Frequency", "GFX Min / Max (MHz)", field_offset_by_name("freq_gfx"), format!("{} / {}", fg_min, fg_max), format!("{} / {}", self.freq_gfx[0], self.freq_gfx[1])),
            ("Frequency", "Soc Min / Max (MHz)", field_offset_by_name("freq_soc"), format!("{} / {}", fs_min, fs_max), format!("{} / {}", self.freq_soc[0], self.freq_soc[1])),
            ("Frequency", "FCLK Min / Max (MHz)", field_offset_by_name("freq_fclk"), format!("{} / {}", ff_min, ff_max), format!("{} / {}", self.freq_fclk[0], self.freq_fclk[1])),
            ("Frequency", "Mem DPM3 Freq / Div", field_offset_by_name("mem_dpm_freq"), format!("{} / {}", uclk3, div3), format!("{} / {}", self.mem_dpm_freq[3][0], self.mem_dpm_freq[3][1])),
            ("Frequency", "PCIe Gen & Lanes", field_offset_by_name("lclk") - 4, format!("Gen {}/{} x{}/{}", pcie_g0, pcie_g1, pcie_l0, pcie_l1), format!("Gen {}/{} x{}/{}", self.pcie_gen_speed[0], self.pcie_gen_speed[1], self.pcie_lane_count[0], self.pcie_lane_count[1])),
            ("Frequency", "LCLK Min / Max (MHz)", field_offset_by_name("lclk"), format!("{} / {}", lclk0, lclk1), format!("{} / {}", self.lclk_mhz[0], self.lclk_mhz[1])),
            ("Curve", "Static Curve GFX c", field_offset_by_name("curve_gfx"), format!("{:.6}", crv_c), format!("{:.6}", self.static_volt_c[0])),
            ("Curve", "DcTol GFX / Soc", field_offset_by_name("dctol"), format!("{} / {}", dctol_g, dctol_s), format!("{} / {}", self.dctol[0], self.dctol[1])),
            ("Curve", "DcBtc Max GFX / Soc", field_offset_by_name("dcbtc_max"), format!("{:.0} / {:.0}", dcb_max_g, dcb_max_s), format!("{:.0} / {:.0}", self.dcbtc_max[0], self.dcbtc_max[1])),
            ("Curve", "DcBtc GB GFX / Soc", field_offset_by_name("dcbtc_gb"), format!("{:.0} / {:.0}", dcb_gb_g, dcb_gb_s), format!("{:.0} / {:.0}", self.dcbtc_gb[0], self.dcbtc_gb[1])),
            ("Fan", "PWM Min / Max RPM", field_offset_by_name("fan_pwm_min"), format!("{} / {}", f_pwm, f_max), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)),
            ("Fan", "Stop / Start / Target °C", field_offset_by_name("fan_stop_temp"), format!("{} / {} / {}", f_stop, f_start, f_tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)),
            ("Fan", "OD Temp / Speed Min", field_offset_by_name("od_fan_temp"), format!("{} / {}", od_ft, od_fs), format!("{} / {}", self.od_fan_temp[0], self.od_fan_speed[0])),
        ];

        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            egui::Grid::new("debug_audit_grid").striped(true).spacing([10.0, 4.0]).show(ui, |ui| {
                ui.strong("Tab"); ui.strong("Field"); ui.strong("Offset"); ui.strong("Struct"); ui.strong("UI"); ui.strong("Status");
                ui.end_row();
                for (tab, field, off, raw_s, ui_s) in rows {
                    let ok = raw_s == ui_s;
                    if self.debug_only_mismatch && ok { continue; }
                    ui.label(tab);
                    ui.label(field);
                    ui.monospace(format!("0x{:03X}", off));
                    ui.monospace(raw_s);
                    ui.monospace(ui_s);
                    if ok {
                        ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "OK");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(245, 75, 95), "DIFF");
                    }
                    ui.end_row();
                }
            });
        });
    }
    fn build_full_debug_dump(&self) -> String {
        let mut smu_mask = 0u64;
        for (bit, &en) in self.smu_features.iter().enumerate() {
            if en { smu_mask |= 1u64 << bit; }
        }
        let enabled_bits: Vec<String> = (0..64)
            .filter(|&b| self.smu_features[b])
            .map(|b| format!("{:02}", b))
            .collect();
        let gpu_desc = format!("[{}]", self.selected_gpu);

        let mut vars: Vec<(String, String)> = vec![
            ("vid_curve_gfx_a".into(), format!("{:.6}", self.static_volt_a[0])),
            ("vid_curve_gfx_b".into(), format!("{:.6}", self.static_volt_b[0])),
            ("vid_curve_gfx_c".into(), format!("{:.6}", self.static_volt_c[0])),
            ("vid_curve_soc_a".into(), format!("{:.6}", self.static_volt_a[1])),
            ("vid_curve_soc_b".into(), format!("{:.6}", self.static_volt_b[1])),
            ("vid_curve_soc_c".into(), format!("{:.6}", self.static_volt_c[1])),
            ("curve_gfx_a".into(), format!("{:.6}", self.static_off_a[0])),
            ("curve_gfx_b".into(), format!("{:.6}", self.static_off_b[0])),
            ("curve_gfx_c".into(), format!("{:.6}", self.static_off_c[0])),
            ("curve_soc_a".into(), format!("{:.6}", self.static_off_a[1])),
            ("curve_soc_b".into(), format!("{:.6}", self.static_off_b[1])),
            ("curve_soc_c".into(), format!("{:.6}", self.static_off_c[1])),
            ("dctol_gfx".into(), format!("{}", self.dctol[0])),
            ("dctol_soc".into(), format!("{}", self.dctol[1])),
            ("dcbtc_en_gfx".into(), format!("{}", self.dcbtc_gfx_en as u8)),
            ("dcbtc_en_soc".into(), format!("{}", self.dcbtc_soc_en as u8)),
            ("fan_pwm_min".into(), format!("{}", self.fan_pwm_min)),
            ("fan_acoustic_lim".into(), format!("{}", self.fan_acoustic_limit)),
            ("fan_throttling".into(), format!("{}", self.fan_throttling_rpm)),
            ("fan_max_rpm".into(), format!("{}", self.fan_max_rpm)),
            ("fan_target_temp".into(), format!("{}", self.fan_target_temp)),
            ("fan_target_gfx".into(), format!("{}", self.fan_target_gfx_clock)),
            ("fan_zero_rpm".into(), format!("{}", self.fan_zero_rpm_enable as u8)),
            ("fan_stop_temp".into(), format!("{}", self.fan_stop_temp)),
            ("fan_start_temp".into(), format!("{}", self.fan_start_temp)),
            ("freq_gfx_min".into(), format!("{}", self.freq_gfx[0])),
            ("freq_gfx_max".into(), format!("{}", self.freq_gfx[1])),
            ("freq_soc_min".into(), format!("{}", self.freq_soc[0])),
            ("freq_soc_max".into(), format!("{}", self.freq_soc[1])),
            ("freq_fclk_min".into(), format!("{}", self.freq_fclk[0])),
            ("freq_fclk_max".into(), format!("{}", self.freq_fclk[1])),
            ("lclk_min".into(), format!("{}", self.lclk_mhz[0])),
            ("lclk_max".into(), format!("{}", self.lclk_mhz[1])),
            ("pcie_gen_min".into(), format!("{}", self.pcie_gen_speed[0])),
            ("pcie_gen_max".into(), format!("{}", self.pcie_gen_speed[1])),
            ("pcie_lane_min".into(), format!("{}", self.pcie_lane_count[0])),
            ("pcie_lane_max".into(), format!("{}", self.pcie_lane_count[1])),
            ("power_limit_gpu".into(), format!("{}", self.power_limit_w)),
            ("tdc_gfx".into(), format!("{}", self.tdc_gfx)),
            ("tdc_soc".into(), format!("{}", self.tdc_soc)),
            ("vmin_gfx".into(), format!("{}", self.volt_gfx[0])),
            ("vmax_gfx".into(), format!("{}", self.volt_gfx[1])),
            ("vmin_soc".into(), format!("{}", self.volt_soc[0])),
            ("vmax_soc".into(), format!("{}", self.volt_soc[1])),
            ("smu_features_mask".into(), format!("0x{:016X}", smu_mask)),
        ];
        for i in 0..5 {
            vars.push((format!("fan_curve_pwm_{}", i), format!("{}", self.fan_curve_pwm[i])));
            vars.push((format!("fan_curve_temp_{}", i), format!("{}", self.fan_curve_temp[i])));
        }
        for i in 0..4 {
            vars.push((format!("mem_div{}", i), format!("{}", self.mem_dpm_freq[i][1])));
            vars.push((format!("mem_dpm{}", i), format!("{}", self.mem_dpm_freq[i][0])));
            vars.push((format!("vddci_dpm{}", i), format!("{}", self.mem_dpm_volt[i][0])));
            vars.push((format!("mvdd_dpm{}", i), format!("{}", self.mem_dpm_volt[i][1])));
            vars.push((format!("power_limit_ac_{}", i), format!("{}", self.power_limit_ac[i])));
        }
        for i in 0..9 {
            vars.push((format!("temp_limit_{}", i), format!("{}", self.temp_limits[i])));
        }
        vars.sort_by(|a, b| a.0.cmp(&b.0));

        let mut out = String::new();
        out.push_str("=== NaviTune FULL SYSTEM & TABLE DEBUG DUMP ===\n");
        out.push_str(&format!("OS / Arch        : {} / {}\n", std::env::consts::OS, std::env::consts::ARCH));
        out.push_str(&format!("Selected GPU     : {}\n", gpu_desc));
        out.push_str(&format!("Telemetry Active : {}\n", self.telemetry_data.lock().is_ok()));
        let chip_name = crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).map(|t| t.detect_chip(&self.raw_buffer).display_name()).unwrap_or("Unknown");
        out.push_str(&format!("Detected Silicon : {}\n", chip_name));
        out.push_str(&format!("Encoded Size     : {} bytes\n", self.raw_buffer.len()));
        out.push_str(&format!("SMU Enabled Bits : [{}]\n", enabled_bits.join(", ")));
        out.push_str(&format!("Decoded Vars     : {} entries\n", vars.len()));
        for (k, v) in vars {
            out.push_str(&format!("  {} = {}\n", k, v));
        }
        out
    }

    fn draw_debug_capture_log(&mut self, ui: &mut egui::Ui) {
        let dump_text = self.build_full_debug_dump();
        ui.horizontal(|ui| {
            if ui.button("Copy Full Dump").clicked() {
                ui.ctx().copy_text(dump_text.clone());
                self.status_msg = "Copied full system & table debug dump to clipboard.".to_string();
            }
            if ui.button("Save Log (.log)").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("navitune_debug.log")
                    .add_filter("Log / Text", &["log", "txt"])
                    .save_file()
                {
                    match std::fs::write(&path, &dump_text) {
                        Ok(_) => self.status_msg = format!("Saved debug dump to {}", path.display()),
                        Err(e) => self.status_msg = format!("Failed to save debug dump: {}", e),
                    }
                }
            }
        });
        ui.add_space(6.0);
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let mut buf = dump_text;
            ui.add(
                egui::TextEdit::multiline(&mut buf)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
        });
    }

    fn draw_debug_safe_tune(&self, ui: &mut egui::Ui) {
        use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, NaviRDNA2Chip};

        let chip = AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer)
            .map(|t| t.detect_chip(&self.raw_buffer))
            .unwrap_or(NaviRDNA2Chip::Navi23DimgreyCavefish);

        let (max_gfx_v, max_soc_v, min_w, max_w, max_tdc_gfx, max_tdc_soc, safe_uclk_max) = match chip {
            NaviRDNA2Chip::Navi23DimgreyCavefish => (1150, 1150, 100, 135, 100, 22, 875),
            NaviRDNA2Chip::Navi24BeigeGoby => (1150, 1150, 53, 90, 75, 18, 1000),
            NaviRDNA2Chip::Navi22NavyFlounder => (1200, 1200, 186, 230, 175, 32, 1000),
            NaviRDNA2Chip::Navi21SiennaCichlid => (1200, 1200, 250, 300, 255, 40, 1000),
        };

        let mut checks: Vec<(&str, &str, String, String, u8, &str)> = Vec::new();

        let gfx_ok = self.freq_gfx[0] > 0
            && self.freq_gfx[0] < self.freq_gfx[1]
            && (self.oc_gfx_clock[1] == 0 || self.freq_gfx[1] <= self.oc_gfx_clock[1]);
        let gfx_sev = if self.freq_gfx[0] >= self.freq_gfx[1] { 2 } else if !gfx_ok { 1 } else { 0 };
        checks.push((
            "Clocks",
            "GFX Min / Max vs OD8",
            format!("{} / {} MHz (OD8 Max: {})", self.freq_gfx[0], self.freq_gfx[1], self.oc_gfx_clock[1]),
            "Min < Max <= OD8 Max".to_string(),
            gfx_sev,
            "Keep GFX Max within OD8 Max to prevent driver clock clamping.",
        ));

        let uclk_dpm3 = self.mem_dpm_freq[3][0];
        let uclk_sev = if uclk_dpm3 > safe_uclk_max + 50 { 2 } else if uclk_dpm3 > safe_uclk_max || (self.oc_mem_clock[1] > 0 && uclk_dpm3 > self.oc_mem_clock[1]) { 1 } else { 0 };
        checks.push((
            "Clocks",
            "UCLK DPM3 vs Safe Max",
            format!("{} MHz (OD8 Max: {} MHz)", uclk_dpm3, self.oc_mem_clock[1]),
            format!("<= {} MHz (OD8 <= {})", safe_uclk_max, self.oc_mem_clock[1]),
            uclk_sev,
            "GDDR6 strap timing loosens or throws ECC retries if UCLK exceeds safe ceiling.",
        ));

        let eff_vmax_gfx = (self.volt_gfx[1] as f32 + self.static_off_c[0] as f32 * 1000.0).round() as i32;
        let v_gfx_sev = if self.volt_gfx[0] > self.volt_gfx[1] || eff_vmax_gfx > max_gfx_v + 50 || eff_vmax_gfx < 650 {
            2
        } else if self.volt_gfx[0] < 675 || eff_vmax_gfx > max_gfx_v {
            1
        } else {
            0
        };
        checks.push((
            "Voltage",
            "GFX Vmin / Vmax (+Offset)",
            format!("{} / {} mV (Eff Max: {} mV)", self.volt_gfx[0], self.volt_gfx[1], eff_vmax_gfx),
            format!("762..{} mV (Min <= Max)", max_gfx_v),
            v_gfx_sev,
            "Effective Vmax includes Static Voltage Offset (c); keep within silicon safe bounds.",
        ));

        let v_soc_sev = if self.volt_soc[0] > self.volt_soc[1] || self.volt_soc[1] > max_soc_v + 50 {
            2
        } else if self.volt_soc[0] < 675 || self.volt_soc[1] > max_soc_v {
            1
        } else {
            0
        };
        checks.push((
            "Voltage",
            "Soc Vmin / Vmax",
            format!("{} / {} mV", self.volt_soc[0], self.volt_soc[1]),
            format!("762..{} mV (Min <= Max)", max_soc_v),
            v_soc_sev,
            "Keep SoC voltage within silicon spec for Infinity Cache & memory controller stability.",
        ));

        let pwr_sev = if self.power_limit_w <= 0 || self.tdc_gfx <= 0 || self.power_limit_w > max_w + 40 {
            2
        } else if self.power_limit_w < min_w || self.power_limit_w > max_w || self.tdc_gfx > max_tdc_gfx || self.tdc_soc > max_tdc_soc {
            1
        } else {
            0
        };
        checks.push((
            "Power",
            "PPT & TDC GFX / Soc",
            format!("{} W | {} A / {} A", self.power_limit_w, self.tdc_gfx, self.tdc_soc),
            format!("{}..{} W | TDC <= {}/{} A", min_w, max_w, max_tdc_gfx, max_tdc_soc),
            pwr_sev,
            "Calibrated to silicon VRM current & socket power limits.",
        ));

        let mut mem_v_sev = 0u8;
        for i in 0..4 {
            let vddci = self.mem_dpm_volt[i][0];
            let mvdd = self.mem_dpm_volt[i][1];
            if vddci > mvdd || mvdd > 1425 {
                mem_v_sev = 2;
            } else if (vddci > 900 || mvdd > 1365) && mem_v_sev < 1 {
                mem_v_sev = 1;
            }
        }
        checks.push((
            "Memory",
            "VDDCI / MVDD DPM 0..3",
            format!("DPM3: {} / {} mV", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1]),
            "VDDCI <= MVDD <= 1365 mV".to_string(),
            mem_v_sev,
            "Stock Navi 23 DPM3 is 850 / 1350 mV; keep MVDD <= 1365 mV without extra VRAM cooling.",
        ));

        let hotspot_lim = self.temp_limits[1];
        let fan_sev = if self.fan_stop_temp >= self.fan_start_temp || self.fan_start_temp >= self.fan_target_temp || hotspot_lim > 115 {
            2
        } else if self.fan_target_temp > 95 || self.fan_pwm_min < 10 || hotspot_lim > 110 {
            1
        } else {
            0
        };
        checks.push((
            "Thermal",
            "Fan & Hotspot Limits",
            format!("Stop {}° / Start {}° / Tgt {}° | Hotspot {}°C", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp, hotspot_lim),
            "Stop < Start < Tgt <= 95° | Hotspot <= 110°C".to_string(),
            fan_sev,
            "Ensures smooth Zero-RPM hysteresis and safe Junction/Hotspot throttling margin.",
        ));

        let od_pwr_sev = if self.oc_power_limit[0] > self.oc_power_limit[1] || self.oc_power_limit[1] > 30 {
            2
        } else if self.oc_power_limit[0] < -10 || self.oc_power_limit[1] > 20 {
            1
        } else {
            0
        };
        checks.push((
            "OverDrive",
            "OD8 Power Slider (%)",
            format!("{}% .. +{}%", self.oc_power_limit[0], self.oc_power_limit[1]),
            "-6% .. +20% (Factory Range)".to_string(),
            od_pwr_sev,
            "Standard AMD OverDrive power slider allows -6% to +20% around socket PPT.",
        ));

        let fclk_sev = if self.freq_fclk[0] > self.freq_fclk[1] || self.freq_soc[0] > self.freq_soc[1] {
            2
        } else if self.freq_fclk[1] < 1550 || self.freq_soc[1] < 1150 {
            1
        } else {
            0
        };
        checks.push((
            "Fabric",
            "FCLK & SoC Max Coupling",
            format!("FCLK {} MHz | SoC {} MHz", self.freq_fclk[1], self.freq_soc[1]),
            "FCLK >= 1550 MHz | SoC >= 1150 MHz".to_string(),
            fclk_sev,
            "Factory Navi 23 runs 1800 MHz FCLK / 1280 MHz SoC; lowering bottlenecks Infinity Cache.",
        ));

        let expected_lanes = match chip {
            NaviRDNA2Chip::Navi23DimgreyCavefish => 8,
            NaviRDNA2Chip::Navi24BeigeGoby => 4,
            _ => 16,
        };
        let pcie_sev = if self.pcie_gen_speed[1] > 4 || self.pcie_lane_count[1] > expected_lanes {
            2
        } else if self.pcie_gen_speed[1] < 4 || self.pcie_lane_count[1] < expected_lanes {
            1
        } else {
            0
        };
        checks.push((
            "PCIe Bus",
            "Link Gen & Lane Width",
            format!("Gen {} x{} lanes", self.pcie_gen_speed[1], self.pcie_lane_count[1]),
            format!("Gen 4 x{} lanes (Silicon Native)", expected_lanes),
            pcie_sev,
            "Navi 23 is natively PCIe 4.0 x8 (Navi 24 x4, Navi 21/22 x16).",
        ));

        let prot_ok = self.smu_features.get(32).copied().unwrap_or(true)
            && self.smu_features.get(33).copied().unwrap_or(true)
            && self.smu_features.get(34).copied().unwrap_or(true);
        checks.push((
            "SMU Safety",
            "PPT / TDC / Thermal Guards",
            if prot_ok { "Enabled (Bits 32, 33, 34 ON)".to_string() } else { "DISABLED (Protection Bit Off!)".to_string() },
            "PPT + TDC + Thermal ON".to_string(),
            if prot_ok { 0 } else { 2 },
            "Never disable SMU PPT, TDC, or Thermal throttle bits on daily hardware.",
        ));

        let warn_cnt = checks.iter().filter(|c| c.4 == 1).count();
        let risk_cnt = checks.iter().filter(|c| c.4 == 2).count();
        ui.horizontal(|ui| {
            ui.strong(format!("Target Silicon: {}  |", chip.display_name()));
            if risk_cnt == 0 && warn_cnt == 0 {
                ui.colored_label(egui::Color32::from_rgb(80, 210, 120), "✔ All calibrated Safe Tune checks passed.");
            } else {
                ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Summary: {} Warning(s), {} Risk(s)", warn_cnt, risk_cnt));
            }
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            egui::Grid::new("safe_tune_grid").striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
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

    fn render_struct_inspector_viewport(&mut self, ctx: &egui::Context) {
        if !self.show_struct_inspector_window {
            return;
        }
        let vp_id = egui::ViewportId::from_hash_of("struct_binary_verifier_viewport");
        let builder = egui::ViewportBuilder::default()
            .with_title("Struct & Binary Structure Verifier (AtomSmcDpmInfoV4_9)")
            .with_inner_size([680.0, 520.0])
            .with_resizable(true);

        ctx.show_viewport_immediate(vp_id, builder, |ctx, _class| {
            if ctx.input(|i| i.viewport().close_requested()) {
                self.show_struct_inspector_window = false;
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.raw_buffer.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("Load a PowerPlay table (.bin, .rom, .reg, .mpt, pp_table) first.");
                    });
                    return;
                }

                let b = &self.raw_buffer;
                let hdr_sz = if b.len() >= 2 { u16::from_le_bytes([b[0], b[1]]) as usize } else { 0 };
                let frev = b.get(2).copied().unwrap_or(0);
                let crev = b.get(3).copied().unwrap_or(0);
                let trev = b.get(4).copied().unwrap_or(0);

                let Some(t) = crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(b) else {
                    ui.colored_label(egui::Color32::from_rgb(245, 75, 95), "Buffer is smaller than AtomSmcDpmInfoV4_9 (2470 bytes).");
                    return;
                };

                use crate::pptable_rdna2::field_offset_by_name as off;
                let entries: Vec<(&str, &str, usize, usize, String)> = vec![
                    ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
                    ("OD8", "od_feature_flags", 0x0C2, 16, format!("{:?}", {t.od_feature_flags})),
                    ("OD8", "od_gfx_min / max", off("od_gfx"), 4, format!("{} / {} MHz", {t.od_gfx_min[0]}, {t.od_gfx_max[0]})),
                    ("OD8", "od_mem_min / max", off("od_mem"), 4, format!("{} / {} MHz", {t.od_mem_min[0]}, {t.od_mem_max[0]})),
                    ("OD8", "od_pwr_min / max", off("od_pwr"), 4, format!("{} / {} %", {t.od_pwr_min}, {t.od_pwr_max})),
                    ("OD8", "od_fan_min / max", off("od_fan"), 4, format!("{} / {} RPM", {t.od_fan_min}, {t.od_fan_max})),
                    ("OD8", "od_acoustic_min / max", off("od_acoustic"), 4, format!("{} / {} RPM", {t.od_acoustic_min}, {t.od_acoustic_max})),
                    ("PowerModes", "Quiet / Balanced Limit", off("pm_quiet"), 8, format!("{}% / {}% (Temp {}°C / {}°C)", {t.pm_quiet_limit}, {t.pm_balanced_limit}, {t.pm_quiet_temp}, {t.pm_balanced_temp})),
                    ("PowerModes", "Turbo / Rage Limit", off("pm_turbo"), 8, format!("{}% / {}% (Temp {}°C / {}°C)", {t.pm_turbo_limit}, {t.pm_rage_limit}, {t.pm_turbo_temp}, {t.pm_rage_temp})),
                    ("SMU", "features_to_run (u64)", 0x326, 8, format!("0x{:016X}", {t.features_to_run})),
                    ("Power", "tdc_limit_gfx / soc", off("tdc_gfx"), 4, format!("{} A / {} A", {t.tdc_limit_gfx}, {t.tdc_limit_soc})),
                    ("Power", "power_limit_ac[0..3]", off("pwr_limit_w"), 8, format!("{:?} W", {t.socket_power_limit_ac[0]})),
                    ("Voltage", "ulv_vmin_gfx / soc", off("ulv_gfx"), 4, format!("{} mV / {} mV", {t.ulv_voltage_vmin_gfx} / 4, {t.ulv_voltage_vmin_soc} / 4)),
                    ("Voltage", "min/max_voltage_gfx", off("volt_gfx"), 8, format!("{}..{} mV", {t.min_voltage_gfx} / 4, {t.max_voltage_gfx} / 4)),
                    ("Voltage", "min/max_voltage_soc", off("volt_soc"), 8, format!("{}..{} mV", {t.min_voltage_soc} / 4, {t.max_voltage_soc} / 4)),
                    ("Clocks", "freq_gfx_max / freq_soc", off("freq_soc"), 4, format!("GFX Max={} MHz | Soc={}..{} MHz", {t.freq_gfx_max}, {t.freq_soc[0]}, {t.freq_soc[1]})),
                    ("Clocks", "freq_fclk[0..1]", off("freq_fclk"), 4, format!("{}..{} MHz", {t.freq_fclk[0]}, {t.freq_fclk[1]})),
                    ("Memory", "freq_uclk[0..3]", off("mem_dpm_freq"), 8, format!("{:?} MHz", {t.freq_uclk})),
                    ("Memory", "mem_clk_div[0..3]", 0x65A, 4, format!("{:?}", {t.mem_clk_div})),
                    ("Memory", "mem_vddci / mvdd", off("mem_dpm_volt"), 16, format!("VDDCI={:?} | MVDD={:?}", {t.mem_vddci}, {t.mem_mvdd})),
                    ("PCIe/LCLK", "pcie_gen / lanes / lclk", off("lclk") - 4, 8, format!("Gen {:?} | Lanes {:?} | LCLK {:?} MHz", {t.pcie_gen_speed}, {t.pcie_lane_count}, {t.lclk_freq})),
                    ("Fan", "stop / start / target", off("fan_stop_temp"), 4, format!("Stop={}°C, Start={}°C, Target={}°C, MaxRPM={}", {t.fan_stop_temp}, {t.fan_start_temp}, {t.fan_target_temp}, {t.fan_max_rpm})),
                    ("Curve", "static_volt_gfx (a,b,c)", off("curve_gfx"), 12, format!("a={:.6}, b={:.6}, c={:.6}", {t.static_volt_gfx.a}, {t.static_volt_gfx.b}, {t.static_volt_gfx.c})),
                    ("DcBtc", "dctol / dcbtc_max / gb", off("dctol"), 20, format!("DcTol={}/{} | Max={}/{} mV | GB={}/{} mV", {t.dctol_gfx}, {t.dctol_soc}, {t.dcbtc_max_gfx}/4, {t.dcbtc_max_soc}/4, {t.dcbtc_gb_gfx}/4, {t.dcbtc_gb_soc}/4)),
                ];

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("struct_verifier_grid").striped(true).spacing([10.0, 5.0]).show(ui, |ui| {
                        ui.strong("Section");
                        ui.strong("Struct Field");
                        ui.strong("Table Off");
                        ui.strong("VBIOS Off");
                        ui.strong("Raw Bytes");
                        ui.strong("Decoded Value");
                        ui.end_row();
                        for (sec, name, offset, len, val_str) in entries {
                            let end = (offset + len).min(b.len());
                            let raw_hex: String = b[offset..end].iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                            let rom_off_str = match self.vbios_ppt_offset {
                                Some(voff) => format!("0x{:04X}", voff + offset),
                                None => "N/A".to_string(),
                            };
                            ui.label(sec);
                            ui.monospace(name);
                            ui.monospace(format!("0x{:03X}", offset));
                            ui.monospace(rom_off_str);
                            ui.monospace(raw_hex);
                            ui.label(val_str);
                            ui.end_row();
                        }
                    });
                });
            });
        });
    }

    fn draw_debug_native_offset(&self, ui: &mut egui::Ui) {
        use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name as off};

        if self.raw_buffer.is_empty() {
            ui.label("Load a PowerPlay table (.bin, .rom, .reg, .mpt, or ioreg) first.");
            return;
        }

        let b = &self.raw_buffer;
        let hdr_sz = if b.len() >= 2 { u16::from_le_bytes([b[0], b[1]]) as usize } else { 0 };
        let frev = b.get(2).copied().unwrap_or(0);
        let crev = b.get(3).copied().unwrap_or(0);
        let trev = b.get(4).copied().unwrap_or(0);
        let od_base = 0x0C2usize;
        let smc_base = 0x322usize;

        let mut lines = vec![
            "=== NATIVE RUST OFFSET DEBUG ===".to_string(),
            format!("Buffer Size : {} bytes (header structure_size={})", b.len(), hdr_sz),
            format!("Revisions   : format_rev=0x{:02X}, content_rev=0x{:02X}, table_rev=0x{:02X}", frev, crev, trev),
            format!("Bases       : od_base=0x{:03X} ({}), smc_base=0x{:03X} ({})", od_base, od_base, smc_base, smc_base),
            String::new(),
            "Decoded Keys:".to_string(),
        ];

        if let Some(t) = AtomSmcDpmInfoV4_9::from_bytes(b) {
            let keys: Vec<(&str, usize, usize, String)> = vec![
                ("AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
                ("od_feature_flags[0..16]", od_base, 16, format!("{:?}", {t.od_feature_flags})),
                ("od_gfx_max / od_gfx_min", off("od_gfx"), 4, format!("{} / {} MHz (min @ 0x1E2)", {t.od_gfx_max[0]}, {t.od_gfx_min[0]})),
                ("od_mem_max / od_mem_min", off("od_mem"), 4, format!("{} / {} MHz (min @ 0x1FA)", {t.od_mem_max[0]}, {t.od_mem_min[0]})),
                ("od_pwr_max / od_pwr_min", off("od_pwr"), 4, format!("{} / {} % (min @ 0x202)", {t.od_pwr_max}, {t.od_pwr_min})),
                ("od_fan_max / od_fan_min", off("od_fan"), 4, format!("{} / {} RPM (min @ 0x1E6)", {t.od_fan_max}, {t.od_fan_min})),
                ("od_acoustic_max / min", off("od_acoustic"), 4, format!("{} / {} RPM (min @ 0x1F6)", {t.od_acoustic_max}, {t.od_acoustic_min})),
                ("od_mem_timing_ctrl", 0x116, 4, format!("{}", {t.od_mem_timing_ctrl})),
                ("od_pwr_mode_ctrl", 0x11A, 4, format!("{}", {t.od_pwr_mode_ctrl})),
                ("od_zero_rpm_ctrl", 0x11E, 4, format!("{}", {t.od_zero_rpm_ctrl})),
                ("pm_quiet_limit / temp", off("pm_quiet"), 8, format!("{}% / {}°C", {t.pm_quiet_limit}, {t.pm_quiet_temp})),
                ("pm_balanced_limit / temp", off("pm_balanced"), 8, format!("{}% / {}°C", {t.pm_balanced_limit}, {t.pm_balanced_temp})),
                ("pm_turbo_limit / temp", off("pm_turbo"), 8, format!("{}% / {}°C", {t.pm_turbo_limit}, {t.pm_turbo_temp})),
                ("pm_rage_limit / temp", off("pm_rage"), 8, format!("{}% / {}°C", {t.pm_rage_limit}, {t.pm_rage_temp})),
                ("smc_pptable_version", smc_base, 4, format!("0x{:08X}", u32::from_le_bytes([b[smc_base], b[smc_base+1], b[smc_base+2], b[smc_base+3]]))),
                ("features_to_run (u64)", 0x326, 8, format!("0x{:016X} (DPM_GFXCLK={}, VCN_PG={})", {t.features_to_run}, ({t.features_to_run} >> 1) & 1, ({t.features_to_run} >> 22) & 1)),
                ("socket_power_limit_ac", 0x32E, 2, format!("{} W", {t.socket_power_limit_ac[0]})),
                ("tdc_limit_gfx / soc", off("tdc_gfx"), 4, format!("{} A / {} A", {t.tdc_limit_gfx}, {t.tdc_limit_soc})),
                ("power_limit_ac[0..3]", off("pwr_limit_w"), 8, format!("{:?} W", {t.socket_power_limit_ac[0]})),
                ("ulv_voltage_vmin_gfx / soc", off("ulv_gfx"), 4, format!("{} mV / {} mV", {t.ulv_voltage_vmin_gfx} / 4, {t.ulv_voltage_vmin_soc} / 4)),
                ("min/max_voltage_gfx", off("volt_gfx"), 8, format!("{}..{} mV", {t.min_voltage_gfx} / 4, {t.max_voltage_gfx} / 4)),
                ("min/max_voltage_soc", off("volt_soc"), 8, format!("{}..{} mV", {t.min_voltage_soc} / 4, {t.max_voltage_soc} / 4)),
                ("tvmin_hyst / tvmin_gfx", off("tvmin_gfx"), 4, format!("{} / {} °C", {t.tvmin_hyst_gfx}, {t.tvmin_gfx})),
                ("vmin_low_gfx / soc", off("vmin_lh_gfx"), 4, format!("{} / {} mV", {t.vmin_low_gfx} / 4, {t.vmin_low_soc} / 4)),
                ("freq_gfx_min / max", 0x52E, 4, format!("{}..{} MHz (mirror @ 0x626 = {})", {t.freq_gfx_min}, {t.freq_gfx_max}, {t.freq_gfx_max_mirror})),
                ("freq_soc[0..1]", off("freq_soc"), 4, format!("{}..{} MHz", {t.freq_soc[0]}, {t.freq_soc[1]})),
                ("freq_fclk[0..1]", off("freq_fclk"), 4, format!("{}..{} MHz", {t.freq_fclk[0]}, {t.freq_fclk[1]})),
                ("freq_uclk[0..3]", off("mem_dpm_freq"), 8, format!("{:?} MHz", {t.freq_uclk})),
                ("mem_clk_div[0..3]", 0x65A, 4, format!("{:?}", {t.mem_clk_div})),
                ("mem_vddci[0..3]", off("mem_dpm_volt"), 8, format!("{:?} (raw 0.25mV)", {t.mem_vddci})),
                ("mem_mvdd[0..3]", off("mem_dpm_volt") + 8, 8, format!("{:?} (raw 0.25mV)", {t.mem_mvdd})),
                ("pcie_gen / lane / lclk", off("lclk") - 4, 8, format!("Gen {:?} | Lanes {:?} | LCLK {:?} MHz", {t.pcie_gen_speed}, {t.pcie_lane_count}, {t.lclk_freq})),
                ("fan_pwm_min / acoustic", off("fan_pwm_min"), 4, format!("{}% / {} RPM", {t.fan_pwm_min}, {t.fan_acoustic_limit})),
                ("fan_throttling / max_rpm", off("fan_throttling"), 4, format!("{} / {} RPM", {t.fan_throttling_rpm}, {t.fan_max_rpm})),
                ("fan_target_temp / gfx", off("fan_target_temp"), 4, format!("{}°C / {} MHz", {t.fan_target_temp}, {t.fan_target_gfx_clock})),
                ("fan_zero_rpm / stop / start", off("fan_zero_rpm"), 6, format!("en={}, stop={}°C, start={}°C", {t.fan_zero_rpm_enable}, {t.fan_stop_temp}, {t.fan_start_temp})),
                ("static_volt_gfx (a,b,c)", off("curve_gfx"), 12, format!("a={:.6}, b={:.6}, c={:.6}", {t.static_volt_gfx.a}, {t.static_volt_gfx.b}, {t.static_volt_gfx.c})),
                ("dcbtc_enable_gfx / soc", off("dcbtc_en"), 2, format!("{} / {}", {t.dcbtc_enable_gfx}, {t.dcbtc_enable_soc})),
                ("dcbtc_min / max / gb", off("dcbtc_min"), 12, format!("min={}/{}, max={}/{}, gb={}/{}", {t.dcbtc_min_gfx}/4, {t.dcbtc_min_soc}/4, {t.dcbtc_max_gfx}/4, {t.dcbtc_max_soc}/4, {t.dcbtc_gb_gfx}/4, {t.dcbtc_gb_soc}/4)),
                ("dctol_gfx / soc", off("dctol"), 4, format!("{} / {}", {t.dctol_gfx}, {t.dctol_soc})),
            ];

            for (name, offset, len, decoded) in keys {
                let end = (offset + len).min(b.len());
                let raw_hex: String = b[offset..end].iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                lines.push(format!("  @ 0x{:03X} ({:4}) | {:<26} | {:<23} -> {}", offset, offset, name, raw_hex, decoded));
            }
        } else {
            lines.push("  [Buffer smaller than 2470 bytes; showing header only]".to_string());
        }

        let report = lines.join("\n");
        ui.horizontal(|ui| {
            if ui.button("Copy Native Offset Debug").clicked() {
                ui.ctx().copy_text(report.clone());
            }
            ui.colored_label(egui::Color32::from_rgb(150, 165, 190), format!("{} bytes | od_base=0x0C2 | smc_base=0x322", b.len()));
        });
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(265.0).show(ui, |ui| {
            ui.monospace(report);
        });
    }
}
