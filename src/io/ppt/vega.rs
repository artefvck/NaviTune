
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(super) fn try_apply_vega20(&mut self, b: &[u8], label: &str, frev: u8, crev: u8, trev: u8) -> bool {
        if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes_padded(b) {
            let saved_vbios = self.vbios_buffer.take();
            let saved_voff = self.vbios_ppt_offset.take();
            let saved_file = self.loaded_file.take();
            let saved_gpu = self.selected_gpu;
            self.reset_all_values();
            self.vbios_buffer = saved_vbios;
            self.vbios_ppt_offset = saved_voff;
            self.loaded_file = saved_file;
            self.selected_gpu = saved_gpu;
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_v = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev);

            let f = { v20.od_feature_flags };
            self.load_od8_flags_10(&f);
            let od_min = { v20.od_settings_min };
            let od_max = { v20.od_settings_max };
            self.load_od8_settings_5(&od_min, &od_max);

            let ppt0 = { v20.socket_power_limit_ac[0] } as i32;
            self.power_limit_w = if ppt0 > 0 { ppt0 } else { ({ v20.boost_power_limit }) as i32 };
            self.tdc_gfx = { v20.tdc_limit_gfx } as i32;
            self.tdc_soc = { v20.tdc_limit_soc } as i32;
            self.volt_gfx = [{ v20.min_voltage_gfx } as i32 / 4, { v20.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ v20.min_voltage_soc } as i32 / 4, { v20.max_voltage_soc } as i32 / 4];

            let desc = { v20.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let ft_gfx = { v20.freq_table_gfx };
            let ft_soc = { v20.freq_table_socclk };
            let ft_uclk = { v20.freq_table_uclk };
            let ft_fclk = { v20.freq_table_fclk };
            self.freq_gfx = [ft_gfx[0] as i32, ft_gfx[gfx_n - 1] as i32];
            self.freq_soc = [ft_soc[0] as i32, ft_soc[soc_n - 1] as i32];
            self.freq_fclk = [ft_fclk[0] as i32, ft_fclk[fclk_n - 1] as i32];
            let vddci_hi = { v20.vddci_max } as i32 / 4;
            let mvdd_hi = { v20.mvdd_max } as i32 / 4;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [ft_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [vddci_hi, mvdd_hi];
            }
            self.fan_pwm_min = { v20.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { v20.fan_acoustic_limit_rpm } as i32;
            self.fan_throttling_rpm = { v20.fan_throttling_rpm } as i32;
            self.fan_max_rpm = { v20.fan_maximum_rpm } as i32;
            self.fan_target_temp = { v20.fan_target_temperature } as i32;
            self.fan_target_gfx_clock = { v20.fan_target_gfxclk } as i32;
            self.ulv_gfx = [{ v20.min_voltage_ulv_gfx } as i32 / 4, { v20.ulv_voltage_offset_gfx } as i32 / 4];
            self.ulv_soc = [{ v20.min_voltage_ulv_soc } as i32 / 4, { v20.ulv_voltage_offset_soc } as i32 / 4];
            if let Some(st) = crate::pptable_vega::VegaParsedState::parse(b, frev) { self.apply_vega_parsed(st); }
            self.fan_zero_rpm_enable = v20.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { v20.fan_stop_temp } as i32;
            self.fan_start_temp = { v20.fan_start_temp } as i32;
            let mask = if b.len() == 1534 && b.len() >= 0x01E6 {
                u64::from_le_bytes([
                    b[0x01DE], b[0x01DF], b[0x01E0], b[0x01E1],
                    b[0x01E2], b[0x01E3], b[0x01E4], b[0x01E5],
                ])
            } else {
                v20.features_to_run
            };
            for (bit, feature) in self.smu_features.iter_mut().enumerate() {
                *feature = (mask & (1u64 << bit)) != 0;
            }
            let tl = { v20.temp_limit };
            for i in 0..7 { self.temp_limits[i] = tl[i] as i32; }

            // pol_caps_to_od_feat: map ulPlatformCaps (0x013) onto OverDrive Feature checkboxes
            if b.len() >= 0x17 {
                let pcaps = u32::from_le_bytes([b[0x13], b[0x14], b[0x15], b[0x16]]);
                let odn_ac = (pcaps & (1 << 18)) != 0;
                self.gpu_clock_limit = odn_ac;
                self.memory_clock_limit = odn_ac;
                self.power_limit_feat = odn_ac;
                self.min_fan_speed = (pcaps & (1 << 11)) != 0 || self.fan_pwm_min > 0;
                self.fan_target_temp_limit = self.fan_target_temp > 0;
                self.operating_temp_limit = (pcaps & (1 << 2)) != 0;
                self.zero_rpm = self.fan_zero_rpm_enable;
            }
            if b.len() == 1534 && b.len() >= 0x53C {
                let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as i32;
                let v20_pro_toffs = [0x1FAusize, 0x1FC, 0x202, 0x204, 0x206, 0x208, 0x20A];
                for (i, &toff) in v20_pro_toffs.iter().enumerate() {
                    self.temp_limits[i] = u16_at(toff);
                }
                self.fan_acoustic_limit = u16_at(0x536);
                self.fan_throttling_rpm = u16_at(0x538);
                self.fan_max_rpm = u16_at(0x53A);
            }
            let pcaps = u32::from_le_bytes([b[0x013], b[0x014], b[0x015], b[0x016]]);
            let odn_ac = (pcaps & (1 << 18)) != 0;
            self.gpu_clock_limit = odn_ac || self.oc_gfx_clock[1] > 0;
            self.memory_clock_limit = odn_ac || self.oc_mem_clock[1] > 0;
            self.power_limit_feat = odn_ac || self.oc_power_limit[1] > 0;
            self.min_fan_speed = (pcaps & (1 << 11)) != 0 || self.fan_pwm_min > 0;
            self.fan_target_temp_limit = (pcaps & (1 << 12)) != 0 || self.fan_target_temp > 0;
            self.operating_temp_limit = (pcaps & (1 << 2)) != 0 || self.temp_limits[0] > 0;
            self.zero_rpm = (pcaps & (1 << 14)) != 0 || self.fan_zero_rpm_enable;
            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_v.display_name(), frev, crev, trev, b.len());
            self.baseline_buffer = b.to_owned();
            self.raw_buffer = b.to_owned();
            self.baseline_gen = self.baseline_gen.wrapping_add(1);
            return true;
        }
        false
    }
    pub(super) fn try_apply_vega10(&mut self, b: &[u8], label: &str, frev: u8, crev: u8, trev: u8) -> bool {
        if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_v = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev);
            self.oc_gfx_clock = [0, ({ v10.max_od_engine_clock } / 100) as i32];
            self.oc_mem_clock = [0, ({ v10.max_od_memory_clock } / 100) as i32];
            self.oc_power_limit = [-50, { v10.power_control_limit } as i32];
            self.oc_fan_rpm = [0, 0];
            self.oc_fan_acoustic_rpm = [0, 0];
            self.power_limit_w = 0;
            self.tdc_gfx = 0;
            self.tdc_soc = 0;
            if let Some((soc_hi, gfx_hi, mem_hi, vddc_hi)) = v10.read_hard_limits(b) {
                self.volt_gfx = [0, vddc_hi];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, gfx_hi];
                self.freq_soc = [0, soc_hi];
                self.mem_dpm_freq[3] = [mem_hi, 1];
                if let Some((vddci_hi, mvdd_hi)) = v10.read_hard_limit_mem_volt(b) { self.mem_dpm_volt[3] = [vddci_hi, mvdd_hi]; }
            } else {
                self.volt_gfx = [0, 0];
                self.volt_soc = [0, 0];
            }
            self.ulv_gfx = [0, 0];
            self.ulv_soc = [0, 0];
            self.freq_fclk = [0, 0];
            self.freq_vclk = [0, 0];
            self.freq_dclk = [0, 0];
            self.freq_dcefclk = [0, 0];
            self.freq_dispclk = [0, 0];
            self.freq_pixclk = [0, 0];
            self.freq_phyclk = [0, 0];
            for i in 0..4 {
                self.mem_dpm_freq[i] = [0, 1];
                self.mem_dpm_volt[i] = [0, 0];
            }
            if let Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t)) = v10.read_fan_subtable(b) {
                self.fan_pwm_min = pwm_min;
                self.fan_acoustic_limit = 0;
                self.fan_throttling_rpm = 0;
                self.fan_max_rpm = max_rpm;
                self.fan_target_temp = tgt_temp;
                self.fan_target_gfx_clock = tgt_gfx;
                self.fan_zero_rpm_enable = zrpm;
                self.fan_stop_temp = stop_t;
                self.fan_start_temp = start_t;
            } else {
                self.fan_pwm_min = 0;
                self.fan_acoustic_limit = 0;
                self.fan_throttling_rpm = 0;
                self.fan_max_rpm = 0;
                self.fan_target_temp = 0;
                self.fan_target_gfx_clock = 0;
                self.fan_zero_rpm_enable = false;
                self.fan_stop_temp = 0;
                self.fan_start_temp = 0;
            }
            self.temp_limits.fill(0);
            if b.len() >= 0x17 {
                let pcaps = u32::from_le_bytes([b[0x13], b[0x14], b[0x15], b[0x16]]);
                for (bit, feat) in self.smu_features.iter_mut().enumerate().take(32) {
                    *feat = (pcaps & (1u32 << bit)) != 0;
                }
            }
            if let Some(st) = crate::pptable_vega::VegaParsedState::parse(b, frev) {
                self.apply_vega_parsed(st);
            }
            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_v.display_name(), frev, crev, trev, b.len());
            self.baseline_buffer = b.to_owned();
            self.raw_buffer = b.to_owned();
            self.baseline_gen = self.baseline_gen.wrapping_add(1);
            return true;
        }
        false
    }
    pub(super) fn try_sync_vega20(&mut self) -> bool {
        if let Some(mut v20_val) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes_padded(&self.raw_buffer) { let v20 = &mut v20_val;
            v20.boost_power_limit = self.power_limit_w.max(0) as u16;
            v20.socket_power_limit_ac[0] = self.power_limit_w.max(0) as u16;
            v20.tdc_limit_gfx = self.tdc_gfx.max(0) as u16;
            v20.tdc_limit_soc = self.tdc_soc.max(0) as u16;
            v20.min_voltage_gfx = (self.volt_gfx[0].max(0) * 4) as u16;
            v20.max_voltage_gfx = (self.volt_gfx[1].max(0) * 4) as u16;
            v20.min_voltage_soc = (self.volt_soc[0].max(0) * 4) as u16;
            v20.max_voltage_soc = (self.volt_soc[1].max(0) * 4) as u16;

            v20.od_settings_min[0] = self.oc_gfx_clock[0].max(0) as u32;
            v20.od_settings_max[0] = self.oc_gfx_clock[1].max(0) as u32;
            v20.od_settings_min[1] = self.oc_mem_clock[0].max(0) as u32;
            v20.od_settings_max[1] = self.oc_mem_clock[1].max(0) as u32;
            v20.od_settings_min[2] = self.oc_power_limit[0] as u32;
            v20.od_settings_max[2] = self.oc_power_limit[1].max(0) as u32;
            v20.od_settings_min[3] = self.oc_fan_rpm[0].max(0) as u32;
            v20.od_settings_max[3] = self.oc_fan_rpm[1].max(0) as u32;
            v20.od_settings_min[4] = self.oc_fan_acoustic_rpm[0].max(0) as u32;
            v20.od_settings_max[4] = self.oc_fan_acoustic_rpm[1].max(0) as u32;

            let desc = { v20.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            v20.freq_table_gfx[0] = self.freq_gfx[0].max(0) as u16;
            v20.freq_table_gfx[gfx_n - 1] = self.freq_gfx[1].max(0) as u16;
            v20.freq_table_socclk[0] = self.freq_soc[0].max(0) as u16;
            v20.freq_table_socclk[soc_n - 1] = self.freq_soc[1].max(0) as u16;
            v20.freq_table_fclk[0] = self.freq_fclk[0].max(0) as u16;
            v20.freq_table_fclk[fclk_n - 1] = self.freq_fclk[1].max(0) as u16;
            for i in 0..4 { v20.freq_table_uclk[i] = self.mem_dpm_freq[i][0].max(0) as u16; }
            v20.vddci_max = (self.mem_dpm_volt[3][0].max(0) * 4) as u16;
            v20.mvdd_max = (self.mem_dpm_volt[3][1].max(0) * 4) as u16;

            v20.fan_pwm_min = self.fan_pwm_min.max(0) as u16;
            v20.fan_acoustic_limit_rpm = self.fan_acoustic_limit.max(0) as u16;
            v20.fan_throttling_rpm = self.fan_throttling_rpm.max(0) as u16;
            v20.fan_maximum_rpm = self.fan_max_rpm.max(0) as u16;
            v20.fan_target_temperature = self.fan_target_temp.max(0) as u16;
            v20.fan_target_gfxclk = self.fan_target_gfx_clock.max(0) as u16;
            let qmv_u = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
            let qmv_i = |ui_mv: i32, orig: i16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv * 4) as i16 };
            v20.min_voltage_ulv_gfx = qmv_u(self.ulv_gfx[0], v20.min_voltage_ulv_gfx);
            v20.min_voltage_ulv_soc = qmv_u(self.ulv_soc[0], v20.min_voltage_ulv_soc);
            v20.ulv_voltage_offset_gfx = qmv_i(self.ulv_gfx[1], v20.ulv_voltage_offset_gfx);
            v20.ulv_voltage_offset_soc = qmv_i(self.ulv_soc[1], v20.ulv_voltage_offset_soc);
            v20.fan_zero_rpm_enable = u8::from(self.fan_zero_rpm_enable);
            v20.fan_stop_temp = self.fan_stop_temp.max(0) as u16;
            v20.fan_start_temp = self.fan_start_temp.max(0) as u16;
            for i in 0..7 { v20.temp_limit[i] = self.temp_limits[i].max(0) as u16; }

            v20.features_to_run = Self::build_smu_mask(&self.smu_features);
            v20.od_feature_flags = [
                self.gpu_clock_limit as u8, self.gpu_clock_curve as u8,
                self.memory_clock_limit as u8, self.zero_rpm as u8,
                self.min_fan_speed as u8, self.fan_target_temp_limit as u8,
                self.operating_temp_limit as u8, self.fan_acoustic_rpm as u8,
                self.fan_curve as u8, self.power_limit_feat as u8,
                self.timing_tuning as u8, self.power_mode as u8,
                self.auto_undervolt_gpu as u8, self.auto_overclock_gpu as u8,
                self.auto_overclock_mem as u8, self.auto_fan_acoustic_limit as u8,
            ];
            v20_val.write_back_to_slice(&mut self.raw_buffer);
            if self.raw_buffer.len() == 1534 && self.raw_buffer.len() >= 0x53C {
                let mask1534 = Self::build_smu_mask(&self.smu_features);
                self.raw_buffer[0x01DE..0x01E6].copy_from_slice(&mask1534.to_le_bytes());
                let v20_pro_toffs = [0x1FAusize, 0x1FC, 0x202, 0x204, 0x206, 0x208, 0x20A];
                for (i, &toff) in v20_pro_toffs.iter().enumerate() {
                    let bytes = (self.temp_limits[i].max(0) as u16).to_le_bytes();
                    self.raw_buffer[toff..toff + 2].copy_from_slice(&bytes);
                }
                self.raw_buffer[0x536..0x538].copy_from_slice(&(self.fan_acoustic_limit.max(0) as u16).to_le_bytes());
                self.raw_buffer[0x538..0x53A].copy_from_slice(&(self.fan_throttling_rpm.max(0) as u16).to_le_bytes());
                self.raw_buffer[0x53A..0x53C].copy_from_slice(&(self.fan_max_rpm.max(0) as u16).to_le_bytes());
            }
            return true;
        }
        false
    }
    pub(super) fn try_sync_vega10(&mut self) -> bool {
        if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
            v10.max_od_engine_clock = (self.oc_gfx_clock[1].max(0) * 100) as u32;
            v10.max_od_memory_clock = (self.oc_mem_clock[1].max(0) * 100) as u32;
            v10.power_control_limit = self.oc_power_limit[1].max(0) as u16;
            if self.raw_buffer.len() >= 0x17 {
                let mut pcaps = 0u32;
                for (bit, &en) in self.smu_features.iter().enumerate().take(32) {
                    if en {
                        pcaps |= 1u32 << bit;
                    }
                }
                self.raw_buffer[0x13..0x17].copy_from_slice(&pcaps.to_le_bytes());
            }
            crate::pptable_vega::AtomVega10PowerPlayTable::write_hard_limits_full(
                &mut self.raw_buffer,
                self.freq_soc[1],
                self.freq_gfx[1],
                self.mem_dpm_freq[3][0],
                self.volt_gfx[1],
                self.mem_dpm_volt[3][0],
                self.mem_dpm_volt[3][1],
            );
            crate::pptable_vega::AtomVega10PowerPlayTable::write_fan_subtable(
                &mut self.raw_buffer,
                self.fan_pwm_min,
                self.fan_max_rpm,
                self.fan_target_temp,
                self.fan_target_gfx_clock,
                self.fan_zero_rpm_enable,
                self.fan_stop_temp,
                self.fan_start_temp,
            );
            return true;
        }
        false
    }
    fn apply_vega_parsed(&mut self, st: crate::pptable_vega::VegaParsedState) {
        self.power_limit_w = st.power_limit_w;
        self.power_limit_ac[0] = st.power_limit_w;
        self.tdc_gfx = st.tdc_gfx;
        self.tdc_soc = st.tdc_soc;
        self.oc_gfx_clock = st.oc_gfx_clock;
        self.oc_mem_clock = st.oc_mem_clock;
        self.oc_power_limit = st.oc_power_limit;
        self.oc_fan_rpm = [0, 0];
        self.oc_fan_acoustic_rpm = [0, 0];
        self.volt_gfx = st.volt_gfx;
        self.volt_soc = st.volt_soc;
        self.freq_gfx = st.freq_gfx;
        self.freq_soc = st.freq_soc;
        self.freq_fclk = st.freq_fclk;
        self.mem_dpm_freq = st.mem_dpm_freq;
        self.mem_dpm_volt = st.mem_dpm_volt;
        self.fan_pwm_min = st.fan_pwm_min;
        self.fan_max_rpm = st.fan_max_rpm;
        self.fan_acoustic_limit = st.fan_acoustic_limit;
        self.fan_throttling_rpm = st.fan_throttling_rpm;
        self.fan_target_temp = st.fan_target_temp;
        self.fan_target_gfx_clock = st.fan_target_gfx_clock;
        self.fan_zero_rpm_enable = st.fan_zero_rpm_enable;
        self.fan_stop_temp = st.fan_stop_temp;
        self.fan_start_temp = st.fan_start_temp;
        self.temp_limits[0] = st.temp_limit_0;
        self.gpu_clock_limit = st.caps[0];
        self.memory_clock_limit = st.caps[1];
        self.power_limit_feat = st.caps[2];
        self.min_fan_speed = st.caps[3];
        self.fan_target_temp_limit = st.caps[4];
        self.operating_temp_limit = st.caps[5];
        self.zero_rpm = st.caps[6];
    }

    pub(super) fn capture_vega_parsed(&self) -> crate::pptable_vega::VegaParsedState {
        let base = crate::pptable_vega::VegaParsedState::parse(&self.raw_buffer, self.raw_buffer.get(2).copied().unwrap_or(0)).unwrap_or_default();
        crate::pptable_vega::VegaParsedState {
            power_limit_w: self.power_limit_w,
            tdc_gfx: self.tdc_gfx,
            tdc_soc: self.tdc_soc,
            oc_gfx_clock: self.oc_gfx_clock,
            oc_mem_clock: self.oc_mem_clock,
            oc_power_limit: self.oc_power_limit,
            volt_gfx: self.volt_gfx,
            volt_soc: self.volt_soc,
            freq_gfx: self.freq_gfx,
            freq_soc: self.freq_soc,
            freq_fclk: self.freq_fclk,
            mem_dpm_freq: self.mem_dpm_freq,
            mem_dpm_volt: self.mem_dpm_volt,
            fan_pwm_min: self.fan_pwm_min,
            fan_max_rpm: self.fan_max_rpm,
            fan_acoustic_limit: self.fan_acoustic_limit,
            fan_throttling_rpm: self.fan_throttling_rpm,
            fan_target_temp: self.fan_target_temp,
            fan_target_gfx_clock: self.fan_target_gfx_clock,
            fan_zero_rpm_enable: self.fan_zero_rpm_enable,
            fan_stop_temp: self.fan_stop_temp,
            fan_start_temp: self.fan_start_temp,
            temp_limit_0: self.temp_limits[0],
            caps: [self.gpu_clock_limit, self.memory_clock_limit, self.power_limit_feat, self.min_fan_speed, self.fan_target_temp_limit, self.operating_temp_limit, self.zero_rpm],
            ..base
        }
    }

}
