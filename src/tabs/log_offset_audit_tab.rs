use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn draw_debug_audit(&mut self, ui: &mut egui::Ui) {
        use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name as off2};
        use crate::pptable_rdna3::{AtomSmcDpmInfoV4_12, field_offset_by_name as off3};

        let (arch, rows): (&str, Vec<(&str, &str, usize, String, String)>) =
            if let Some(t3) = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
                let (od_g, od_m) = ({t3.od_gfx_max}, {t3.od_mem_max});
                let (od_p, od_f, od_a) = ({t3.od_pwr_max}, {t3.od_fan_max}, {t3.od_acoustic_max});
                let pwr_w = {t3.socket_power_limit_ac};
                let (tdc_g, tdc_s) = ({t3.tdc_limit_gfx}, {t3.tdc_limit_soc});
                let (vg0, vg1) = ({t3.min_voltage_gfx} / 4, {t3.max_voltage_gfx} / 4);
                let (vs0, vs1) = ({t3.min_voltage_soc} / 4, {t3.max_voltage_soc} / 4);
                let (ug, us) = ({t3.ulv_voltage_vmin_gfx} / 4, {t3.ulv_voltage_vmin_soc} / 4);
                let (vddci3, mvdd3) = ({t3.mem_vddci[3]} / 4, {t3.mem_mvdd[3]} / 4);
                let (fg0, fg1) = ({t3.freq_gfx_min}, {t3.freq_gfx_max});
                let fs1 = {t3.freq_soc_max};
                let (ff0, ff1) = ({t3.freq_fclk_min}, {t3.freq_fclk_max});
                let uclk3 = {t3.freq_uclk[3]};
                let (pcie_g0, pcie_g1) = ({t3.pcie_gen_speed[0]}, {t3.pcie_gen_speed[2]});
                let (pcie_l0, pcie_l1) = ({t3.pcie_lane_count[0]}, {t3.pcie_lane_count[2]});
                let (lclk0, lclk1) = ({t3.lclk_freq[0]}, {t3.lclk_freq[2]});
                let (f_pwm, f_max) = ({t3.fan_pwm_min}, {t3.fan_max_rpm});
                let (f_stop, f_start, f_tgt) = ({t3.fan_stop_temp}, {t3.fan_start_temp}, { let tt = t3.fan_target_temp; if tt > 200 { tt / 10 } else { tt } });
                ("RDNA 3 · SMU 13.0", vec![
                    ("OC Limits", "OD8 GFX Max (MHz)", off3("od_gfx"), format!("{}", od_g), format!("{}", self.oc_gfx_clock[1])),
                    ("OC Limits", "OD8 Mem Max (MHz)", off3("od_mem"), format!("{}", od_m), format!("{}", self.oc_mem_clock[1])),
                    ("OC Limits", "OD8 Power Max (%)", off3("od_pwr"), format!("{}", od_p), format!("{}", self.oc_power_limit[1])),
                    ("OC Limits", "OD8 Fan Max (RPM)", off3("od_fan"), format!("{}", od_f), format!("{}", self.oc_fan_rpm[1])),
                    ("OC Limits", "OD8 Acoustic Max (RPM)", off3("od_acoustic"), format!("{}", od_a), format!("{}", self.oc_fan_acoustic_rpm[1])),
                    ("Power", "Power Limit (W)", off3("pwr_limit_w"), format!("{}", pwr_w), format!("{}", self.power_limit_w)),
                    ("Power", "TDC GFX / Soc (A)", off3("tdc_gfx"), format!("{} / {}", tdc_g, tdc_s), format!("{} / {}", self.tdc_gfx, self.tdc_soc)),
                    ("Power", "GFX Vmin / Vmax (mV)", off3("volt_gfx"), format!("{} / {}", vg0, vg1), format!("{} / {}", self.volt_gfx[0], self.volt_gfx[1])),
                    ("Power", "Soc Vmin / Vmax (mV)", off3("volt_soc"), format!("{} / {}", vs0, vs1), format!("{} / {}", self.volt_soc[0], self.volt_soc[1])),
                    ("Power", "ULV GFX / Soc Vmin (mV)", off3("ulv_gfx"), format!("{} / {}", ug, us), format!("{} / {}", self.ulv_gfx[0], self.ulv_soc[0])),
                    ("Power", "Mem DPM3 VDDCI / MVDD", off3("mem_dpm_volt"), format!("{} / {}", vddci3, mvdd3), format!("{} / {}", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1])),
                    ("Frequency", "GFX Min / Max (MHz)", off3("freq_gfx"), format!("{} / {}", fg0, fg1), format!("{} / {}", self.freq_gfx[0], self.freq_gfx[1])),
                    ("Frequency", "Soc Max (MHz)", off3("freq_soc"), format!("{}", fs1), format!("{}", self.freq_soc[1])),
                    ("Frequency", "FCLK Min / Max (MHz)", off3("freq_fclk"), format!("{} / {}", ff0, ff1), format!("{} / {}", self.freq_fclk[0], self.freq_fclk[1])),
                    ("Frequency", "Mem DPM3 UCLK (MHz)", off3("mem_dpm_freq"), format!("{}", uclk3), format!("{}", self.mem_dpm_freq[3][0])),
                    ("Frequency", "PCIe Gen & Lanes", 0x810usize, format!("Gen {}/{} x{}/{}", pcie_g0, pcie_g1, pcie_l0, pcie_l1), format!("Gen {}/{} x{}/{}", self.pcie_gen_speed[0], self.pcie_gen_speed[1], self.pcie_lane_count[0], self.pcie_lane_count[1])),
                    ("Frequency", "LCLK Min / Max (MHz)", off3("lclk"), format!("{} / {}", lclk0, lclk1), format!("{} / {}", self.lclk_mhz[0], self.lclk_mhz[1])),
                    ("Fan", "PWM Min / Max RPM", off3("fan_pwm_min"), format!("{} / {}", f_pwm, f_max), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)),
                    ("Fan", "Stop / Start / Target", off3("fan_stop_temp"), format!("{} / {} / {}", f_stop, f_start, f_tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)),
                ])
            } else if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&self.raw_buffer) {
                let (od_min, od_max) = ({ t1.od_settings_min }, { t1.od_settings_max });
                let pwr_w = { t1.socket_power_limit_ac[0] };
                let (tdc_g, tdc_s) = ({ t1.tdc_limit_gfx }, { t1.tdc_limit_soc });
                let (vg_min, vg_max) = ({ t1.min_voltage_gfx } / 4, { t1.max_voltage_gfx } / 4);
                let (vs_min, vs_max) = ({ t1.min_voltage_soc } / 4, { t1.max_voltage_soc } / 4);
                let (ft_gfx, ft_uclk) = ({ t1.freq_table_gfx }, { t1.freq_table_uclk });
                let (f_pwm, f_max) = ({ t1.fan_pwm_min }, { t1.fan_maximum_rpm });
                let (f_stop, f_start, f_tgt) = ({ t1.fan_stop_temp }, { t1.fan_start_temp }, { t1.fan_target_temperature });
                ("RDNA 1 (V4_5)", vec![
                    ("OC Limits", "OD GFX Clock (Min/Max)", 0x02F, format!("{} .. {}", od_min[0], od_max[0]), format!("{} .. {}", self.oc_gfx_clock[0], self.oc_gfx_clock[1])),
                    ("OC Limits", "OD Mem Clock (Min/Max)", 0x033, format!("{} .. {}", od_min[1], od_max[1]), format!("{} .. {}", self.oc_mem_clock[0], self.oc_mem_clock[1])),
                    ("Power", "PPT0 Limit (W)", 0x0B7, format!("{}", pwr_w), format!("{}", self.power_limit_w)),
                    ("Power", "TDC GFX / Soc (A)", 0x0D7, format!("{} / {}", tdc_g, tdc_s), format!("{} / {}", self.tdc_gfx, self.tdc_soc)),
                    ("Power", "GFX Voltage (mV)", 0x0F3, format!("{} .. {}", vg_min, vg_max), format!("{} .. {}", self.volt_gfx[0], self.volt_gfx[1])),
                    ("Power", "Soc Voltage (mV)", 0x0F5, format!("{} .. {}", vs_min, vs_max), format!("{} .. {}", self.volt_soc[0], self.volt_soc[1])),
                    ("Frequency", "GFXCLK Min / Max (MHz)", 0x0FF, format!("{} .. {}", ft_gfx[0], ft_gfx[15]), format!("{} .. {}", self.freq_gfx[0], self.freq_gfx[1])),
                    ("Frequency", "UCLK DPM 0..3 (MHz)", 0x14F, format!("{}/{}/{}/{}", ft_uclk[0], ft_uclk[1], ft_uclk[2], ft_uclk[3]), format!("{}/{}/{}/{}", self.mem_dpm_freq[0][0], self.mem_dpm_freq[1][0], self.mem_dpm_freq[2][0], self.mem_dpm_freq[3][0])),
                    ("Fan", "PWM Min / Max RPM", 0x68B, format!("{} / {}", f_pwm, f_max), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)),
                    ("Fan", "Stop / Start / Target", 0x679, format!("{} / {} / {}", f_stop, f_start, f_tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)),
                ])
            } else if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(&self.raw_buffer) {
                let (od_min, od_max) = ({ v20.od_settings_min }, { v20.od_settings_max });
                let pwr_w = { v20.socket_power_limit_ac[0] };
                let (tdc_g, tdc_s) = ({ v20.tdc_limit_gfx }, { v20.tdc_limit_soc });
                let (vg_min, vg_max) = ({ v20.min_voltage_gfx } / 4, { v20.max_voltage_gfx } / 4);
                let (vs_min, vs_max) = ({ v20.min_voltage_soc } / 4, { v20.max_voltage_soc } / 4);
                let (ft_gfx, ft_uclk) = ({ v20.freq_table_gfx }, { v20.freq_table_uclk });
                let (f_pwm, f_max) = ({ v20.fan_pwm_min }, { v20.fan_maximum_rpm });
                let (f_stop, f_start, f_tgt) = ({ v20.fan_stop_temp }, { v20.fan_start_temp }, { v20.fan_target_temperature });
                ("Vega 20 (0x0B)", vec![
                    ("OC Limits", "OD GFX Clock (Min/Max)", 0x055, format!("{} .. {}", od_min[0], od_max[0]), format!("{} .. {}", self.oc_gfx_clock[0], self.oc_gfx_clock[1])),
                    ("OC Limits", "OD Mem Clock (Min/Max)", 0x059, format!("{} .. {}", od_min[1], od_max[1]), format!("{} .. {}", self.oc_mem_clock[0], self.oc_mem_clock[1])),
                    ("Power", "PPT0 Limit (W)", 0x11D, format!("{}", pwr_w), format!("{}", self.power_limit_w)),
                    ("Power", "TDC GFX / Soc (A)", 0x13D, format!("{} / {}", tdc_g, tdc_s), format!("{} / {}", self.tdc_gfx, self.tdc_soc)),
                    ("Power", "GFX Voltage (mV)", 0x159, format!("{} .. {}", vg_min, vg_max), format!("{} .. {}", self.volt_gfx[0], self.volt_gfx[1])),
                    ("Power", "Soc Voltage (mV)", 0x15B, format!("{} .. {}", vs_min, vs_max), format!("{} .. {}", self.volt_soc[0], self.volt_soc[1])),
                    ("Frequency", "GFXCLK Min / Max (MHz)", 0x165, format!("{} .. {}", ft_gfx[0], ft_gfx[15]), format!("{} .. {}", self.freq_gfx[0], self.freq_gfx[1])),
                    ("Frequency", "UCLK DPM 0..3 (MHz)", 0x1B5, format!("{}/{}/{}/{}", ft_uclk[0], ft_uclk[1], ft_uclk[2], ft_uclk[3]), format!("{}/{}/{}/{}", self.mem_dpm_freq[0][0], self.mem_dpm_freq[1][0], self.mem_dpm_freq[2][0], self.mem_dpm_freq[3][0])),
                    ("Fan", "PWM Min / Max RPM", 0x675, format!("{} / {}", f_pwm, f_max), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)),
                    ("Fan", "Stop / Start / Target", 0x663, format!("{} / {} / {}", f_stop, f_start, f_tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)),
                ])
            } else if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(&self.raw_buffer) {
                let fan_off = { v10.fan_table_offset } as usize;
                let mut v10_rows = vec![
                    ("OC Limits", "Max OD Engine Clock (MHz)", 0x017, format!("{}", { v10.max_od_engine_clock } / 100), format!("{}", self.oc_gfx_clock[1])),
                    ("OC Limits", "Max OD Memory Clock (MHz)", 0x01B, format!("{}", { v10.max_od_memory_clock } / 100), format!("{}", self.oc_mem_clock[1])),
                    ("OC Limits", "Power Control Limit (+%)", 0x01F, format!("{}", { v10.power_control_limit }), format!("{}", self.oc_power_limit[1])),
                ];
                let hard_off = { v10.hard_limit_table_offset } as usize;
                if let Some((soc_hi, gfx_hi, mem_hi, vddc_hi)) = v10.read_hard_limits(&self.raw_buffer) {
                    v10_rows.push(("HardLimit SubTable", "GFXCLK / MCLK Max (MHz)", hard_off + 0x06, format!("{} / {}", gfx_hi, mem_hi), format!("{} / {}", self.freq_gfx[1], self.mem_dpm_freq[3][0])));
                    v10_rows.push(("HardLimit SubTable", "SOCCLK Max / VDDC Max", hard_off + 0x02, format!("{}MHz / {}mV", soc_hi, vddc_hi), format!("{}MHz / {}mV", self.freq_soc[1], self.volt_gfx[1])));
                    if let Some((vddci, mvdd)) = v10.read_hard_limit_mem_volt(&self.raw_buffer) {
                        v10_rows.push(("HardLimit SubTable", "VDDCI / MVDD (mV)", hard_off + 0x12, format!("{} / {}", vddci, mvdd), format!("{} / {}", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1])));
                    }
                }
                if let Some((pwm, rpm, tgt, _gfx, _zrpm, stp, strt)) = v10.read_fan_subtable(&self.raw_buffer) {
                    v10_rows.push(("Fan SubTable", "PWM Min / Max RPM", fan_off + 0x09, format!("{} / {}", pwm, rpm), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)));
                    v10_rows.push(("Fan SubTable", "Stop / Start / Target", fan_off + 0x0F, format!("{} / {} / {}", stp, strt, tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)));
                }
                ("Vega 10 (0x08)", v10_rows)
            } else if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&self.raw_buffer) {
                let hard_off = { pol.hard_limit_table_offset } as usize;
                let mut rows = vec![
                    ("OC Limits", "Max OD Engine Clock (MHz)", 0x017, format!("{}", { pol.max_od_engine_clock } / 100), format!("{}", self.oc_gfx_clock[1])),
                    ("OC Limits", "Max OD Memory Clock (MHz)", 0x01B, format!("{}", { pol.max_od_memory_clock } / 100), format!("{}", self.oc_mem_clock[1])),
                    ("OC Limits", "Power Control Limit (+%)", 0x01F, format!("{}", { pol.power_control_limit }), format!("{}", self.oc_power_limit[1])),
                ];
                if let Some((gfx_hi, mem_hi, vddc_hi)) = pol.read_hard_limits(&self.raw_buffer) {
                    rows.push(("HardLimit SubTable", "GFXCLK / MCLK / VDDC", hard_off + 0x02, format!("{} / {}MHz / {}mV", gfx_hi, mem_hi, vddc_hi), format!("{} / {}MHz / {}mV", self.freq_gfx[1], self.mem_dpm_freq[3][0], self.volt_gfx[1])));
                    if let Some((vddci, mvdd)) = pol.read_hard_limit_mem_volt(&self.raw_buffer) {
                        rows.push(("HardLimit SubTable", "VDDCI / MVDD (mV)", hard_off + 0x0E, format!("{} / {}", vddci, mvdd), format!("{} / {}", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1])));
                    }
                }
                ("Polaris (0x07)", rows)
            } else if let Some(t) = AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer) {
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
                ("RDNA 2 · SMU 11.0", vec![
                    ("OC Limits", "OD8 GFX Min / Max", off2("od_gfx"), format!("{} / {}", od_gfx_min, od_gfx_max), format!("{} / {}", self.oc_gfx_clock[0], self.oc_gfx_clock[1])),
                    ("OC Limits", "OD8 Mem Min / Max", off2("od_mem"), format!("{} / {}", od_mem_min, od_mem_max), format!("{} / {}", self.oc_mem_clock[0], self.oc_mem_clock[1])),
                    ("OC Limits", "OD8 Pwr Min / Max", off2("od_pwr"), format!("{} / {}", od_pwr_min, od_pwr_max), format!("{} / {}", self.oc_power_limit[0], self.oc_power_limit[1])),
                    ("OC Limits", "OD8 Fan Min / Max", off2("od_fan"), format!("{} / {}", od_fan_min, od_fan_max), format!("{} / {}", self.oc_fan_rpm[0], self.oc_fan_rpm[1])),
                    ("OC Limits", "OD8 Acoustic Min / Max", off2("od_acoustic"), format!("{} / {}", od_ac_min, od_ac_max), format!("{} / {}", self.oc_fan_acoustic_rpm[0], self.oc_fan_acoustic_rpm[1])),
                    ("OC Limits", "PM Quiet Limit / Temp", off2("pm_quiet"), format!("{} / {}", pm_q_l, pm_q_t), format!("{} / {}", self.pm_quiet[0], self.pm_quiet[1])),
                    ("OC Limits", "PM Balanced Limit / Temp", off2("pm_balanced"), format!("{} / {}", pm_b_l, pm_b_t), format!("{} / {}", self.pm_balanced[0], self.pm_balanced[1])),
                    ("OC Limits", "PM Turbo Limit / Temp", off2("pm_turbo"), format!("{} / {}", pm_t_l, pm_t_t), format!("{} / {}", self.pm_turbo[0], self.pm_turbo[1])),
                    ("OC Limits", "PM Rage Limit / Temp", off2("pm_rage"), format!("{} / {}", pm_r_l, pm_r_t), format!("{} / {}", self.pm_rage[0], self.pm_rage[1])),
                    ("Power", "Power Limit (W)", off2("pwr_limit_w"), format!("{}", pwr_w), format!("{}", self.power_limit_w)),
                    ("Power", "TDC GFX / Soc (A)", off2("tdc_gfx"), format!("{} / {}", tdc_g, tdc_s), format!("{} / {}", self.tdc_gfx, self.tdc_soc)),
                    ("Power", "GFX Vmin / Vmax (mV)", off2("volt_gfx"), format!("{} / {}", vg_min, vg_max), format!("{} / {}", self.volt_gfx[0], self.volt_gfx[1])),
                    ("Power", "Soc Vmin / Vmax (mV)", off2("volt_soc"), format!("{} / {}", vs_min, vs_max), format!("{} / {}", self.volt_soc[0], self.volt_soc[1])),
                    ("Power", "ULV GFX Vmin / Offset", off2("ulv_gfx"), format!("{} / {}", ug_min, ug_off), format!("{} / {}", self.ulv_gfx[0], self.ulv_gfx[1])),
                    ("Power", "ULV Soc Vmin / Offset", off2("ulv_soc"), format!("{} / {}", us_min, us_off), format!("{} / {}", self.ulv_soc[0], self.ulv_soc[1])),
                    ("Power", "Mem DPM3 VDDCI / MVDD", off2("mem_dpm_volt"), format!("{} / {}", vddci3, mvdd3), format!("{} / {}", self.mem_dpm_volt[3][0], self.mem_dpm_volt[3][1])),
                    ("Frequency", "GFX Min / Max (MHz)", off2("freq_gfx"), format!("{} / {}", fg_min, fg_max), format!("{} / {}", self.freq_gfx[0], self.freq_gfx[1])),
                    ("Frequency", "Soc Min / Max (MHz)", off2("freq_soc"), format!("{} / {}", fs_min, fs_max), format!("{} / {}", self.freq_soc[0], self.freq_soc[1])),
                    ("Frequency", "FCLK Min / Max (MHz)", off2("freq_fclk"), format!("{} / {}", ff_min, ff_max), format!("{} / {}", self.freq_fclk[0], self.freq_fclk[1])),
                    ("Frequency", "Mem DPM3 Freq / Div", off2("mem_dpm_freq"), format!("{} / {}", uclk3, div3), format!("{} / {}", self.mem_dpm_freq[3][0], self.mem_dpm_freq[3][1])),
                    ("Frequency", "PCIe Gen & Lanes", off2("lclk") - 4, format!("Gen {}/{} x{}/{}", pcie_g0, pcie_g1, pcie_l0, pcie_l1), format!("Gen {}/{} x{}/{}", self.pcie_gen_speed[0], self.pcie_gen_speed[1], self.pcie_lane_count[0], self.pcie_lane_count[1])),
                    ("Frequency", "LCLK Min / Max (MHz)", off2("lclk"), format!("{} / {}", lclk0, lclk1), format!("{} / {}", self.lclk_mhz[0], self.lclk_mhz[1])),
                    ("Curve", "Static Curve GFX c", off2("curve_gfx"), format!("{:.6}", crv_c), format!("{:.6}", self.static_volt_c[0])),
                    ("Curve", "DcTol GFX / Soc", off2("dctol"), format!("{} / {}", dctol_g, dctol_s), format!("{} / {}", self.dctol[0], self.dctol[1])),
                    ("Curve", "DcBtc Max GFX / Soc", off2("dcbtc_max"), format!("{:.0} / {:.0}", dcb_max_g, dcb_max_s), format!("{:.0} / {:.0}", self.dcbtc_max[0], self.dcbtc_max[1])),
                    ("Curve", "DcBtc GB GFX / Soc", off2("dcbtc_gb"), format!("{:.0} / {:.0}", dcb_gb_g, dcb_gb_s), format!("{:.0} / {:.0}", self.dcbtc_gb[0], self.dcbtc_gb[1])),
                    ("Fan", "PWM Min / Max RPM", off2("fan_pwm_min"), format!("{} / {}", f_pwm, f_max), format!("{} / {}", self.fan_pwm_min, self.fan_max_rpm)),
                    ("Fan", "Stop / Start / Target °C", off2("fan_stop_temp"), format!("{} / {} / {}", f_stop, f_start, f_tgt), format!("{} / {} / {}", self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp)),
                    ("Fan", "OD Temp / Speed Min", off2("od_fan_temp"), format!("{} / {}", od_ft, od_fs), format!("{} / {}", self.od_fan_temp[0], self.od_fan_speed[0])),
                ])
            } else {
                ui.weak("No table loaded");
                return;
            };

        let ok_count = rows.iter().filter(|r| r.3 == r.4).count();
        let total_count = rows.len();

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.debug_only_mismatch, "Show only mismatches (DIFF)");
            ui.separator();
            ui.colored_label(
                egui::Color32::from_rgb(95, 200, 145),
                format!("{} · {}/{} OK", arch, ok_count, total_count),
            );
        });
        ui.add_space(4.0);

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
