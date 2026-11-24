use crate::ppt_arch::PptArch;
use crate::PowerPlayApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafePresetKind {
    QuietUndervolt,
    BalancedOc,
    AutoFixRisks,
}

#[derive(Clone, Copy, Debug)]
struct ArchSafeCaps {
    v_cap: i32,
    pwr_cap: i32,
    uclk_cap: i32,
    fan_cap: i32,
    supports_zero_rpm: bool,
    supports_direct_mv: bool,
}

impl ArchSafeCaps {
    fn for_arch(arch: PptArch) -> Self {
        match arch {
            PptArch::Polaris => Self { v_cap: 1150, pwr_cap: 185, uclk_cap: 2100, fan_cap: 80, supports_zero_rpm: false, supports_direct_mv: true },
            PptArch::Vega10  => Self { v_cap: 1100, pwr_cap: 260, uclk_cap: 1050, fan_cap: 82, supports_zero_rpm: true,  supports_direct_mv: true },
            PptArch::Vega20  => Self { v_cap: 1100, pwr_cap: 280, uclk_cap: 1100, fan_cap: 85, supports_zero_rpm: true,  supports_direct_mv: true },
            PptArch::Rdna1   => Self { v_cap: 1150, pwr_cap: 210, uclk_cap: 920,  fan_cap: 85, supports_zero_rpm: true,  supports_direct_mv: true },
            PptArch::Rdna2   => Self { v_cap: 1150, pwr_cap: 285, uclk_cap: 1000, fan_cap: 85, supports_zero_rpm: true,  supports_direct_mv: true },
            PptArch::Rdna3   => Self { v_cap: 1150, pwr_cap: 340, uclk_cap: 1250, fan_cap: 90, supports_zero_rpm: true,  supports_direct_mv: false },
            _                => Self { v_cap: 1150, pwr_cap: 250, uclk_cap: 1000, fan_cap: 85, supports_zero_rpm: true,  supports_direct_mv: true },
        }
    }
}

