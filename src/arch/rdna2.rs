//! Official AMD `atomfirmware.h` (`struct atom_smc_dpm_info_v4_9`)
//! and `smu11_driver_if_navi23.h` (`PPTable_t`, Dimgrey Cavefish / Navi 23, 2,470 bytes).

use std::mem::{offset_of, size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaviRDNA2Chip {
    Navi21SiennaCichlid,
    Navi22NavyFlounder,
    Navi23DimgreyCavefish,
    Navi24BeigeGoby,
}

impl NaviRDNA2Chip {
    pub fn from_header(size: u16, format_rev: u8, table_rev: u8, smc_ver: u32) -> Self {
        if size >= 2486 || format_rev >= 0x13 {
            Self::Navi24BeigeGoby
        } else if table_rev == 0x02 && smc_ver == 0x06 {
            Self::Navi23DimgreyCavefish
        } else if smc_ver == 0x05 {
            Self::Navi22NavyFlounder
        } else {
            Self::Navi21SiennaCichlid
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Navi21SiennaCichlid => "Navi 21 (Sienna Cichlid)",
            Self::Navi22NavyFlounder => "Navi 22 (Navy Flounder)",
            Self::Navi23DimgreyCavefish => "Navi 23 (Dimgrey Cavefish)",
            Self::Navi24BeigeGoby => "Navi 24 (Beige Goby)",
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomCommonTableHeader {
    pub structure_size: u16,
    pub format_revision: u8,
    pub content_revision: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct OdFanPoint {
    pub temp: u32,
    pub pwm: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct QuadraticCoeff {
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

#[allow(dead_code)]
pub type DimgreyCavefishPPTable = AtomSmcDpmInfoV4_9;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomSmcDpmInfoV4_9 {
    pub header: AtomCommonTableHeader,          // 0x000..0x004
    pub table_revision: u8,                     // 0x004
    pub _od8_hdr_caps: [u8; 0x0C2 - 0x005],     // 0x005..0x0C2
    pub od_feature_flags: [u8; 16],             // 0x0C2..0x0D2
    pub _od8_max_res0: [u8; 0x0E2 - 0x0D2],     // 0x0D2..0x0E2
    pub od_gfx_max: [u32; 2],                   // 0x0E2..0x0EA
    pub _od8_max_res1: [u8; 0x0FA - 0x0EA],     // 0x0EA..0x0FA
    pub od_mem_max: [u32; 2],                   // 0x0FA..0x102
    pub od_pwr_max: u32,                        // 0x102..0x106
    pub od_fan_max: u32,                        // 0x106..0x10A
    pub od_acoustic_max: u32,                   // 0x10A..0x10E
    pub _od8_max_res2: [u8; 0x116 - 0x10E],     // 0x10E..0x116
    pub od_mem_timing_ctrl: u32,                // 0x116..0x11A
    pub od_pwr_mode_ctrl: u32,                  // 0x11A..0x11E
    pub _od8_max_res3: [u8; 0x126 - 0x11E],     // 0x11E..0x126
    pub od_zero_rpm_ctrl: u32,                  // 0x126..0x12A
    pub od_fan_curve_max: [OdFanPoint; 5],      // 0x12A..0x152
    pub _od8_min_res0: [u8; 0x1E2 - 0x152],     // 0x152..0x1E2
    pub od_gfx_min: [u32; 2],                   // 0x1E2..0x1EA
    pub _od8_min_res1: [u8; 0x1FA - 0x1EA],     // 0x1EA..0x1FA
    pub od_mem_min: [u32; 2],                   // 0x1FA..0x202
    pub od_pwr_min: u32,                        // 0x202..0x206
    pub od_fan_min: u32,                        // 0x206..0x20A
    pub od_acoustic_min: u32,                   // 0x20A..0x20E
    pub _od8_min_res2: [u8; 0x22A - 0x20E],     // 0x20E..0x22A
    pub od_fan_curve_min: [OdFanPoint; 5],      // 0x22A..0x252
    pub _od8_tail: [u8; 0x2DA - 0x252],         // 0x252..0x2DA
    pub pm_quiet_limit: u32,                    // 0x2DA..0x2DE
    pub pm_balanced_limit: u32,                 // 0x2DE..0x2E2
    pub pm_turbo_limit: u32,                    // 0x2E2..0x2E6
    pub pm_rage_limit: u32,                     // 0x2E6..0x2EA
    pub pm_quiet_temp: u16,                     // 0x2EA..0x2EC
    pub pm_balanced_temp: u16,                  // 0x2EC..0x2EE
    pub pm_turbo_temp: u16,                     // 0x2EE..0x2F0
    pub pm_rage_temp: u16,                      // 0x2F0..0x2F2
    pub ac_quiet: [u16; 2],                     // 0x2F2..0x2F6
    pub ac_balanced: [u16; 2],                  // 0x2F6..0x2FA
    pub ac_turbo: [u16; 2],                     // 0x2FA..0x2FE
    pub ac_rage: [u16; 2],                      // 0x2FE..0x302
    pub _smc_pre_hdr: [u8; 0x322 - 0x302],      // 0x302..0x322
    pub smc_pptable_if_version: u32,            // 0x322..0x326
    pub features_to_run: u64,                   // 0x326..0x32E
    pub socket_power_limit_ac: [u16; 4],        // 0x32E..0x336 (SocketPowerLimitAc[PPT0..3])
    pub _smc_ppt_res0: [u8; 0x33E - 0x336],     // 0x336..0x33E
    pub socket_power_limit_dc: u16,             // 0x33E..0x340
    pub _smc_ppt_res1: [u8; 0x346 - 0x340],     // 0x340..0x346
    pub power_limit_ac: [u16; 4],               // 0x346..0x34E
    pub tdc_limit_gfx: u16,                     // 0x34E..0x350
    pub tdc_limit_soc: u16,                     // 0x350..0x352
    pub _smc_tdc_res: [u8; 0x356 - 0x352],      // 0x352..0x356
    pub temp_limit: [u16; 9],                   // 0x356..0x368
    pub _smc_temp_res: [u8; 0x39A - 0x368],     // 0x368..0x39A
    pub ulv_voltage_vmin_gfx: u16,              // 0x39A..0x39C
    pub ulv_voltage_vmin_soc: u16,              // 0x39C..0x39E
    pub ulv_voltage_offset_gfx: u16,            // 0x39E..0x3A0
    pub ulv_voltage_offset_soc: u16,            // 0x3A0..0x3A2
    pub _smc_volt_res0: [u8; 0x3A6 - 0x3A2],    // 0x3A2..0x3A6
    pub min_voltage_gfx: u16,                   // 0x3A6..0x3A8
    pub min_voltage_soc: u16,                   // 0x3A8..0x3AA
    pub max_voltage_gfx: u16,                   // 0x3AA..0x3AC
    pub max_voltage_soc: u16,                   // 0x3AC..0x3AE
    pub _smc_volt_res1: [u8; 0x3B2 - 0x3AE],    // 0x3AE..0x3B2
    pub tvmin_gfx: u16,                         // 0x3B2..0x3B4 (55 C)
    pub tvmin_soc: u16,                         // 0x3B4..0x3B6 (55 C)
    pub vmin_low_gfx: u16,                      // 0x3B6..0x3B8 (750 mV * 4)
    pub vmin_low_soc: u16,                      // 0x3B8..0x3BA (750 mV * 4)
    pub vmin_high_gfx: u16,                     // 0x3BA..0x3BC (750 mV * 4)
    pub vmin_high_soc: u16,                     // 0x3BC..0x3BE (750 mV * 4)
    pub tvmin_hyst_gfx: u16,                    // 0x3BE..0x3C0 (5 C)
    pub tvmin_hyst_soc: u16,                    // 0x3C0..0x3C2 (5 C)
    pub _smc_dpm_res0: [u8; 0x52E - 0x3C2],     // 0x3C2..0x52E
    pub freq_gfx_min: u16,                      // 0x52E..0x530 (FreqTableGfx)
    pub freq_gfx_max: u16,                      // 0x530..0x532 (FreqTableGfx)
    pub _smc_dpm_res1: [u8; 0x54E - 0x532],     // 0x532..0x54E
    pub freq_vclk: [u16; 2],                    // 0x54E..0x552 (FreqTableVclk[0..1])
    pub _smc_dpm_res2: [u8; 0x55E - 0x552],     // 0x552..0x55E
    pub freq_dclk: [u16; 2],                    // 0x55E..0x562 (FreqTableDclk[0..1])
    pub _smc_dpm_res3: [u8; 0x56E - 0x562],     // 0x562..0x56E
    pub freq_soc: [u16; 2],                     // 0x56E..0x572 (FreqTableSocclk[0..1])
    pub _smc_dpm_res4: [u8; 0x57E - 0x572],     // 0x572..0x57E
    pub freq_uclk: [u16; 4],                    // 0x57E..0x586 (FreqTableUclk[0..3])
    pub freq_dcef: [u16; 2],                    // 0x586..0x58A (FreqTableDcefclk[0..1])
    pub _smc_dpm_res5: [u8; 0x596 - 0x58A],     // 0x58A..0x596
    pub freq_disp: [u16; 2],                    // 0x596..0x59A (FreqTableDispclk[0..1])
    pub _smc_dpm_res6: [u8; 0x5A6 - 0x59A],     // 0x59A..0x5A6
    pub freq_pix: [u16; 2],                     // 0x5A6..0x5AA (FreqTablePixclk[0..1])
    pub _smc_dpm_res7: [u8; 0x5B6 - 0x5AA],     // 0x5AA..0x5B6
    pub freq_phy: [u16; 2],                     // 0x5B6..0x5BA (FreqTablePhyclk[0..1])
    pub _smc_dpm_res8: [u8; 0x5C6 - 0x5BA],     // 0x5BA..0x5C6
    pub freq_dtb: [u16; 2],                     // 0x5C6..0x5CA (FreqTableDtbclk[0..1])
    pub _smc_dpm_res9: [u8; 0x5D6 - 0x5CA],     // 0x5CA..0x5D6
    pub freq_fclk: [u16; 2],                    // 0x5D6..0x5DA (FreqTableFclk[0..1])
    pub _smc_dpm_res10: [u8; 0x624 - 0x5DA],    // 0x5DA..0x624
    pub freq_gfx_min_mirror: u16,               // 0x624..0x626
    pub freq_gfx_max_mirror: u16,               // 0x626..0x628
    pub _smc_dpm_res11: u16,                    // 0x628..0x62A
    pub _smc_mem_res0: [u8; 0x65A - 0x62A],     // 0x62A..0x65A
    pub mem_clk_div: [u8; 4],                   // 0x65A..0x65E
    pub _smc_mem_res1: [u8; 0x66A - 0x65E],     // 0x65E..0x66A
    pub mem_vddci: [u16; 4],                    // 0x66A..0x672
    pub mem_mvdd: [u16; 4],                     // 0x672..0x67A
    pub _smc_pcie_res0: [u8; 0x6F6 - 0x67A],    // 0x67A..0x6F6
    pub pcie_gen_speed: [u8; 2],                // 0x6F6..0x6F8
    pub pcie_lane_count: [u8; 2],               // 0x6F8..0x6FA
    pub lclk_freq: [u16; 2],                    // 0x6FA..0x6FE
    pub fan_stop_temp: u16,                     // 0x6FE..0x700
    pub fan_start_temp: u16,                    // 0x700..0x702
    pub _smc_fan_res0: [u8; 0x716 - 0x702],     // 0x702..0x716
    pub fan_pwm_min: u16,                       // 0x716..0x718
    pub fan_acoustic_limit: u16,                // 0x718..0x71A
    pub fan_throttling_rpm: u16,                // 0x71A..0x71C
    pub fan_max_rpm: u16,                       // 0x71C..0x71E
    pub _smc_fan_res1: u16,                     // 0x71E..0x720
    pub fan_target_temp: u16,                   // 0x720..0x722
    pub fan_target_gfx_clock: u16,              // 0x722..0x724
    pub fan_mode: u8,                           // 0x724
    pub fan_hyst_ctrl: u8,                      // 0x725
    pub fan_zero_rpm_enable: u8,                // 0x726
    pub _smc_crv_res0: [u8; 0x74E - 0x727],     // 0x727..0x74E
    pub static_volt_gfx: QuadraticCoeff,        // 0x74E..0x75A
    pub _smc_crv_res1: [u8; 0x766 - 0x75A],     // 0x75A..0x766
    pub static_volt_soc: QuadraticCoeff,        // 0x766..0x772
    pub _smc_crv_res2: [u8; 0x7AA - 0x772],     // 0x772..0x7AA
    pub static_volt_offset_gfx: QuadraticCoeff, // 0x7AA..0x7B6
    pub static_volt_offset_soc: QuadraticCoeff, // 0x7B6..0x7C2
    pub dctol_gfx: i16,                         // 0x7C2..0x7C4
    pub dctol_soc: i16,                         // 0x7C4..0x7C6
    pub dcbtc_enable_gfx: u8,                   // 0x7C6
    pub dcbtc_enable_soc: u8,                   // 0x7C7
    pub _smc_dcbtc_res: [u8; 0x7CA - 0x7C8],    // 0x7C8..0x7CA
    pub dcbtc_min_gfx: i16,                     // 0x7CA..0x7CC
    pub dcbtc_min_soc: i16,                     // 0x7CC..0x7CE
    pub dcbtc_max_gfx: i16,                     // 0x7CE..0x7D0
    pub dcbtc_max_soc: i16,                     // 0x7D0..0x7D2
    pub dcbtc_gb_gfx: i16,                      // 0x7D2..0x7D4
    pub dcbtc_gb_soc: i16,                      // 0x7D4..0x7D6
    pub _smc_tail: [u8; 2470 - 0x7D6],          // 0x7D6..0x9A6
}

const _: () = assert!(size_of::<AtomSmcDpmInfoV4_9>() == 2470);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, pm_quiet_limit) == 0x2DA);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, pm_quiet_temp) == 0x2EA);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, ac_quiet) == 0x2F2);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, smc_pptable_if_version) == 0x322);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, features_to_run) == 0x326);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, socket_power_limit_ac) == 0x32E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_gfx_min) == 0x52E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_vclk) == 0x54E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_dclk) == 0x55E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_soc) == 0x56E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_uclk) == 0x57E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_dcef) == 0x586);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_disp) == 0x596);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_pix) == 0x5A6);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_phy) == 0x5B6);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_dtb) == 0x5C6);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, freq_fclk) == 0x5D6);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, mem_clk_div) == 0x65A);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, pcie_gen_speed) == 0x6F6);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, fan_pwm_min) == 0x716);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, static_volt_gfx) == 0x74E);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, static_volt_offset_gfx) == 0x7AA);
const _: () = assert!(offset_of!(AtomSmcDpmInfoV4_9, dctol_gfx) == 0x7C2);

