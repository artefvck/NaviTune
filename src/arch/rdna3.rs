//! Official AMD `atomfirmware.h` (`struct atom_smc_dpm_info_v4_12` / `v4_13`, 5,424 bytes)
//! and `smu13_driver_if_v13_0_0.h` / `v13_0_7.h` (RDNA 3: Navi 31, Navi 32, Navi 33).

use std::mem::{offset_of, size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaviRDNA3Chip {
    Navi31PlumBonito,
    Navi32WheatNas,
    Navi33Hotbeige,
    UnknownRdna3,
}

impl NaviRDNA3Chip {
    pub fn from_header_and_did(frev: u8, crev: u8, size: usize, did: u16) -> Self {
        match did {
            0x744C | 0x7448 | 0x745E => return Self::Navi31PlumBonito,
            0x747E | 0x7470 | 0x7479 => return Self::Navi32WheatNas,
            0x7480 | 0x7483 | 0x7489 => return Self::Navi33Hotbeige,
            _ => {}
        }
        match (frev, crev, size) {
            (0x14, 0x00..=0x03, 5424) => Self::Navi31PlumBonito,
            (0x15, _, 5424) => Self::Navi32WheatNas,
            (0x16, _, 5424) | (0x14, 0x04..=0x08, _) => Self::Navi33Hotbeige,
            _ => Self::UnknownRdna3,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Navi31PlumBonito => "Navi 31 (Plum Bonito — RX 7900 XTX / XT / GRE)",
            Self::Navi32WheatNas => "Navi 32 (Wheat Nas — RX 7800 XT / 7700 XT)",
            Self::Navi33Hotbeige => "Navi 33 (Hotbeige — RX 7600 / 7600 XT)",
            Self::UnknownRdna3 => "RDNA 3 (Navi 31 / 32 / 33)",
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AtomCommonTableHeader {
    pub structure_size: u16,  // 0x000..0x002 (5424 = 0x1530)
    pub format_revision: u8,  // 0x002 (0x14 = N31/N32, 0x15 = N32, 0x16 = N33)
    pub content_revision: u8, // 0x003
    pub table_revision: u8,   // 0x004 (0x03)
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomSmcDpmInfoV4_12 {
    pub header: AtomCommonTableHeader,          // 0x000..0x005
    pub _od_pre: [u8; 0x0DF - 0x005],           // 0x005..0x0DF
    pub od_feature_flags: [u8; 20],             // 0x0DF..0x0F3
    pub _od_pad0: [u8; 0x0FF - 0x0F3],          // 0x0F3..0x0FF
    pub od_gfx_max: u32,                        // 0x0FF..0x103
    pub od_gfx_max_2: u32,                      // 0x103..0x107
    pub od_mem_max: u32,                        // 0x107..0x10B
    pub od_mem_max_2: u32,                      // 0x10B..0x10F
    pub od_pwr_max: u32,                        // 0x10F..0x113
    pub od_fan_max: u32,                        // 0x113..0x117
    pub od_acoustic_max: u32,                   // 0x117..0x11B
    pub od_temp_target: u32,                    // 0x11B..0x11F
    pub od_temp_max: u32,                       // 0x11F..0x123
    pub _od_min_pad: [u8; 0x1FF - 0x123],       // 0x123..0x1FF
    pub od_gfx_min: u32,                        // 0x1FF..0x203
    pub od_gfx_min_2: u32,                      // 0x203..0x207
    pub od_mem_min: u32,                        // 0x207..0x20B
    pub od_mem_min_2: u32,                      // 0x20B..0x20F
    pub od_pwr_min: u32,                        // 0x20F..0x213
    pub od_fan_min: u32,                        // 0x213..0x217
    pub od_acoustic_min: u32,                   // 0x217..0x21B
    pub _smc_pre_hdr: [u8; 0x340 - 0x21B],      // 0x21B..0x340
    pub smc_pptable_if_version: u32,            // 0x340..0x344
    pub features_to_run: u64,                   // 0x344..0x34C
    pub _smc_pwr_pad: [u8; 0x350 - 0x34C],      // 0x34C..0x350
    pub socket_power_limit_ac: u16,             // 0x350..0x352
    pub socket_power_limit_ac_2: u16,           // 0x352..0x354
    pub _smc_tdc_pad: [u8; 0x366 - 0x354],      // 0x354..0x366
    pub tdc_limit_gfx: u16,                     // 0x366..0x368
    pub tdc_limit_soc: u16,                     // 0x368..0x36A
    pub _smc_temp_pad: [u8; 0x372 - 0x36A],     // 0x36A..0x372
    pub temp_limit: [u16; 9],                   // 0x372..0x384
    pub _smc_volt_pad: [u8; 0x3B4 - 0x384],     // 0x384..0x3B4
    pub min_voltage_gfx: u16,                   // 0x3B4..0x3B6
    pub max_voltage_gfx: u16,                   // 0x3B6..0x3B8
    pub min_voltage_soc: u16,                   // 0x3B8..0x3BA
    pub max_voltage_soc: u16,                   // 0x3BA..0x3BC
    pub _smc_ulv_pad: [u8; 0x3C4 - 0x3BC],      // 0x3BC..0x3C4
    pub ulv_voltage_vmin_gfx: u16,              // 0x3C4..0x3C6
    pub ulv_voltage_vmin_soc: u16,              // 0x3C6..0x3C8
    pub _smc_clk_pad: [u8; 0x5EC - 0x3C8],      // 0x3C8..0x5EC
    pub freq_gfx_min: u16,                      // 0x5EC..0x5EE
    pub freq_gfx_max: u16,                      // 0x5EE..0x5F0
    pub _smc_soc_pad: [u8; 0x60E - 0x5F0],      // 0x5F0..0x60E
    pub freq_soc_max: u16,                      // 0x60E..0x610
    pub _smc_fclk_pad: [u8; 0x62C - 0x610],     // 0x610..0x62C
    pub freq_fclk_min: u16,                     // 0x62C..0x62E
    pub freq_fclk_max: u16,                     // 0x62E..0x630
    pub _smc_uclk_pad: [u8; 0x63C - 0x630],     // 0x630..0x63C
    pub freq_uclk: [u16; 4],                    // 0x63C..0x644
    pub _smc_memv_pad: [u8; 0x7D4 - 0x644],     // 0x644..0x7D4
    pub mem_vddci: [u16; 4],                    // 0x7D4..0x7DC
    pub mem_mvdd: [u16; 4],                     // 0x7DC..0x7E4
    pub _smc_pcie_pad: [u8; 0x810 - 0x7E4],     // 0x7E4..0x810
    pub pcie_gen_speed: [u8; 3],                // 0x810..0x813
    pub pcie_lane_count: [u8; 3],               // 0x813..0x816
    pub lclk_freq: [u16; 3],                    // 0x816..0x81C
    pub _smc_fan_pad0: [u8; 0x81E - 0x81C],     // 0x81C..0x81E
    pub fan_stop_temp: u16,                     // 0x81E..0x820
    pub _smc_fan_pad1: [u8; 0x838 - 0x820],     // 0x820..0x838
    pub fan_start_temp: u16,                    // 0x838..0x83A
    pub _smc_fan_pad2: [u8; 0x86A - 0x83A],     // 0x83A..0x86A
    pub fan_throttling_rpm: u16,                // 0x86A..0x86C
    pub fan_pwm_min: u16,                       // 0x86C..0x86E
    pub fan_acoustic_limit: u16,                // 0x86E..0x870
    pub fan_max_rpm: u16,                       // 0x870..0x872
    pub fan_max_rpm_2: u16,                     // 0x872..0x874
    pub _smc_fan_pad3: [u8; 0x876 - 0x874],     // 0x874..0x876
    pub fan_target_gfx_clock: u16,              // 0x876..0x878
    pub fan_target_temp: u16,                   // 0x878..0x87A
    pub _smc_fan_pad4: [u8; 0x87C - 0x87A],     // 0x87A..0x87C
    pub fan_zero_rpm_enable: u8,                // 0x87C..0x87D
    pub _tail: [u8; 5424 - 0x87D],              // 0x87D..0x1530
}

const _: () = assert!(size_of::<AtomSmcDpmInfoV4_12>() == 5424);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, od_gfx_min) == 0x1FF);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, tdc_limit_gfx) == 0x366);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, mem_vddci) == 0x7D4);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, pcie_gen_speed) == 0x810);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, fan_stop_temp) == 0x81E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, fan_start_temp) == 0x838);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, fan_throttling_rpm) == 0x86A);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_12, fan_zero_rpm_enable) == 0x87C);

