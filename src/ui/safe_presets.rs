use crate::ppt_arch::PptArch;
use crate::PowerPlayApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafePresetKind {
    QuietUndervolt,
    BalancedOc,
    AutoFixRisks,
}

impl PowerPlayApp {
    pub(crate) fn apply_safe_preset(&mut self, kind: SafePresetKind) {
        if self.raw_buffer.is_empty() {
            return;
        }
        let arch = PptArch::detect(&self.raw_buffer);
        let is_gcn = arch.is_gcn_subtable();
        let is_rdna3 = arch == PptArch::Rdna3;

        let (v_cap, pwr_cap, tdc_cap, uclk_cap, fan_cap) = match arch {
            PptArch::Polaris => (1150, 185, 160, 2100, 80),
            PptArch::Vega10 => (1100, 260, 240, 1050, 82),
            PptArch::Vega20 => (1100, 280, 250, 1100, 85),
            PptArch::Rdna1 => (1150, 210, 195, 920, 85),
            PptArch::Rdna2 => (1150, 285, 240, 1000, 85),
            PptArch::Rdna3 => (1150, 340, 420, 1250, 90),
            _ => (1150, 250, 220, 1000, 85),
        };

        match kind {
            SafePresetKind::QuietUndervolt => {
                if !is_rdna3 && self.volt_gfx[1] > 850 {
                    self.volt_gfx[1] = (self.volt_gfx[1] - 65).clamp(780, v_cap);
                }
                if self.power_limit_w > 60 {
                    self.power_limit_w = ((self.power_limit_w as f32 * 0.90).round() as i32).min(pwr_cap);
                    self.power_limit_ac[0] = self.power_limit_w;
                }
                if self.fan_target_temp > 55 {
                    self.fan_target_temp = (self.fan_target_temp - 4).clamp(55, fan_cap);
                }
                if arch != PptArch::Polaris && self.fan_start_temp > 0 {
                    self.fan_zero_rpm_enable = true;
                }
                self.status_msg = format!("Applied Quiet Undervolt preset ({:?})", arch);
            }
            SafePresetKind::BalancedOc => {
                if self.oc_gfx_clock[1] > self.freq_gfx[1] + 40 {
                    let target = ((self.oc_gfx_clock[1] as f32 * 0.97).round() as i32).max(self.freq_gfx[1]);
                    self.freq_gfx[1] = target.min(self.oc_gfx_clock[1]);
                }
                if self.power_limit_w > 50 {
                    let boosted = (self.power_limit_w as f32 * 1.10).round() as i32;
                    self.power_limit_w = boosted.min(pwr_cap);
                    self.power_limit_ac[0] = self.power_limit_w;
                }
                if !is_rdna3 && self.volt_gfx[1] > 0 {
                    self.volt_gfx[1] = self.volt_gfx[1].min(v_cap);
                }
                if self.fan_target_temp > 60 {
                    self.fan_target_temp = (self.fan_target_temp - 3).clamp(58, fan_cap);
                }
                self.status_msg = format!("Applied Balanced OC preset ({:?})", arch);
            }
            SafePresetKind::AutoFixRisks => {
                if self.oc_gfx_clock[1] > 0 && self.freq_gfx[1] > self.oc_gfx_clock[1] {
                    self.freq_gfx[1] = self.oc_gfx_clock[1];
                }
                if self.freq_gfx[0] >= self.freq_gfx[1] && self.freq_gfx[1] > 300 {
                    self.freq_gfx[0] = (self.freq_gfx[1] - 100).max(300);
                }
                for d in &mut self.mem_dpm_freq {
                    if d[0] > uclk_cap {
                        d[0] = uclk_cap;
                    }
                    if self.oc_mem_clock[1] > 0 && d[0] > self.oc_mem_clock[1] {
                        d[0] = self.oc_mem_clock[1];
                    }
                }
                if !is_rdna3 && self.volt_gfx[1] > 0 {
                    if self.volt_gfx[1] > v_cap {
                        self.volt_gfx[1] = v_cap;
                    }
                    if self.volt_gfx[0] > self.volt_gfx[1] {
                        self.volt_gfx[0] = (self.volt_gfx[1] - 150).max(700);
                    }
                }
                if self.power_limit_w > pwr_cap {
                    self.power_limit_w = pwr_cap;
                    self.power_limit_ac[0] = pwr_cap;
                }
                if self.tdc_gfx > tdc_cap {
                    self.tdc_gfx = tdc_cap;
                }
                if !is_gcn && self.fan_stop_temp > 0 && self.fan_stop_temp >= self.fan_start_temp {
                    self.fan_stop_temp = (self.fan_start_temp - 5).max(30);
                }
                if self.fan_target_temp > fan_cap {
                    self.fan_target_temp = fan_cap;
                }
                self.status_msg = format!("Auto-Fixed out-of-spec values to safe {:?} limits", arch);
            }
        }
    }
}
