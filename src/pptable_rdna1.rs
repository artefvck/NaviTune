use core::mem::{offset_of, size_of};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AtomCommonTableHeaderR1 {
    pub structure_size: u16,
    pub table_format_revision: u8,
    pub table_content_revision: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PptArch {
    Vega10,
    Vega20,
    Rdna1,
    Rdna2,
    Rdna3,
    Unknown,
}

impl PptArch {
    pub fn detect(buf: &[u8]) -> Self {
        if crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(buf).is_some() {
            Self::Rdna3
        } else if crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(buf).is_some() {
            Self::Rdna2
        } else if AtomSmcDpmInfoV4_5::from_bytes(buf).is_some() {
            Self::Rdna1
        } else if crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(buf).is_some() {
            Self::Vega20
        } else if crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(buf).is_some() {
            Self::Vega10
        } else {
            Self::Unknown
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vega10 => "Vega 10",
            Self::Vega20 => "Vega 20",
            Self::Rdna1 => "RDNA 1",
            Self::Rdna2 => "RDNA 2",
            Self::Rdna3 => "RDNA 3",
            Self::Unknown => "Unknown",
        }
    }
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
    pub ulv_voltage_offset_soc: u16,
    pub ulv_voltage_offset_gfx: u16,
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
    assert!(size_of::<AtomSmcDpmInfoV4_5>() == 1924);
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
    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() || buf.len() >= 2400 || !matches!(buf.get(2), Some(0x0C | 0x0D)) {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
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