impl AtomSmcDpmInfoV4_12 {
    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() || !matches!(buf.get(2), Some(0x14..=0x16)) {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || !matches!(buf.get(2), Some(0x14..=0x16)) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }

    pub fn detect_chip(&self, device_id: u16) -> NaviRDNA3Chip {
        let mut chip = NaviRDNA3Chip::from_header_and_did(
            self.header.format_revision,
            self.header.content_revision,
            size_of::<Self>(),
            device_id,
        );
        if chip == NaviRDNA3Chip::Navi31PlumBonito && device_id == 0 && self.socket_power_limit_ac <= 230 {
            chip = NaviRDNA3Chip::Navi32WheatNas;
        }
        chip
    }

    pub fn effective_gfx_min_offset_and_val(&self, buf: &[u8]) -> (usize, i32) {
        if self.header.format_revision >= 0x16 && self.freq_gfx_min < 500 && buf.len() > 0x695 {
            (0x694usize, u16::from_le_bytes([buf[0x694], buf[0x695]]) as i32)
        } else {
            (offset_of!(Self, freq_gfx_min), self.freq_gfx_min as i32)
        }
    }
}

pub fn field_offset_by_name(name: &str) -> usize {
    match name {
        "od_features" => offset_of!(AtomSmcDpmInfoV4_12, od_feature_flags),
        "od_gfx" => offset_of!(AtomSmcDpmInfoV4_12, od_gfx_max),
        "od_mem" => offset_of!(AtomSmcDpmInfoV4_12, od_mem_max),
        "od_pwr" => offset_of!(AtomSmcDpmInfoV4_12, od_pwr_max),
        "od_fan" => offset_of!(AtomSmcDpmInfoV4_12, od_fan_max),
        "od_acoustic" => offset_of!(AtomSmcDpmInfoV4_12, od_acoustic_max),
        "od_temp" => offset_of!(AtomSmcDpmInfoV4_12, od_temp_target),
        "od_gfx_min" => offset_of!(AtomSmcDpmInfoV4_12, od_gfx_min),
        "od_mem_min" => offset_of!(AtomSmcDpmInfoV4_12, od_mem_min),
        "od_pwr_min" => offset_of!(AtomSmcDpmInfoV4_12, od_pwr_min),
        "od_fan_min" => offset_of!(AtomSmcDpmInfoV4_12, od_fan_min),
        "od_acoustic_min" => offset_of!(AtomSmcDpmInfoV4_12, od_acoustic_min),
        "smc_if_ver" => offset_of!(AtomSmcDpmInfoV4_12, smc_pptable_if_version),
        "smu_features" => offset_of!(AtomSmcDpmInfoV4_12, features_to_run),
        "pwr_limit_w" => offset_of!(AtomSmcDpmInfoV4_12, socket_power_limit_ac),
        "tdc_gfx" => offset_of!(AtomSmcDpmInfoV4_12, tdc_limit_gfx),
        "tdc_soc" => offset_of!(AtomSmcDpmInfoV4_12, tdc_limit_soc),
        "temp_limit" => offset_of!(AtomSmcDpmInfoV4_12, temp_limit),
        "volt_gfx" => offset_of!(AtomSmcDpmInfoV4_12, min_voltage_gfx),
        "volt_soc" => offset_of!(AtomSmcDpmInfoV4_12, min_voltage_soc),
        "ulv_gfx" => offset_of!(AtomSmcDpmInfoV4_12, ulv_voltage_vmin_gfx),
        "freq_gfx" => offset_of!(AtomSmcDpmInfoV4_12, freq_gfx_min),
        "freq_soc" => offset_of!(AtomSmcDpmInfoV4_12, freq_soc_max),
        "freq_fclk" => offset_of!(AtomSmcDpmInfoV4_12, freq_fclk_min),
        "mem_dpm_freq" => offset_of!(AtomSmcDpmInfoV4_12, freq_uclk),
        "mem_dpm_volt" => offset_of!(AtomSmcDpmInfoV4_12, mem_vddci),
        "pcie" => offset_of!(AtomSmcDpmInfoV4_12, pcie_gen_speed),
        "lclk" => offset_of!(AtomSmcDpmInfoV4_12, lclk_freq),
        "fan_stop_temp" => offset_of!(AtomSmcDpmInfoV4_12, fan_stop_temp),
        "fan_start_temp" => offset_of!(AtomSmcDpmInfoV4_12, fan_start_temp),
        "fan_throttling" => offset_of!(AtomSmcDpmInfoV4_12, fan_throttling_rpm),
        "fan_pwm_min" => offset_of!(AtomSmcDpmInfoV4_12, fan_pwm_min),
        "fan_max_rpm" => offset_of!(AtomSmcDpmInfoV4_12, fan_max_rpm),
        "fan_target_temp" => offset_of!(AtomSmcDpmInfoV4_12, fan_target_temp),
        "fan_zero_rpm" => offset_of!(AtomSmcDpmInfoV4_12, fan_zero_rpm_enable),
        _ => 0,
    }
}

#[cfg(test)]
mod arch_codec_resilience_tests {
    #[test]
    fn parser_handles_truncated_and_skeleton_buffers_without_panic() {
        for len in 0..=16 {
            let short = vec![0u8; len];
            let _ = super::AtomSmcDpmInfoV4_12::from_bytes(&short);
        }
        let cases = [
            ("rdna3_v4_12", 20u8, 20u8, 1u8, 5424usize),
        ];
        for (_label, trev, frev, crev, size) in cases {
            let mut buf = vec![0u8; size];
            let sz = (size as u16).to_le_bytes();
            buf[0] = sz[0];
            buf[1] = sz[1];
            buf[2] = frev;
            buf[3] = crev;
            buf[4] = trev;
            let _ = super::AtomSmcDpmInfoV4_12::from_bytes(&buf);
        }
    }
}
