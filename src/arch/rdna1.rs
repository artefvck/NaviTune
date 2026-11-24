pub use crate::ppt_arch::PptArch;
use core::mem::{offset_of, size_of};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AtomCommonTableHeaderR1 {
    pub structure_size: u16,
    pub table_format_revision: u8,
    pub table_content_revision: u8,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NaviRDNA1Chip {
    Navi10,
    Navi12,
    Navi14,
    UnknownRdna1,
}

impl NaviRDNA1Chip {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Navi10 => "Navi 10 (RX 5600 / 5700 XT / Pro W5700)",
            Self::Navi12 => "Navi 12 (Radeon Pro V520 / 5600M)",
            Self::Navi14 => "Navi 14 (RX 5300 / 5500 XT / Pro W5500)",
            Self::UnknownRdna1 => "RDNA 1 (SMU 11.0)",
        }
    }

    pub fn from_did_and_clock(did: u16, od_gfx_max: u32) -> Self {
        match did {
            0x7310 | 0x7312 | 0x7318 | 0x7319 | 0x731A | 0x731B | 0x731E | 0x731F => Self::Navi10,
            0x7360 | 0x7362 => Self::Navi12,
            0x7340 | 0x7341 | 0x7347 | 0x734F => Self::Navi14,
            _ if od_gfx_max >= 2000 => Self::Navi10,
            _ if od_gfx_max >= 1750 => Self::Navi14,
            _ => Self::UnknownRdna1,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomSmcDpmInfoV4_5 {
    pub header: AtomCommonTableHeaderR1,
    pub table_revision: u8,
    pub table_size: u16,
    pub golden_pp_id: u32,
    pub golden_revision: u32,
    pub format_id: u16,
    pub platform_caps: u32,
    pub thermal_controller_type: u8,
    pub board_reserved: [u8; 3],
    pub small_power_limit1: u16,
    pub small_power_limit2: u16,
    pub boost_power_limit: u16,
    pub od_turbo_power_limit: u16,
    pub od_power_save_power_limit: u16,
    pub software_shutdown_temp: u16,
    pub reserve: u16,
    pub od_feature_capabilities: [u32; 2],
    pub od_settings_max: [u32; 13],
    pub od_settings_min: [u32; 13],
    pub od_feature_count: u32,
    pub od_feature_flags: [u8; 16],

    pub smc_version: u32,
    pub features_to_run: u64,
    pub socket_power_limit_ac: [u16; 4],
    pub socket_power_limit_ac_tau: [u16; 4],
    pub socket_power_limit_dc: [u16; 4],
    pub socket_power_limit_dc_tau: [u16; 4],
    pub tdc_limit_gfx: u16,
    pub tdc_limit_soc: u16,
    pub tdc_limit_gfx_tau: u16,
    pub tdc_limit_soc_tau: u16,
    pub vddci_max: u16,
    pub mvdd_max: u16,
    pub load_line_slope_gfx: u16,
    pub load_line_slope_soc: u16,
    pub ulv_voltage_offset_soc: i16,
    pub ulv_voltage_offset_gfx: i16,
    pub ulv_smnclk_did: u8,
    pub ulv_mp1clk_did: u8,
    pub ulv_gfxclk_bypass: u8,
    pub gfxclk_slew_rate_quiescent: u8,
    pub min_voltage_ulv_gfx: u16,
    pub min_voltage_ulv_soc: u16,
    pub min_voltage_gfx: u16,
    pub min_voltage_soc: u16,
    pub max_voltage_gfx: u16,
    pub max_voltage_soc: u16,
    pub max_voltage_board: u16,
    pub padding_dpm_pre: u16,
    pub freq_table_gfx: [u16; 16],
    pub freq_table_vclk: [u16; 8],
    pub freq_table_dclk: [u16; 8],
    pub freq_table_socclk: [u16; 8],
    pub freq_table_uclk: [u16; 4],
    pub freq_table_dcefclk: [u16; 8],
    pub freq_table_dispclk: [u16; 8],
    pub freq_table_pixclk: [u16; 8],
    pub freq_table_phyclk: [u16; 8],
    pub freq_table_fclk: [u16; 8],
    pub padding_clks: [u32; 16],
    pub dpm_descriptor: [u8; 16],
    pub avfs_mid_payload: [u8; 1140],
    pub temp_limit: [u16; 7],
    pub fan_stop_temp: u16,
    pub fan_start_temp: u16,
    pub fan_gain: [u16; 7],
    pub fan_pwm_min: u16,
    pub fan_acoustic_limit_rpm: u16,
    pub fan_throttling_rpm: u16,
    pub fan_maximum_rpm: u16,
    pub fan_target_temperature: u16,
    pub fan_target_gfxclk: u16,
    pub fan_zero_rpm_enable: u8,
    pub fan_zero_rpm_stop_rpm: u8,
    pub board_tail_payload: [u8; 235],
}

const _: () = {
    assert!(std::mem::size_of::<AtomSmcDpmInfoV4_5>() == 1924);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, smc_version) == 0x0AB);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, socket_power_limit_ac) == 0x0B7);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, tdc_limit_gfx) == 0x0D7);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, min_voltage_gfx) == 0x0F3);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, freq_table_gfx) == 0x0FF);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, freq_table_uclk) == 0x14F);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, temp_limit) == 0x66B);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, fan_pwm_min) == 0x68B);
    assert!(offset_of!(AtomSmcDpmInfoV4_5, fan_zero_rpm_enable) == 0x697);
};

