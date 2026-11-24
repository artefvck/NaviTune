
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(super) fn try_apply_rdna1(&mut self, b: &[u8], label: &str, frev: u8, crev: u8, trev: u8) -> bool {
        if let Some(r1p) = crate::pptable_rdna1::Rdna1ParsedState::parse(b) {
            self.raw_buffer = b.to_owned();
            for (i, feature) in self.smu_features.iter_mut().enumerate() {
                *feature = ((r1p.smu_features >> i) & 1) == 1;
            }
            self.power_limit_w = r1p.power_limit_w as i32;
            self.tdc_soc = r1p.tdc_soc as i32;
            self.tdc_gfx = r1p.tdc_gfx as i32;
            self.ulv_gfx = [r1p.ulv_vmin_gfx as i32 / 4, 0];
            self.ulv_soc = [r1p.ulv_vmin_soc as i32 / 4, 0];
            self.volt_gfx = [r1p.min_volt_gfx as i32 / 4, r1p.max_volt_gfx as i32 / 4];
            self.volt_soc = [r1p.min_volt_soc as i32 / 4, r1p.max_volt_soc as i32 / 4];
            self.freq_gfx = [r1p.gfx_min as i32, r1p.gfx_max as i32];
            self.freq_soc = [r1p.soc_min as i32, r1p.soc_max as i32];
            self.freq_fclk = [r1p.fclk_min as i32, r1p.fclk_max as i32];
            for i in 0..4 {
                self.mem_dpm_freq[i] = [r1p.uclk_dpm[i] as i32, 1];
                self.mem_dpm_volt[i] = [r1p.mem_vddci[i] as i32 / 4, r1p.mem_mvdd[i] as i32 / 4];
            }
            for i in 0..7 { self.temp_limits[i] = r1p.temp_limit[i] as i32; }
            self.fan_stop_temp = r1p.fan_stop_temp as i32;
            self.fan_start_temp = r1p.fan_start_temp as i32;
            self.fan_pwm_min = r1p.fan_pwm_min as i32;
            self.fan_max_rpm = r1p.fan_max_rpm as i32;
            self.fan_target_temp = r1p.fan_target_temp as i32;
            let f = r1p.od_feature_flags;
            self.gpu_clock_limit = f[0] != 0;
            self.gpu_clock_curve = f[1] != 0;
            self.memory_clock_limit = f[2] != 0;
            self.zero_rpm = f[3] != 0;
            self.min_fan_speed = f[4] != 0;
            self.fan_target_temp_limit = f[5] != 0;
            self.operating_temp_limit = f[6] != 0;
            self.fan_acoustic_rpm = f[7] != 0;
            self.fan_curve = f[8] != 0;
            self.power_limit_feat = f[9] != 0;
            self.timing_tuning = f[10] != 0;
            self.power_mode = f[11] != 0;
            self.auto_undervolt_gpu = f[12] != 0;
            self.auto_overclock_gpu = f[13] != 0;
            self.auto_overclock_mem = f[14] != 0;
            self.auto_fan_acoustic_limit = f[15] != 0;

            self.oc_gfx_clock = [r1p.od_settings_min[0] as i32, r1p.od_settings_max[0] as i32];
            self.oc_mem_clock = [r1p.od_settings_min[1] as i32, r1p.od_settings_max[1] as i32];
            let pwr_min_raw = r1p.od_settings_min[2] as i32;
            let pwr_max_raw = r1p.od_settings_max[2] as i32;
            self.oc_power_limit = [if pwr_min_raw > pwr_max_raw { -pwr_min_raw } else { pwr_min_raw }, pwr_max_raw];
            self.oc_fan_rpm = [r1p.od_settings_min[3] as i32, r1p.od_settings_max[3] as i32];
            self.oc_fan_acoustic_rpm = [r1p.od_settings_min[4] as i32, r1p.od_settings_max[4] as i32];
            self.fan_acoustic_limit = r1p.fan_acoustic_rpm as i32;
            self.fan_throttling_rpm = r1p.fan_throttling_rpm as i32;
            self.fan_target_gfx_clock = r1p.fan_target_gfx as i32;
            self.fan_zero_rpm_enable = r1p.fan_zero_rpm;
            self.static_volt_a[0] = r1p.curve_gfx[0] as f64;
            self.static_volt_b[0] = r1p.curve_gfx[1] as f64;
            self.static_volt_c[0] = r1p.curve_gfx[2] as f64;
            self.baseline_buffer = b.to_owned();

            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_name = crate::pptable_rdna1::NaviRDNA1Chip::from_did_and_clock(did, r1p.od_settings_max[0]).display_name().to_string();
            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PP_PhmSoftPowerPlayTable @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, chip_name, frev, crev, trev, b.len(), vbios_tag
            );
            return true;
        }
        if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip1 = t1.detect_chip(did);

            let f = { t1.od_feature_flags };
            self.gpu_clock_limit = f[0] != 0;
            self.gpu_clock_curve = f[1] != 0;
            self.memory_clock_limit = f[2] != 0;
            self.zero_rpm = f[3] != 0;
            self.min_fan_speed = f[4] != 0;
            self.fan_target_temp_limit = f[5] != 0;
            self.operating_temp_limit = f[6] != 0;
            self.fan_acoustic_rpm = f[7] != 0;
            self.fan_curve = f[8] != 0;
            self.power_limit_feat = f[9] != 0;
            self.timing_tuning = f[10] != 0;
            self.power_mode = f[11] != 0;

            let mask = { t1.features_to_run };
            for (bit, feature) in self.smu_features.iter_mut().enumerate() {
                *feature = (mask & (1u64 << bit)) != 0;
            }

            let od_min = { t1.od_settings_min };
            let od_max = { t1.od_settings_max };
            self.oc_gfx_clock = [od_min[0] as i32, od_max[0] as i32];
            self.oc_mem_clock = [od_min[1] as i32, od_max[1] as i32];
            self.oc_power_limit = [od_min[2] as i32, od_max[2] as i32];
            self.oc_fan_rpm = [od_min[3] as i32, od_max[3] as i32];
            self.oc_fan_acoustic_rpm = [od_min[4] as i32, od_max[4] as i32];

            let ppt0 = { t1.socket_power_limit_ac[0] } as i32;
            self.power_limit_w = if ppt0 > 0 { ppt0 } else { ({ t1.boost_power_limit }) as i32 };
            self.tdc_gfx = { t1.tdc_limit_gfx } as i32;
            self.tdc_soc = { t1.tdc_limit_soc } as i32;

            self.volt_gfx = [{ t1.min_voltage_gfx } as i32 / 4, { t1.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ t1.min_voltage_soc } as i32 / 4, { t1.max_voltage_soc } as i32 / 4];
            self.ulv_gfx = [{ t1.min_voltage_ulv_gfx } as i32 / 4, { t1.ulv_voltage_offset_gfx } as i32 / 4];
            self.ulv_soc = [{ t1.min_voltage_ulv_soc } as i32 / 4, { t1.ulv_voltage_offset_soc } as i32 / 4];

            let desc = { t1.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let dclk_n = (desc[4] as usize).clamp(1, 8);
            let vclk_n = (desc[5] as usize).clamp(1, 8);
            let dcef_n = (desc[6] as usize).clamp(1, 8);
            let disp_n = (desc[7] as usize).clamp(1, 8);
            let pix_n = (desc[8] as usize).clamp(1, 8);
            let phy_n = (desc[9] as usize).clamp(1, 8);

            let ft_gfx = { t1.freq_table_gfx };
            let ft_soc = { t1.freq_table_socclk };
            let ft_uclk = { t1.freq_table_uclk };
            let ft_fclk = { t1.freq_table_fclk };
            let ft_vclk = { t1.freq_table_vclk };
            let ft_dclk = { t1.freq_table_dclk };
            let ft_dcef = { t1.freq_table_dcefclk };
            let ft_disp = { t1.freq_table_dispclk };
            let ft_pix = { t1.freq_table_pixclk };
            let ft_phy = { t1.freq_table_phyclk };

            self.freq_gfx = [ft_gfx[0] as i32, ft_gfx[gfx_n - 1] as i32];
            self.freq_soc = [ft_soc[0] as i32, ft_soc[soc_n - 1] as i32];
            self.freq_fclk = [ft_fclk[0] as i32, ft_fclk[fclk_n - 1] as i32];
            self.freq_vclk = [ft_vclk[0] as i32, ft_vclk[vclk_n - 1] as i32];
            self.freq_dclk = [ft_dclk[0] as i32, ft_dclk[dclk_n - 1] as i32];
            self.freq_dcefclk = [ft_dcef[0] as i32, ft_dcef[dcef_n - 1] as i32];
            self.freq_dispclk = [ft_disp[0] as i32, ft_disp[disp_n - 1] as i32];
            self.freq_pixclk = [ft_pix[0] as i32, ft_pix[pix_n - 1] as i32];
            self.freq_phyclk = [ft_phy[0] as i32, ft_phy[phy_n - 1] as i32];

            let vddci_hi = { t1.vddci_max } as i32 / 4;
            let mvdd_hi = { t1.mvdd_max } as i32 / 4;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [ft_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [vddci_hi, mvdd_hi];
            }

            self.fan_pwm_min = { t1.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { t1.fan_acoustic_limit_rpm } as i32;
            self.fan_throttling_rpm = { t1.fan_throttling_rpm } as i32;
            self.fan_max_rpm = { t1.fan_maximum_rpm } as i32;
            self.fan_target_temp = { t1.fan_target_temperature } as i32;
            self.fan_target_gfx_clock = { t1.fan_target_gfxclk } as i32;
            self.fan_zero_rpm_enable = t1.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { t1.fan_stop_temp } as i32;
            self.fan_start_temp = { t1.fan_start_temp } as i32;

            let tl = { t1.temp_limit };
            for i in 0..7 {
                self.temp_limits[i] = tl[i] as i32;
            }

            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PowerPlayInfo @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, chip1.display_name(), frev, crev, trev, b.len(), vbios_tag
            );
            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PP_PhmSoftPowerPlayTable @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [RDNA 1 (Navi 10 / Navi 14) | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, frev, crev, trev, b.len(), vbios_tag
            );
            self.baseline_buffer = b.to_owned();
            self.raw_buffer = b.to_owned();
            return true;
        }

        false
    }
    pub(super) fn try_sync_rdna1(&mut self) -> bool {
        if let Some(mut r1p) = crate::pptable_rdna1::Rdna1ParsedState::parse(&self.raw_buffer) {
            let qmv = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
            let mut mask = 0u64;
            for (i, &enabled) in self.smu_features.iter().enumerate() { if enabled { mask |= 1u64 << i; } }
            r1p.smu_features = mask;
            r1p.power_limit_w = self.power_limit_w.max(0) as u16;
            r1p.tdc_soc = self.tdc_soc.max(0) as u16;
            r1p.tdc_gfx = self.tdc_gfx.max(0) as u16;
            r1p.ulv_vmin_gfx = qmv(self.ulv_gfx[0], r1p.ulv_vmin_gfx);
            r1p.ulv_vmin_soc = qmv(self.ulv_soc[0], r1p.ulv_vmin_soc);
            r1p.min_volt_gfx = qmv(self.volt_gfx[0], r1p.min_volt_gfx);
            r1p.max_volt_gfx = qmv(self.volt_gfx[1], r1p.max_volt_gfx);
            r1p.min_volt_soc = qmv(self.volt_soc[0], r1p.min_volt_soc);
            r1p.max_volt_soc = qmv(self.volt_soc[1], r1p.max_volt_soc);
            r1p.gfx_min = self.freq_gfx[0].max(0) as u16;
            r1p.gfx_max = self.freq_gfx[1].max(0) as u16;
            r1p.soc_min = self.freq_soc[0].max(0) as u16;
            r1p.soc_max = self.freq_soc[1].max(0) as u16;
            for i in 0..4 {
                r1p.uclk_dpm[i] = self.mem_dpm_freq[i][0].max(0) as u16;
                r1p.mem_vddci[i] = qmv(self.mem_dpm_volt[i][0], r1p.mem_vddci[i]);
                r1p.mem_mvdd[i] = qmv(self.mem_dpm_volt[i][1], r1p.mem_mvdd[i]);
            }
            r1p.fclk_min = self.freq_fclk[0].max(0) as u16;
            r1p.fclk_max = self.freq_fclk[1].max(0) as u16;
            for i in 0..7 { r1p.temp_limit[i] = self.temp_limits[i].max(0) as u16; }
            r1p.fan_stop_temp = self.fan_stop_temp.max(0) as u16;
            r1p.fan_start_temp = self.fan_start_temp.max(0) as u16;
            r1p.fan_pwm_min = self.fan_pwm_min.max(0) as u16;
            r1p.fan_max_rpm = self.fan_max_rpm.max(0) as u16;
            r1p.fan_target_temp = self.fan_target_temp.max(0) as u16;
            r1p.od_settings_min = [
                self.oc_gfx_clock[0].max(0) as u32,
                self.oc_mem_clock[0].max(0) as u32,
                if self.oc_power_limit[0] < 0 { self.oc_power_limit[0].unsigned_abs() } else { self.oc_power_limit[0] as u32 },
                self.oc_fan_rpm[0].max(0) as u32,
                self.oc_fan_acoustic_rpm[0].max(0) as u32,
            ];
            r1p.od_settings_max = [
                self.oc_gfx_clock[1].max(0) as u32,
                self.oc_mem_clock[1].max(0) as u32,
                self.oc_power_limit[1] as u32,
                self.oc_fan_rpm[1].max(0) as u32,
                self.oc_fan_acoustic_rpm[1].max(0) as u32,
            ];
            r1p.od_gfx_max = r1p.od_settings_max[0];
            r1p.od_uclk_max = r1p.od_settings_max[1];
            r1p.od_feature_flags = [
                u8::from(self.gpu_clock_limit), u8::from(self.gpu_clock_curve),
                u8::from(self.memory_clock_limit), u8::from(self.zero_rpm),
                u8::from(self.min_fan_speed), u8::from(self.fan_target_temp_limit),
                u8::from(self.operating_temp_limit), u8::from(self.fan_acoustic_rpm),
                u8::from(self.fan_curve), u8::from(self.power_limit_feat),
                u8::from(self.timing_tuning), u8::from(self.power_mode),
                u8::from(self.auto_undervolt_gpu), u8::from(self.auto_overclock_gpu),
                u8::from(self.auto_overclock_mem), u8::from(self.auto_fan_acoustic_limit),
            ];
            r1p.write_to_buf(&mut self.raw_buffer);
            return true;
        }
        if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Rdna1 && self.raw_buffer.len() >= 1674 {
            let orig_len = self.raw_buffer.len();
            let mut padded = vec![0u8; orig_len.max(1924)];
            padded[..orig_len].copy_from_slice(&self.raw_buffer);
            if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes_mut(&mut padded) {
            t1.boost_power_limit = self.power_limit_w.max(0) as u16;
            t1.socket_power_limit_ac[0] = self.power_limit_w.max(0) as u16;
            t1.tdc_limit_gfx = self.tdc_gfx.max(0) as u16;
            t1.tdc_limit_soc = self.tdc_soc.max(0) as u16;

            t1.min_voltage_gfx = (self.volt_gfx[0].max(0) * 4) as u16;
            t1.max_voltage_gfx = (self.volt_gfx[1].max(0) * 4) as u16;
            t1.min_voltage_soc = (self.volt_soc[0].max(0) * 4) as u16;
            t1.max_voltage_soc = (self.volt_soc[1].max(0) * 4) as u16;
            let qmv_u = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
            let qmv_i = |ui_mv: i32, orig: i16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv * 4) as i16 };
            t1.min_voltage_ulv_gfx = qmv_u(self.ulv_gfx[0], t1.min_voltage_ulv_gfx);
            t1.min_voltage_ulv_soc = qmv_u(self.ulv_soc[0], t1.min_voltage_ulv_soc);
            t1.ulv_voltage_offset_gfx = qmv_i(self.ulv_gfx[1], t1.ulv_voltage_offset_gfx);
            t1.ulv_voltage_offset_soc = qmv_i(self.ulv_soc[1], t1.ulv_voltage_offset_soc);

            t1.od_settings_min[0] = self.oc_gfx_clock[0].max(0) as u32;
            t1.od_settings_max[0] = self.oc_gfx_clock[1].max(0) as u32;
            t1.od_settings_min[1] = self.oc_mem_clock[0].max(0) as u32;
            t1.od_settings_max[1] = self.oc_mem_clock[1].max(0) as u32;
            t1.od_settings_min[2] = self.oc_power_limit[0] as u32;
            t1.od_settings_max[2] = self.oc_power_limit[1].max(0) as u32;
            t1.od_settings_min[3] = self.oc_fan_rpm[0].max(0) as u32;
            t1.od_settings_max[3] = self.oc_fan_rpm[1].max(0) as u32;
            t1.od_settings_min[4] = self.oc_fan_acoustic_rpm[0].max(0) as u32;
            t1.od_settings_max[4] = self.oc_fan_acoustic_rpm[1].max(0) as u32;
            t1.od_settings_max[5] = self.fan_target_temp.max(0) as u32;

            let desc = { t1.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let dclk_n = (desc[4] as usize).clamp(1, 8);
            let vclk_n = (desc[5] as usize).clamp(1, 8);
            let dcef_n = (desc[6] as usize).clamp(1, 8);
            let disp_n = (desc[7] as usize).clamp(1, 8);
            let pix_n = (desc[8] as usize).clamp(1, 8);
            let phy_n = (desc[9] as usize).clamp(1, 8);

            t1.freq_table_gfx[0] = self.freq_gfx[0].max(0) as u16;
            t1.freq_table_gfx[gfx_n - 1] = self.freq_gfx[1].max(0) as u16;
            t1.freq_table_socclk[0] = self.freq_soc[0].max(0) as u16;
            t1.freq_table_socclk[soc_n - 1] = self.freq_soc[1].max(0) as u16;
            t1.freq_table_fclk[0] = self.freq_fclk[0].max(0) as u16;
            t1.freq_table_fclk[fclk_n - 1] = self.freq_fclk[1].max(0) as u16;
            t1.freq_table_vclk[0] = self.freq_vclk[0].max(0) as u16;
            t1.freq_table_vclk[vclk_n - 1] = self.freq_vclk[1].max(0) as u16;
            t1.freq_table_dclk[0] = self.freq_dclk[0].max(0) as u16;
            t1.freq_table_dclk[dclk_n - 1] = self.freq_dclk[1].max(0) as u16;
            t1.freq_table_dcefclk[0] = self.freq_dcefclk[0].max(0) as u16;
            t1.freq_table_dcefclk[dcef_n - 1] = self.freq_dcefclk[1].max(0) as u16;
            t1.freq_table_dispclk[0] = self.freq_dispclk[0].max(0) as u16;
            t1.freq_table_dispclk[disp_n - 1] = self.freq_dispclk[1].max(0) as u16;
            t1.freq_table_pixclk[0] = self.freq_pixclk[0].max(0) as u16;
            t1.freq_table_pixclk[pix_n - 1] = self.freq_pixclk[1].max(0) as u16;
            t1.freq_table_phyclk[0] = self.freq_phyclk[0].max(0) as u16;
            t1.freq_table_phyclk[phy_n - 1] = self.freq_phyclk[1].max(0) as u16;

            for i in 0..4 {
                t1.freq_table_uclk[i] = self.mem_dpm_freq[i][0].max(0) as u16;
            }
            t1.vddci_max = (self.mem_dpm_volt[3][0].max(0) * 4) as u16;
            t1.mvdd_max = (self.mem_dpm_volt[3][1].max(0) * 4) as u16;

            t1.fan_pwm_min = self.fan_pwm_min.max(0) as u16;
            t1.fan_acoustic_limit_rpm = self.fan_acoustic_limit.max(0) as u16;
            t1.fan_throttling_rpm = self.fan_throttling_rpm.max(0) as u16;
            t1.fan_maximum_rpm = self.fan_max_rpm.max(0) as u16;
            t1.fan_target_temperature = self.fan_target_temp.max(0) as u16;
            t1.fan_target_gfxclk = self.fan_target_gfx_clock.max(0) as u16;
            t1.fan_zero_rpm_enable = u8::from(self.fan_zero_rpm_enable);
            t1.fan_stop_temp = self.fan_stop_temp.max(0) as u16;
            t1.fan_start_temp = self.fan_start_temp.max(0) as u16;

            for i in 0..7 {
                t1.temp_limit[i] = self.temp_limits[i].max(0) as u16;
            }

            let mut mask = 0u64;
            for (bit, &enabled) in self.smu_features.iter().enumerate() {
                if enabled {
                    mask |= 1u64 << bit;
                }
            }
            t1.features_to_run = mask;

            t1.od_feature_flags[0] = u8::from(self.gpu_clock_limit);
            t1.od_feature_flags[1] = u8::from(self.gpu_clock_curve);
            t1.od_feature_flags[2] = u8::from(self.memory_clock_limit);
            t1.od_feature_flags[3] = u8::from(self.zero_rpm);
            t1.od_feature_flags[4] = u8::from(self.min_fan_speed);
            t1.od_feature_flags[5] = u8::from(self.fan_target_temp_limit);
            t1.od_feature_flags[6] = u8::from(self.operating_temp_limit);
            t1.od_feature_flags[7] = u8::from(self.fan_acoustic_rpm);
            t1.od_feature_flags[8] = u8::from(self.fan_curve);
            t1.od_feature_flags[9] = u8::from(self.power_limit_feat);
            t1.od_feature_flags[10] = u8::from(self.timing_tuning);
            t1.od_feature_flags[11] = u8::from(self.power_mode);
            }
            if orig_len == 1674 && u16::from_le_bytes([self.raw_buffer[5], self.raw_buffer[6]]) == 0x01E2 {
                let dst = std::mem::offset_of!(crate::pptable_rdna1::AtomSmcDpmInfoV4_5, socket_power_limit_ac) - 0x00C;
                let inner_n = 1674 - 0x01E2;
                self.raw_buffer[0x01E2..1674].copy_from_slice(&padded[dst..dst + inner_n]);
            } else {
                self.raw_buffer.copy_from_slice(&padded[..orig_len]);
            }
            return true;
        }

        let vst = self.capture_vega_parsed();
        vst.write_to_buf(&mut self.raw_buffer)
    }
}