impl PowerPlayApp {
    pub(crate) fn apply_safe_preset(&mut self, kind: SafePresetKind) {
        if self.raw_buffer.is_empty() {
            return;
        }
        let arch = PptArch::detect(&self.raw_buffer);
        let caps = ArchSafeCaps::for_arch(arch);

        match kind {
            SafePresetKind::QuietUndervolt => {
                if caps.supports_direct_mv {
                    let vmin_floor = self.volt_gfx.first().copied().unwrap_or(780).max(780);
                    if let Some(vmax) = self.volt_gfx.last_mut() {
                        if *vmax > vmin_floor + 20 {
                            *vmax = (*vmax - 65).clamp(vmin_floor, caps.v_cap.max(vmin_floor));
                        }
                    }
                }
                if self.power_limit_w > 60 {
                    self.power_limit_w = ((self.power_limit_w as f32 * 0.90).round() as i32).min(caps.pwr_cap);
                    if let Some(ac0) = self.power_limit_ac.first_mut() {
                        *ac0 = self.power_limit_w;
                    }
                }
                let min_tgt = (self.fan_start_temp + 5).max(55);
                if self.fan_target_temp > min_tgt {
                    self.fan_target_temp = (self.fan_target_temp - 4).clamp(min_tgt, caps.fan_cap.max(min_tgt));
                }
                if caps.supports_zero_rpm && self.fan_start_temp > 0 {
                    self.fan_zero_rpm_enable = true;
                    self.zero_rpm = true;
                }
                self.status_msg = format!("Applied Quiet Undervolt preset ({:?})", arch);
            }
            SafePresetKind::BalancedOc => {
                let oc_max = self.oc_gfx_clock.last().copied().unwrap_or(0);
                if let Some(gmax) = self.freq_gfx.last_mut() {
                    if oc_max > *gmax + 40 {
                        let target = ((oc_max as f32 * 0.97).round() as i32).max(*gmax);
                        *gmax = target.min(oc_max);
                    }
                }
                if self.power_limit_w > 50 {
                    let boosted = (self.power_limit_w as f32 * 1.10).round() as i32;
                    self.power_limit_w = boosted.min(caps.pwr_cap);
                    if let Some(ac0) = self.power_limit_ac.first_mut() {
                        *ac0 = self.power_limit_w;
                    }
                }
                if caps.supports_direct_mv {
                    if let Some(vmax) = self.volt_gfx.last_mut() {
                        if *vmax > 0 {
                            *vmax = (*vmax).min(caps.v_cap);
                        }
                    }
                }
                let min_tgt = (self.fan_start_temp + 5).max(58);
                if self.fan_target_temp > min_tgt {
                    self.fan_target_temp = (self.fan_target_temp - 3).clamp(min_tgt, caps.fan_cap.max(min_tgt));
                }
                self.status_msg = format!("Applied Balanced OC preset ({:?})", arch);
            }
            SafePresetKind::AutoFixRisks => {
                let oc_max = self.oc_gfx_clock.last().copied().unwrap_or(0);
                if let Some(gmax) = self.freq_gfx.last_mut() {
                    if oc_max > 0 && *gmax > oc_max {
                        *gmax = oc_max;
                    }
                }
                let gmax_now = self.freq_gfx.last().copied().unwrap_or(0);
                if let Some(gmin) = self.freq_gfx.first_mut() {
                    if *gmin >= gmax_now && gmax_now > 300 {
                        *gmin = (gmax_now - 100).max(300);
                    }
                }
                let oc_mem_max = self.oc_mem_clock.last().copied().unwrap_or(0);
                for d in &mut self.mem_dpm_freq {
                    if let Some(uclk) = d.first_mut() {
                        if *uclk > caps.uclk_cap {
                            *uclk = caps.uclk_cap;
                        }
                        if oc_mem_max > 0 && *uclk > oc_mem_max {
                            *uclk = oc_mem_max;
                        }
                    }
                }
                if caps.supports_direct_mv {
                    if let Some(vmax) = self.volt_gfx.last_mut() {
                        if *vmax > caps.v_cap {
                            *vmax = caps.v_cap;
                        }
                    }
                    let vmax_now = self.volt_gfx.last().copied().unwrap_or(0);
                    if let Some(vmin) = self.volt_gfx.first_mut() {
                        if vmax_now > 0 && *vmin > vmax_now {
                            *vmin = (vmax_now - 150).max(700);
                        }
                    }
                }
                if self.power_limit_w > caps.pwr_cap {
                    self.power_limit_w = caps.pwr_cap;
                    if let Some(ac0) = self.power_limit_ac.first_mut() {
                        *ac0 = caps.pwr_cap;
                    }
                }
                if self.fan_target_temp > caps.fan_cap {
                    self.fan_target_temp = caps.fan_cap;
                }
                if self.fan_max_rpm > 0 {
                    if self.fan_throttling_rpm > self.fan_max_rpm {
                        self.fan_throttling_rpm = self.fan_max_rpm;
                    }
                    if self.fan_acoustic_limit > self.fan_max_rpm {
                        self.fan_acoustic_limit = self.fan_max_rpm;
                    }
                }
                if self.fan_stop_temp > 0 || self.fan_start_temp > 0 {
                    if self.fan_stop_temp >= self.fan_start_temp {
                        self.fan_stop_temp = (self.fan_start_temp - 5).max(25);
                    }
                    if self.fan_target_temp >= 45 && self.fan_start_temp >= self.fan_target_temp {
                        self.fan_start_temp = (self.fan_target_temp - 5).max(self.fan_stop_temp + 2);
                    }
                }
                if self.temp_limits[0] > 100 {
                    self.temp_limits[0] = 100;
                }
                if self.temp_limits[1] > 110 {
                    self.temp_limits[1] = 110;
                }
                if self.temp_limits[2] > 105 {
                    self.temp_limits[2] = 105;
                }
                if self.oc_power_limit[1] > 20 {
                    self.oc_power_limit[1] = 20;
                }
                if arch.is_rdna3() {
                    self.smu_features[17] = true;
                    self.smu_features[20] = true;
                } else if !arch.is_gcn_subtable() && arch != PptArch::Unknown {
                    self.smu_features[32] = true;
                    self.smu_features[33] = true;
                    self.smu_features[34] = true;
                }
                self.status_msg = format!("Auto-fixed out-of-range tunables ({:?})", arch);
            }
        }
        self.sync_fields_to_buffer();
    }
}