impl AtomSmcDpmInfoV4_5 {
    pub fn from_bytes(b: &[u8]) -> Option<&Self> {
        if b.len() < std::mem::size_of::<Self>() || b[2] != 0x0C || (b[3] != 0x00 && b[3] != 0x01) {
            return None;
        }
        Some(unsafe { &*(b.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || buf.len() >= 2400 || !matches!(buf.get(2), Some(0x0C | 0x0D)) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }

    pub fn detect_chip(&self, device_id: u16) -> NaviRDNA1Chip {
        let gfx_max = { self.od_settings_max[0] };
        NaviRDNA1Chip::from_did_and_clock(device_id, gfx_max)
    }
}

#[derive(Clone, Debug)]
pub struct Rdna1ParsedState {
    pub outer_sz: usize,
    pub smu_features: u64,
    pub power_limit_w: u16,
    pub tdc_soc: u16,
    pub tdc_gfx: u16,
    pub ulv_vmin_gfx: u16,
    pub ulv_vmin_soc: u16,
    pub min_volt_gfx: u16,
    pub min_volt_soc: u16,
    pub max_volt_gfx: u16,
    pub max_volt_soc: u16,
    pub curve_gfx: [f32; 3],
    pub gfx_min: u16,
    pub gfx_max: u16,
    pub soc_min: u16,
    pub soc_max: u16,
    pub uclk_dpm: [u16; 4],
    pub fclk_min: u16,
    pub fclk_max: u16,
    pub mem_vddci: [u16; 4],
    pub mem_mvdd: [u16; 4],
    pub temp_limit: [u16; 7],
    pub fan_stop_temp: u16,
    pub fan_start_temp: u16,
    pub fan_pwm_min: u16,
    pub fan_acoustic_rpm: u16,
    pub fan_max_rpm: u16,
    pub fan_target_temp: u16,
    pub fan_target_gfx: u16,
    pub fan_zero_rpm: bool,
    pub od_settings_min: [u32; 5],
    pub od_settings_max: [u32; 5],
    pub od_feature_flags: [u8; 16],
    pub od_gfx_max: u32,
    pub od_uclk_max: u32,
}

impl Rdna1ParsedState {
    pub fn parse(b: &[u8]) -> Option<Self> {
        if b.len() < 1674 || (b[2] != 0x0C && b[2] != 0x0D) {
            return None;
        }
        let outer_sz = u16::from_le_bytes([b[5], b[6]]) as usize;
        if outer_sz != 0x01E2 || outer_sz + 0x2A6 > b.len() {
            return None;
        }
        let inn = &b[outer_sz..];
        let u16_at = |s: &[u8], o: usize| u16::from_le_bytes([s[o], s[o + 1]]);
        let u32_at = |s: &[u8], o: usize| u32::from_le_bytes([s[o], s[o + 1], s[o + 2], s[o + 3]]);
        let f32_at = |s: &[u8], o: usize| f32::from_le_bytes([s[o], s[o + 1], s[o + 2], s[o + 3]]);
        let smu_lo = u32_at(inn, 0x004) as u64;
        let smu_hi = u32_at(inn, 0x008) as u64;
        Some(Self {
            outer_sz,
            smu_features: smu_lo | (smu_hi << 32),
            power_limit_w: u16_at(inn, 0x00C),
            tdc_soc: u16_at(inn, 0x02C),
            tdc_gfx: u16_at(inn, 0x030),
            ulv_vmin_gfx: u16_at(inn, 0x064),
            ulv_vmin_soc: u16_at(inn, 0x066),
            min_volt_gfx: u16_at(inn, 0x068),
            min_volt_soc: u16_at(inn, 0x06A),
            max_volt_gfx: u16_at(inn, 0x06C),
            max_volt_soc: u16_at(inn, 0x06E),
            curve_gfx: [f32_at(inn, 0x080), f32_at(inn, 0x084), f32_at(inn, 0x088)],
            gfx_min: u16_at(inn, 0x14C),
            gfx_max: u16_at(inn, 0x14E),
            soc_min: u16_at(inn, 0x16C),
            soc_max: u16_at(inn, 0x16E),
            uclk_dpm: [u16_at(inn, 0x19C), u16_at(inn, 0x19E), u16_at(inn, 0x1A0), u16_at(inn, 0x1A2)],
            fclk_min: u16_at(inn, 0x1B4),
            fclk_max: u16_at(inn, 0x1B6),
            mem_vddci: [u16_at(inn, 0x240), u16_at(inn, 0x242), u16_at(inn, 0x244), u16_at(inn, 0x246)],
            mem_mvdd: [u16_at(inn, 0x248), u16_at(inn, 0x24A), u16_at(inn, 0x24C), u16_at(inn, 0x24E)],
            temp_limit: [u16_at(inn, 0x260), u16_at(inn, 0x262), u16_at(inn, 0x264), u16_at(inn, 0x266), u16_at(inn, 0x268), u16_at(inn, 0x26A), u16_at(inn, 0x26C)],
            fan_stop_temp: u16_at(inn, 0x27C),
            fan_start_temp: u16_at(inn, 0x27E),
            fan_pwm_min: u16_at(inn, 0x294),
            fan_acoustic_rpm: u16_at(inn, 0x296),
            fan_max_rpm: u16_at(inn, 0x298),
            fan_target_temp: u16_at(inn, 0x29C),
            fan_target_gfx: u16_at(inn, 0x29E),
            fan_zero_rpm: inn[0x2A3] != 0,
            od_settings_min: [u32_at(b, 0x162), u32_at(b, 0x16E), u32_at(b, 0x186), u32_at(b, 0x18A), u32_at(b, 0x18E)],
            od_settings_max: [u32_at(b, 0x0E2), u32_at(b, 0x0EE), u32_at(b, 0x106), u32_at(b, 0x10A), u32_at(b, 0x10E)],
            od_feature_flags: [
                b[0x0C2], b[0x0C3], b[0x0C4], b[0x0C5],
                b[0x0C6], b[0x0C7], b[0x0C8], b[0x0C9],
                b[0x0CA], b[0x0CB], b[0x0CC], b[0x0CD],
                b[0x0CE], b[0x0CF], b[0x0D0], b[0x0D1],
            ],
            od_gfx_max: u32_at(b, 0x0E2),
            od_uclk_max: u32_at(b, 0x0EE),
        })
    }

    pub fn write_to_buf(&self, b: &mut [u8]) {
        if b.len() < self.outer_sz + 0x2A6 {
            return;
        }
        let max_offs = [0x0E2usize, 0x0EE, 0x106, 0x10A, 0x10E];
        let min_offs = [0x162usize, 0x16E, 0x186, 0x18A, 0x18E];
        for i in 0..5 {
            let max_v = if i == 0 && self.od_gfx_max != self.od_settings_max[0] { self.od_gfx_max } else if i == 1 && self.od_uclk_max != self.od_settings_max[1] { self.od_uclk_max } else { self.od_settings_max[i] };
            b[max_offs[i]..max_offs[i] + 4].copy_from_slice(&max_v.to_le_bytes());
            b[min_offs[i]..min_offs[i] + 4].copy_from_slice(&self.od_settings_min[i].to_le_bytes());
        }
        b[0x0C2..0x0D2].copy_from_slice(&self.od_feature_flags);
        let inn = &mut b[self.outer_sz..];
        let put_u16 = |s: &mut [u8], o: usize, v: u16| s[o..o + 2].copy_from_slice(&v.to_le_bytes());
        let put_u32 = |s: &mut [u8], o: usize, v: u32| s[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let put_f32 = |s: &mut [u8], o: usize, v: f32| s[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put_u32(inn, 0x004, self.smu_features as u32);
        put_u32(inn, 0x008, (self.smu_features >> 32) as u32);
        put_u16(inn, 0x00C, self.power_limit_w);
        put_u16(inn, 0x02C, self.tdc_soc);
        put_u16(inn, 0x030, self.tdc_gfx);
        put_u16(inn, 0x064, self.ulv_vmin_gfx);
        put_u16(inn, 0x066, self.ulv_vmin_soc);
        put_u16(inn, 0x068, self.min_volt_gfx);
        put_u16(inn, 0x06A, self.min_volt_soc);
        put_u16(inn, 0x06C, self.max_volt_gfx);
        put_u16(inn, 0x06E, self.max_volt_soc);
        put_f32(inn, 0x080, self.curve_gfx[0]);
        put_f32(inn, 0x084, self.curve_gfx[1]);
        put_f32(inn, 0x088, self.curve_gfx[2]);
        put_u16(inn, 0x14C, self.gfx_min);
        put_u16(inn, 0x14E, self.gfx_max);
        put_u16(inn, 0x16C, self.soc_min);
        put_u16(inn, 0x16E, self.soc_max);
        for i in 0..4 {
            put_u16(inn, 0x19C + i * 2, self.uclk_dpm[i]);
            put_u16(inn, 0x240 + i * 2, self.mem_vddci[i]);
            put_u16(inn, 0x248 + i * 2, self.mem_mvdd[i]);
        }
        put_u16(inn, 0x1B4, self.fclk_min);
        put_u16(inn, 0x1B6, self.fclk_max);
        for i in 0..7 { put_u16(inn, 0x260 + i * 2, self.temp_limit[i]); }
        put_u16(inn, 0x27C, self.fan_stop_temp);
        put_u16(inn, 0x27E, self.fan_start_temp);
        put_u16(inn, 0x294, self.fan_pwm_min);
        put_u16(inn, 0x296, self.fan_acoustic_rpm);
        put_u16(inn, 0x298, self.fan_max_rpm);
        put_u16(inn, 0x29C, self.fan_target_temp);
        put_u16(inn, 0x29E, self.fan_target_gfx);
        inn[0x2A3] = if self.fan_zero_rpm { if inn[0x2A3] != 0 { inn[0x2A3] } else { 1 } } else { 0 };
    }
}

#[cfg(test)]
mod arch_codec_resilience_tests {
    #[test]
    fn parser_handles_truncated_and_skeleton_buffers_without_panic() {
        for len in 0..=16 {
            let short = vec![0u8; len];
            let _ = super::AtomSmcDpmInfoV4_5::from_bytes(&short);
        }
        let cases = [
            ("rdna1_v4_5", 13u8, 13u8, 1u8, 1924usize),
        ];
        for (_label, trev, frev, crev, size) in cases {
            let mut buf = vec![0u8; size];
            let sz = (size as u16).to_le_bytes();
            buf[0] = sz[0];
            buf[1] = sz[1];
            buf[2] = frev;
            buf[3] = crev;
            buf[4] = trev;
            let _ = super::AtomSmcDpmInfoV4_5::from_bytes(&buf);
        }
    }
}