impl AtomSmcDpmInfoV4_9 {
    pub fn detect_chip(&self, raw: &[u8]) -> NaviRDNA2Chip {
        let frev = raw.get(2).copied().unwrap_or(0);
        match frev {
            0x0F => NaviRDNA2Chip::Navi21SiennaCichlid,
            0x10 | 0x11 => NaviRDNA2Chip::Navi22NavyFlounder,
            0x13 => NaviRDNA2Chip::Navi24BeigeGoby,
            _ => {
                let pwr = self.socket_power_limit_ac[0];
                if pwr >= 200 {
                    NaviRDNA2Chip::Navi21SiennaCichlid
                } else if pwr >= 145 {
                    NaviRDNA2Chip::Navi22NavyFlounder
                } else {
                    NaviRDNA2Chip::Navi23DimgreyCavefish
                }
            }
        }
    }

    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }
}

pub fn field_offset_by_name(name: &str) -> usize {
    match name {
        "od_features" => offset_of!(AtomSmcDpmInfoV4_9, od_feature_flags),
        "od_gfx" => offset_of!(AtomSmcDpmInfoV4_9, od_gfx_max),
        "od_mem" => offset_of!(AtomSmcDpmInfoV4_9, od_mem_max),
        "od_pwr" => offset_of!(AtomSmcDpmInfoV4_9, od_pwr_max),
        "od_fan" => offset_of!(AtomSmcDpmInfoV4_9, od_fan_max),
        "od_acoustic" => offset_of!(AtomSmcDpmInfoV4_9, od_acoustic_max),
        "od_mem_timing" => offset_of!(AtomSmcDpmInfoV4_9, od_mem_timing_ctrl),
        "od_pwr_mode" => offset_of!(AtomSmcDpmInfoV4_9, od_pwr_mode_ctrl),
        "od_zero_rpm" => offset_of!(AtomSmcDpmInfoV4_9, od_zero_rpm_ctrl),
        "pm_quiet" => offset_of!(AtomSmcDpmInfoV4_9, pm_quiet_limit),
        "pm_balanced" => offset_of!(AtomSmcDpmInfoV4_9, pm_balanced_limit),
        "pm_turbo" => offset_of!(AtomSmcDpmInfoV4_9, pm_turbo_limit),
        "pm_rage" => offset_of!(AtomSmcDpmInfoV4_9, pm_rage_limit),
        "ac_quiet" => offset_of!(AtomSmcDpmInfoV4_9, ac_quiet),
        "ac_balanced" => offset_of!(AtomSmcDpmInfoV4_9, ac_balanced),
        "ac_turbo" => offset_of!(AtomSmcDpmInfoV4_9, ac_turbo),
        "ac_rage" => offset_of!(AtomSmcDpmInfoV4_9, ac_rage),
        "smu_features" => offset_of!(AtomSmcDpmInfoV4_9, features_to_run),
        "tdc_gfx" => offset_of!(AtomSmcDpmInfoV4_9, tdc_limit_gfx),
        "tdc_soc" => offset_of!(AtomSmcDpmInfoV4_9, tdc_limit_soc),
        "pwr_limit_w" => offset_of!(AtomSmcDpmInfoV4_9, socket_power_limit_ac),
        "ulv_gfx" => offset_of!(AtomSmcDpmInfoV4_9, ulv_voltage_vmin_gfx),
        "ulv_soc" => offset_of!(AtomSmcDpmInfoV4_9, ulv_voltage_vmin_soc),
        "volt_gfx" => offset_of!(AtomSmcDpmInfoV4_9, min_voltage_gfx),
        "volt_soc" => offset_of!(AtomSmcDpmInfoV4_9, min_voltage_soc),
        "tvmin_gfx" => offset_of!(AtomSmcDpmInfoV4_9, tvmin_gfx),
        "tvmin_soc" => offset_of!(AtomSmcDpmInfoV4_9, tvmin_soc),
        "vmin_lh_gfx" => offset_of!(AtomSmcDpmInfoV4_9, vmin_low_gfx),
        "vmin_lh_soc" => offset_of!(AtomSmcDpmInfoV4_9, vmin_low_soc),
        "freq_gfx" => offset_of!(AtomSmcDpmInfoV4_9, freq_gfx_min),
        "freq_soc" => offset_of!(AtomSmcDpmInfoV4_9, freq_soc),
        "freq_vclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_vclk),
        "freq_dclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_dclk),
        "freq_dcefclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_dcef),
        "freq_dispclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_disp),
        "freq_pixclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_pix),
        "freq_phyclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_phy),
        "freq_dtbclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_dtb),
        "freq_fclk" => offset_of!(AtomSmcDpmInfoV4_9, freq_fclk),
        "mem_dpm_freq" => offset_of!(AtomSmcDpmInfoV4_9, freq_uclk),
        "mem_dpm_volt" => offset_of!(AtomSmcDpmInfoV4_9, mem_vddci),
        "lclk" => offset_of!(AtomSmcDpmInfoV4_9, lclk_freq),
        "fan_pwm_min" => offset_of!(AtomSmcDpmInfoV4_9, fan_pwm_min),
        "fan_acoustic" => offset_of!(AtomSmcDpmInfoV4_9, fan_acoustic_limit),
        "fan_throttling" => offset_of!(AtomSmcDpmInfoV4_9, fan_throttling_rpm),
        "fan_max_rpm" => offset_of!(AtomSmcDpmInfoV4_9, fan_max_rpm),
        "fan_target_temp" => offset_of!(AtomSmcDpmInfoV4_9, fan_target_temp),
        "fan_target_gfx" => offset_of!(AtomSmcDpmInfoV4_9, fan_target_gfx_clock),
        "fan_zero_rpm" => offset_of!(AtomSmcDpmInfoV4_9, fan_zero_rpm_enable),
        "fan_stop_temp" => offset_of!(AtomSmcDpmInfoV4_9, fan_stop_temp),
        "fan_start_temp" => offset_of!(AtomSmcDpmInfoV4_9, fan_start_temp),
        "od_fan_temp" => offset_of!(AtomSmcDpmInfoV4_9, od_fan_curve_max),
        "od_fan_speed" => offset_of!(AtomSmcDpmInfoV4_9, od_fan_curve_max) + 4,
        "curve_gfx" => offset_of!(AtomSmcDpmInfoV4_9, static_volt_gfx),
        "curve_soc" => offset_of!(AtomSmcDpmInfoV4_9, static_volt_soc),
        "curve_off_gfx" => offset_of!(AtomSmcDpmInfoV4_9, static_volt_offset_gfx),
        "curve_off_soc" => offset_of!(AtomSmcDpmInfoV4_9, static_volt_offset_soc),
        "dcbtc_en" => offset_of!(AtomSmcDpmInfoV4_9, dcbtc_enable_gfx),
        "dcbtc_min" => offset_of!(AtomSmcDpmInfoV4_9, dcbtc_min_gfx),
        "dcbtc_max" => offset_of!(AtomSmcDpmInfoV4_9, dcbtc_max_gfx),
        "dcbtc_gb" => offset_of!(AtomSmcDpmInfoV4_9, dcbtc_gb_gfx),
        "dctol" => offset_of!(AtomSmcDpmInfoV4_9, dctol_gfx),
        _ => 0,
    }
}
