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
}
