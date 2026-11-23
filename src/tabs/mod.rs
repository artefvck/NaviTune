pub mod advance_linux;
pub mod advance_macos;
pub mod advance;
pub mod advance_windows;
pub mod curve;
pub mod fan;
pub mod features;
pub mod frequency;
pub mod log_capture;
pub mod dump_audit;
pub mod native_offset;
pub mod log_offset_audit;
pub mod log_struct_window;
pub mod log;
pub mod more_power_settings;
pub mod more;
pub mod more_throttler;
pub mod oc_limits;
pub mod power;
pub mod safe_tune;

impl crate::PowerPlayApp {
    pub fn is_card_usable(&self, card: crate::ppt_arch::PptCard) -> bool {
        if matches!(card, crate::ppt_arch::PptCard::UlvVoltage)
            && matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega20)
            && self.raw_buffer.len() < 1900
        {
            return false;
        }
        if matches!(card, crate::ppt_arch::PptCard::UlvVoltage)
            && matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega20)
            && self.raw_buffer.len() < 1900
        {
            return false;
        }
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        if !arch.supports_card(card) {
            return false;
        }
        match card {
            crate::ppt_arch::PptCard::PowerAndCurrent if arch.is_gcn_subtable() => {
                self.power_limit_w > 0 || self.tdc_gfx > 0
            }
            crate::ppt_arch::PptCard::MemoryVoltage if arch.is_gcn_subtable() => {
                self.mem_dpm_volt.iter().any(|v| v[0] > 0 || v[1] > 0)
            }
            crate::ppt_arch::PptCard::ClockLimits if arch.is_gcn_subtable() => {
                self.freq_gfx[0] > 0 || self.freq_gfx[1] > 0
            }
            crate::ppt_arch::PptCard::MemoryDpm if arch.is_gcn_subtable() => {
                self.mem_dpm_freq.iter().any(|d| d[0] > 0)
            }
            crate::ppt_arch::PptCard::CoreVoltage if arch.is_gcn_subtable() => {
                self.volt_gfx[0] > 0 || self.volt_gfx[1] > 0
            }
            _ => true,
        }
    }
}