#[cfg(test)]
mod safe_presets_bounds_tests {
    use super::SafePresetKind;
    use crate::app::PowerPlayApp;

    #[test]
    fn default_app_presets_preserve_valid_clock_and_voltage_ordering() {
        let app = PowerPlayApp::default();
        assert!(app.freq_gfx.first() <= app.freq_gfx.last());
        assert!(app.volt_gfx.first() <= app.volt_gfx.last());
        assert!(app.validate_tunables().is_empty());
    }

    #[test]
    fn all_safe_presets_work_across_rdna2_and_rdna3_buffers() {
        for (frev, len) in [(0x0F_u8, 2470_usize), (0x14_u8, 5424_usize)] {
            let mut app = PowerPlayApp::default();
            let mut buf: Vec<u8> = std::iter::repeat_n(0u8, len).collect();
            let sz_bytes = (len as u16).to_le_bytes();
            if let Some(b0) = buf.get_mut(0) { *b0 = sz_bytes.first().copied().unwrap_or(0); }
            if let Some(b1) = buf.get_mut(1) { *b1 = sz_bytes.last().copied().unwrap_or(0); }
            if let Some(b2) = buf.get_mut(2) { *b2 = frev; }
            if let Some(b4) = buf.get_mut(4) { *b4 = 0x02; }
            app.raw_buffer = buf.clone();
            app.baseline_buffer = buf;
            if let Some(g0) = app.freq_gfx.first_mut() { *g0 = 500; }
            if let Some(g1) = app.freq_gfx.last_mut() { *g1 = 2400; }
            if let Some(oc1) = app.oc_gfx_clock.last_mut() { *oc1 = 2700; }
            if let Some(v0) = app.volt_gfx.first_mut() { *v0 = 750; }
            if let Some(v1) = app.volt_gfx.last_mut() { *v1 = 1150; }
            app.power_limit_w = 250;
            app.fan_target_temp = 85;
            for kind in [SafePresetKind::QuietUndervolt, SafePresetKind::BalancedOc, SafePresetKind::AutoFixRisks] {
                app.apply_safe_preset(kind);
                assert!(app.freq_gfx.first() <= app.freq_gfx.last());
                assert!(app.volt_gfx.first() <= app.volt_gfx.last());
            }
        }
    }

    #[test]
    fn rdna1_1674b_presets_and_autofix_repair_thermal_and_fan_risks() {
        let mut app = PowerPlayApp::default();
        let mut buf = vec![0u8; 1674];
        buf[0..2].copy_from_slice(&1674u16.to_le_bytes());
        buf[2] = 0x0C;
        buf[5..7].copy_from_slice(&0x01E2u16.to_le_bytes());
        app.raw_buffer = buf.clone();
        app.baseline_buffer = buf;
        app.freq_gfx = [800, 1950];
        app.oc_gfx_clock = [800, 2100];
        app.volt_gfx = [750, 1180];
        app.power_limit_w = 240;
        app.fan_max_rpm = 2800;
        app.fan_throttling_rpm = 3400;
        app.fan_stop_temp = 65;
        app.fan_start_temp = 60;
        app.fan_target_temp = 90;
        app.temp_limits[0] = 112;
        app.temp_limits[1] = 122;
        app.temp_limits[2] = 115;

        app.apply_safe_preset(SafePresetKind::AutoFixRisks);
        assert_eq!(app.fan_throttling_rpm, 2800);
        assert!(app.fan_stop_temp < app.fan_start_temp);
        assert!(app.fan_start_temp < app.fan_target_temp);
        assert_eq!(app.temp_limits[0], 100);
        assert_eq!(app.temp_limits[1], 110);
        assert_eq!(app.temp_limits[2], 105);

        app.apply_safe_preset(SafePresetKind::QuietUndervolt);
        assert!(app.fan_zero_rpm_enable);
        assert!(app.zero_rpm);
    }
}
