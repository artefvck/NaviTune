use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn draw_debug_audit(&mut self, ui: &mut egui::Ui) {
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

        let avail_w = ui.available_width();
        let avail_h = ui.available_height();
        let col_w = ((avail_w - 60.0) / 6.0).max(80.0);
        egui::ScrollArea::both()
            .id_salt("debug_audit_full_scroll")
            .auto_shrink([false, false])
            .max_height(avail_h)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(avail_w, avail_h));
                egui::Grid::new("debug_audit_grid")
                    .num_columns(6)
                    .min_col_width(col_w)
                    .striped(true)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
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
}
