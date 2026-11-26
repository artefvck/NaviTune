
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(super) fn try_apply_polaris(&mut self, b: &[u8], label: &str, frev: u8, crev: u8, trev: u8) -> bool {
        if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_p = crate::pptable_polaris::PolarisChip::from_did(did);
            self.oc_gfx_clock = [0, ({ pol.max_od_engine_clock } / 100) as i32];
            self.oc_mem_clock = [0, ({ pol.max_od_memory_clock } / 100) as i32];
            self.oc_power_limit = [-50, { pol.power_control_limit } as i32];
            self.oc_fan_rpm = [0, 0];
            self.oc_fan_acoustic_rpm = [0, 0];
            self.power_limit_w = 0;
            self.tdc_gfx = 0;
            self.tdc_soc = 0;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [0, 1];
                self.mem_dpm_volt[i] = [0, 0];
            }
            if let Some((gfx_hi, mem_hi, vddc_hi)) = pol.read_hard_limits(b) {
                self.volt_gfx = [0, vddc_hi];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, gfx_hi];
                self.freq_soc = [0, 0];
                self.mem_dpm_freq[3] = [mem_hi, 1];
                if let Some((vddci_hi, mvdd_hi)) = pol.read_hard_limit_mem_volt(b) { self.mem_dpm_volt[3] = [vddci_hi, mvdd_hi]; }
            } else {
                self.volt_gfx = [0, 0];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, 0];
                self.freq_soc = [0, 0];
            }
            let (s_min, s_max, v_min, v_max, pwr_w, tdc_a, m_states) = pol.read_polaris_dpm_and_powertune(b);
            if s_max > 0 { self.freq_gfx = [s_min, s_max]; }
            if v_max > 0 { self.volt_gfx = [v_min, v_max]; }
            if pwr_w > 0 { self.power_limit_w = pwr_w; self.power_limit_ac[0] = pwr_w; }
            if tdc_a > 0 { self.tdc_gfx = tdc_a; }
            for (idx, &(mc, vci, mvd)) in m_states.iter().enumerate() {
                self.mem_dpm_freq[idx] = [mc, 1];
                self.mem_dpm_volt[idx] = [vci, mvd];
            }
            if let Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t)) = pol.read_fan_subtable(b) {
                self.fan_pwm_min = pwm_min;
                self.fan_max_rpm = max_rpm;
                self.fan_target_temp = tgt_temp;
                self.fan_target_gfx_clock = tgt_gfx;
                self.fan_zero_rpm_enable = zrpm;
                self.fan_stop_temp = stop_t;
                self.fan_start_temp = start_t;
            }
            // polaris_od_caps_loaded: decode ulPlatformCaps (0x013) + ATOM_Tonga_Fan_Table state
            if b.len() >= 0x17 {
                let pcaps = u32::from_le_bytes([b[0x13], b[0x14], b[0x15], b[0x16]]);
                for bit in 0..32usize {
                    self.smu_features[bit] = (pcaps & (1u32 << bit)) != 0;
                }
                let odn_ac = (pcaps & (1 << 18)) != 0;
                self.gpu_clock_limit = odn_ac;
                self.memory_clock_limit = odn_ac;
                self.power_limit_feat = odn_ac;
                self.min_fan_speed = (pcaps & (1 << 11)) != 0 || self.fan_pwm_min > 0;
                self.fan_target_temp_limit = self.fan_target_temp > 0;
                self.operating_temp_limit = (pcaps & (1 << 2)) != 0 || self.fan_target_temp > 0;
                self.zero_rpm = false;
            }
            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_p.display_name(), frev, crev, trev, b.len());
            self.baseline_buffer = b.to_owned();
            self.raw_buffer = b.to_owned();
            self.baseline_gen = self.baseline_gen.wrapping_add(1);
            return true;
        }
        false
    }
    pub(super) fn try_sync_polaris(&mut self) -> bool {
        let pol_orig_caps = if self.raw_buffer.len() >= 0x17 {
            u32::from_le_bytes([self.raw_buffer[0x13], self.raw_buffer[0x14], self.raw_buffer[0x15], self.raw_buffer[0x16]])
        } else { 0 };
        if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
            pol.max_od_engine_clock = (self.oc_gfx_clock[1].max(0) * 100) as u32;
            pol.max_od_memory_clock = (self.oc_mem_clock[1].max(0) * 100) as u32;
            pol.power_control_limit = self.oc_power_limit[1].max(0) as u16;
            crate::pptable_polaris::AtomPolarisPowerPlayTable::write_polaris_dpm_and_powertune(&mut self.raw_buffer, self.freq_gfx[0], self.freq_gfx[1], self.volt_gfx[1], self.power_limit_w, self.tdc_gfx, &self.mem_dpm_freq, &self.mem_dpm_volt);
            // polaris_od_caps_saved: sync 32-bit PlatformCaps + OverDrive Feature checkboxes back to ulPlatformCaps (0x013..0x017)
            if self.raw_buffer.len() >= 0x17 {
                let mut pcaps = 0u32;
                for bit in 0..32usize {
                    if self.smu_features[bit] {
                        pcaps |= 1u32 << bit;
                    }
                }
                if self.gpu_clock_limit || self.memory_clock_limit || self.power_limit_feat { pcaps |= 1 << 18; } else { pcaps &= !(1 << 18); }
                if self.min_fan_speed { pcaps |= 1 << 11; } else { pcaps &= !(1 << 11); }
                if self.operating_temp_limit { pcaps |= 1 << 2; } else { pcaps &= !(1 << 2); }
                self.raw_buffer[0x13..0x17].copy_from_slice(&pcaps.to_le_bytes());
            }
            crate::pptable_polaris::AtomPolarisPowerPlayTable::write_fan_subtable(
                &mut self.raw_buffer,
                self.fan_pwm_min,
                self.fan_max_rpm,
                self.fan_target_temp,
                self.fan_target_gfx_clock,
                self.fan_zero_rpm_enable,
                self.fan_stop_temp,
                self.fan_start_temp,
            );
            crate::pptable_polaris::AtomPolarisPowerPlayTable::write_hard_limits_full(
                &mut self.raw_buffer,
                self.freq_gfx[1],
                self.mem_dpm_freq[3][0],
                self.volt_gfx[1],
                self.mem_dpm_volt[3][0],
                self.mem_dpm_volt[3][1],
            );
            if self.raw_buffer.len() >= 0x17 {
                let next_caps = u32::from_le_bytes([self.raw_buffer[0x13], self.raw_buffer[0x14], self.raw_buffer[0x15], self.raw_buffer[0x16]]);
                let mask = (1u32 << 2) | (1u32 << 11);
                if (pol_orig_caps & mask) == 0 && (next_caps & !mask) == (pol_orig_caps & !mask) {
                    self.raw_buffer[0x13..0x17].copy_from_slice(&pol_orig_caps.to_le_bytes());
                }
            }
            return true;
        }
        false
    }
}
