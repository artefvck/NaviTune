
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(super) fn try_apply_rdna3(&mut self, b: &[u8], label: &str, frev: u8, crev: u8, trev: u8) -> bool {
        if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(b) {
            let chip = t3.detect_chip(0);
            let saved_vbios = self.vbios_buffer.take();
            let saved_voff = self.vbios_ppt_offset.take();
            let saved_file = self.loaded_file.take();
            let saved_gpu = self.selected_gpu;
            self.reset_all_values();
            self.vbios_buffer = saved_vbios;
            self.vbios_ppt_offset = saved_voff;
            self.loaded_file = saved_file;
            self.selected_gpu = saved_gpu;

            let f = t3.od_feature_flags;
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

            let mask = { t3.features_to_run };
            for (bit, feature) in self.smu_features.iter_mut().enumerate() {
                *feature = (mask & (1u64 << bit)) != 0;
            }

            self.oc_gfx_clock = [0, { t3.od_gfx_max } as i32];
            self.oc_mem_clock = [0, { t3.od_mem_max } as i32];
            self.oc_power_limit = [0, { t3.od_pwr_max } as i32];
            self.oc_fan_rpm = [0, { t3.od_fan_max } as i32];
            self.oc_fan_acoustic_rpm = [0, { t3.od_acoustic_max } as i32];

            self.power_limit_w = { t3.socket_power_limit_ac } as i32;
            self.volt_gfx = [{ t3.min_voltage_gfx } as i32 / 4, { t3.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ t3.min_voltage_soc } as i32 / 4, { t3.max_voltage_soc } as i32 / 4];
            self.ulv_gfx = [{ t3.ulv_voltage_vmin_gfx } as i32 / 4, 0];
            self.ulv_soc = [{ t3.ulv_voltage_vmin_soc } as i32 / 4, 0];

            self.freq_gfx = [{ t3.freq_gfx_min } as i32, { t3.freq_gfx_max } as i32];
            self.freq_soc = [500, { t3.freq_soc_max } as i32];
            self.freq_fclk = [{ t3.freq_fclk_min } as i32, { t3.freq_fclk_max } as i32];
            self.tdc_gfx = { t3.tdc_limit_gfx } as i32;
            self.tdc_soc = { t3.tdc_limit_soc } as i32;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [t3.freq_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [(t3.mem_vddci[i] / 4) as i32, (t3.mem_mvdd[i] / 4) as i32];
            }
            self.pcie_gen_speed = [t3.pcie_gen_speed[0] as i32, t3.pcie_gen_speed[2] as i32];
            self.pcie_lane_count = [t3.pcie_lane_count[0] as i32, t3.pcie_lane_count[2] as i32];
            self.lclk_mhz = [t3.lclk_freq[0] as i32, t3.lclk_freq[2] as i32];
            self.fan_throttling_rpm = { t3.fan_throttling_rpm } as i32;
            self.fan_pwm_min = { t3.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { t3.fan_acoustic_limit } as i32;
            self.fan_max_rpm = { t3.fan_max_rpm } as i32;
            self.fan_target_gfx_clock = { t3.fan_target_gfx_clock } as i32;
            let raw_tt = { t3.fan_target_temp } as i32;
            self.fan_target_temp = if raw_tt > 200 { raw_tt / 10 } else { raw_tt };
            self.fan_zero_rpm_enable = t3.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { t3.fan_stop_temp } as i32;
            self.fan_start_temp = { t3.fan_start_temp } as i32;
            for i in 0..9 {
                self.temp_limits[i] = t3.temp_limit[i] as i32;
            }

            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PowerPlayInfo @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, chip.display_name(), frev, crev, trev, b.len(), vbios_tag
            );
            self.baseline_buffer = b.to_owned();
            self.raw_buffer = b.to_owned();
            return true;
        }
        false
    }
    pub(super) fn try_sync_rdna3(&mut self) -> bool {
        if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes_mut(&mut self.raw_buffer) {
            t3.od_feature_flags[0..12].copy_from_slice(&[
                self.gpu_clock_limit as u8, self.gpu_clock_curve as u8, self.memory_clock_limit as u8, self.zero_rpm as u8,
                self.min_fan_speed as u8, self.fan_target_temp_limit as u8, self.operating_temp_limit as u8, self.fan_acoustic_rpm as u8,
                self.fan_curve as u8, self.power_limit_feat as u8, self.timing_tuning as u8, self.power_mode as u8,
            ]);
            let mut mask = 0u64;
            for (bit, &en) in self.smu_features.iter().enumerate() {
                if en { mask |= 1u64 << bit; }
            }
            t3.features_to_run = mask;
            t3.od_gfx_max = self.oc_gfx_clock[1] as u32;
            t3.od_mem_max = self.oc_mem_clock[1] as u32;
            t3.od_pwr_max = self.oc_power_limit[1] as u32;
            t3.od_fan_max = self.oc_fan_rpm[1] as u32;
            t3.od_acoustic_max = self.oc_fan_acoustic_rpm[1] as u32;
            t3.socket_power_limit_ac = self.power_limit_w as u16;
            let qmv_r3 = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
        t3.ulv_voltage_vmin_gfx = qmv_r3(self.ulv_gfx[0], t3.ulv_voltage_vmin_gfx);
        t3.ulv_voltage_vmin_soc = qmv_r3(self.ulv_soc[0], t3.ulv_voltage_vmin_soc);
        t3.min_voltage_gfx = (self.volt_gfx[0] * 4) as u16;
            t3.max_voltage_gfx = (self.volt_gfx[1] * 4) as u16;
            t3.min_voltage_soc = (self.volt_soc[0] * 4) as u16;
            t3.max_voltage_soc = (self.volt_soc[1] * 4) as u16;
            t3.freq_gfx_min = self.freq_gfx[0] as u16;
            t3.freq_gfx_max = self.freq_gfx[1] as u16;
            t3.freq_soc_max = self.freq_soc[1] as u16;
            t3.freq_fclk_min = self.freq_fclk[0] as u16;
            t3.freq_fclk_max = self.freq_fclk[1] as u16;
            t3.tdc_limit_gfx = self.tdc_gfx as u16;
            t3.tdc_limit_soc = self.tdc_soc as u16;
            for i in 0..4 {
                t3.freq_uclk[i] = self.mem_dpm_freq[i][0] as u16;
                t3.mem_vddci[i] = (self.mem_dpm_volt[i][0] * 4) as u16;
                t3.mem_mvdd[i] = (self.mem_dpm_volt[i][1] * 4) as u16;
            }
            t3.pcie_gen_speed[0] = self.pcie_gen_speed[0] as u8;
            t3.pcie_gen_speed[2] = self.pcie_gen_speed[1] as u8;
            t3.pcie_lane_count[0] = self.pcie_lane_count[0] as u8;
            t3.pcie_lane_count[2] = self.pcie_lane_count[1] as u8;
            t3.lclk_freq[0] = self.lclk_mhz[0] as u16;
            t3.lclk_freq[2] = self.lclk_mhz[1] as u16;
            t3.fan_throttling_rpm = self.fan_throttling_rpm as u16;
            t3.fan_pwm_min = self.fan_pwm_min as u16;
            t3.fan_acoustic_limit = self.fan_acoustic_limit as u16;
            t3.fan_max_rpm = self.fan_max_rpm as u16;
            t3.fan_target_gfx_clock = self.fan_target_gfx_clock as u16;
            let orig_tt = { t3.fan_target_temp };
            t3.fan_target_temp = if orig_tt > 200 { (self.fan_target_temp * 10) as u16 } else { self.fan_target_temp as u16 };
            t3.fan_zero_rpm_enable = self.fan_zero_rpm_enable as u8;
            t3.fan_stop_temp = self.fan_stop_temp as u16;
            t3.fan_start_temp = self.fan_start_temp as u16;
            for i in 0..9 {
                t3.temp_limit[i] = self.temp_limits[i] as u16;
            }
            return true;
        }
        false
    }
}
