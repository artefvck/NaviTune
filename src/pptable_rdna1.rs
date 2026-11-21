use core::mem::size_of;

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
    pub fn display_name(self) -> &str {
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
    pub smc_dpm_payload: [u8; 1753],
}

const _: () = assert!(size_of::<AtomSmcDpmInfoV4_5>() == 1924);

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
