
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SiliconArch {
    Navi21,
    Navi22,
    Navi23,
    Navi24,
    Unknown,
}

impl SiliconArch {
    pub fn label(self) -> &'static str {
        match self {
            Self::Navi21 => "Navi 21 (Sienna Cichlid)",
            Self::Navi22 => "Navi 22 (Navy Flounder)",
            Self::Navi23 => "Navi 23 (Dimgrey Cavefish)",
            Self::Navi24 => "Navi 24 (Beige Goby)",
            Self::Unknown => "RDNA 2 (Generic)",
        }
    }

    pub fn mask_tag(self) -> &'static str {
        match self {
            Self::Navi21 => "N21",
            Self::Navi22 => "N22",
            Self::Navi23 => "N23",
            Self::Navi24 => "N24",
            Self::Unknown => "RDNA2",
        }
    }
}

pub struct SmuFeatureDef {
    pub bit: usize,
    pub name: &'static str,
    pub arch_mask: u8, // bit0=N21, bit1=N22, bit2=N23, bit3=N24
}

pub const M_ALL: u8 = 0b1111;
pub const M_N21_ONLY: u8 = 0b0001;
pub const M_N21_N22: u8 = 0b0011;
pub const M_N21_N22_N23: u8 = 0b0111;
pub const M_N23_N24: u8 = 0b1100;
pub const M_N24_ONLY: u8 = 0b1000;

/// Official `smu11_driver_if_sienna_cichlid.h` (`smu_v11_0_7`) 64-bit `FeaturesToRun` map
pub const SMU_FEATURE_DEFS: [SmuFeatureDef; 64] = [
    SmuFeatureDef { bit: 0,  name: "DPM_PREFETCHER",      arch_mask: M_ALL },
    SmuFeatureDef { bit: 1,  name: "DPM_GFXCLK",          arch_mask: M_ALL },
    SmuFeatureDef { bit: 2,  name: "DPM_GFX_GPO",         arch_mask: M_ALL },
    SmuFeatureDef { bit: 3,  name: "DPM_UCLK",            arch_mask: M_ALL },
    SmuFeatureDef { bit: 4,  name: "DPM_FCLK",            arch_mask: M_ALL },
    SmuFeatureDef { bit: 5,  name: "DPM_SOCCLK",          arch_mask: M_ALL },
    SmuFeatureDef { bit: 6,  name: "DPM_MP0CLK",          arch_mask: M_ALL },
    SmuFeatureDef { bit: 7,  name: "DPM_LINK",            arch_mask: M_ALL },
    SmuFeatureDef { bit: 8,  name: "DPM_DCEFCLK",         arch_mask: M_ALL },
    SmuFeatureDef { bit: 9,  name: "DPM_XGMI",            arch_mask: M_N21_ONLY },
    SmuFeatureDef { bit: 10, name: "MEM_VDDCI_SCALING",   arch_mask: M_ALL },
    SmuFeatureDef { bit: 11, name: "MEM_MVDD_SCALING",    arch_mask: M_ALL },
    SmuFeatureDef { bit: 12, name: "DS_GFXCLK",           arch_mask: M_ALL },
    SmuFeatureDef { bit: 13, name: "DS_SOCCLK",           arch_mask: M_ALL },
    SmuFeatureDef { bit: 14, name: "DS_FCLK",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 15, name: "DS_LCLK",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 16, name: "DS_DCEFCLK",          arch_mask: M_ALL },
    SmuFeatureDef { bit: 17, name: "DS_UCLK",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 18, name: "GFX_ULV",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 19, name: "FW_DSTATE",           arch_mask: M_ALL },
    SmuFeatureDef { bit: 20, name: "GFX_PER_CU_CG",       arch_mask: M_ALL },
    SmuFeatureDef { bit: 21, name: "MM_DPM_PG",           arch_mask: M_ALL },
    SmuFeatureDef { bit: 22, name: "VCN_PG",              arch_mask: M_ALL },
    SmuFeatureDef { bit: 23, name: "JPEG_PG",             arch_mask: M_N21_N22_N23 },
    SmuFeatureDef { bit: 24, name: "USB_PG",              arch_mask: M_N21_N22 },
    SmuFeatureDef { bit: 25, name: "DF_CSTATES",          arch_mask: M_ALL },
    SmuFeatureDef { bit: 26, name: "SMU_LOW_POWER",       arch_mask: M_N23_N24 },
    SmuFeatureDef { bit: 27, name: "MPT",                 arch_mask: M_ALL },
    SmuFeatureDef { bit: 28, name: "GFX_IMU",             arch_mask: M_N24_ONLY },
    SmuFeatureDef { bit: 29, name: "DPM_VCLK",            arch_mask: M_N21_N22_N23 },
    SmuFeatureDef { bit: 30, name: "DPM_DCLK",            arch_mask: M_N21_N22_N23 },
    SmuFeatureDef { bit: 31, name: "FW_DATA_READ",        arch_mask: M_ALL },
    SmuFeatureDef { bit: 32, name: "PPT",                 arch_mask: M_ALL },
    SmuFeatureDef { bit: 33, name: "TDC",                 arch_mask: M_ALL },
    SmuFeatureDef { bit: 34, name: "THERMAL",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 35, name: "GFX_PER_CU_CG_2",     arch_mask: M_ALL },
    SmuFeatureDef { bit: 36, name: "RM",                  arch_mask: M_ALL },
    SmuFeatureDef { bit: 37, name: "DS_DCEFCLK_2",        arch_mask: M_ALL },
    SmuFeatureDef { bit: 38, name: "ACDC",                arch_mask: M_ALL },
    SmuFeatureDef { bit: 39, name: "VR0HOT",              arch_mask: M_ALL },
    SmuFeatureDef { bit: 40, name: "VR1HOT",              arch_mask: M_ALL },
    SmuFeatureDef { bit: 41, name: "FW_CTF",              arch_mask: M_ALL },
    SmuFeatureDef { bit: 42, name: "LED_DISPLAY",         arch_mask: M_N21_N22 },
    SmuFeatureDef { bit: 43, name: "FAN_CONTROL",         arch_mask: M_ALL },
    SmuFeatureDef { bit: 44, name: "GFX_EDC",             arch_mask: M_ALL },
    SmuFeatureDef { bit: 45, name: "GFXOFF",              arch_mask: M_ALL },
    SmuFeatureDef { bit: 46, name: "CG",                  arch_mask: M_ALL },
    SmuFeatureDef { bit: 47, name: "DPM_GFX_PACE",        arch_mask: M_N21_N22 },
    SmuFeatureDef { bit: 48, name: "SPARE_48",            arch_mask: 0 },
    SmuFeatureDef { bit: 49, name: "SPARE_49",            arch_mask: 0 },
    SmuFeatureDef { bit: 50, name: "SPARE_50",            arch_mask: 0 },
    SmuFeatureDef { bit: 51, name: "SPARE_51",            arch_mask: 0 },
    SmuFeatureDef { bit: 52, name: "SPARE_52",            arch_mask: 0 },
    SmuFeatureDef { bit: 53, name: "SPARE_53",            arch_mask: 0 },
    SmuFeatureDef { bit: 54, name: "SPARE_54",            arch_mask: 0 },
    SmuFeatureDef { bit: 55, name: "SPARE_55",            arch_mask: 0 },
    SmuFeatureDef { bit: 56, name: "SPARE_56",            arch_mask: 0 },
    SmuFeatureDef { bit: 57, name: "SPARE_57",            arch_mask: 0 },
    SmuFeatureDef { bit: 58, name: "SPARE_58",            arch_mask: 0 },
    SmuFeatureDef { bit: 59, name: "SPARE_59",            arch_mask: 0 },
    SmuFeatureDef { bit: 60, name: "SPARE_60",            arch_mask: 0 },
    SmuFeatureDef { bit: 61, name: "SPARE_61",            arch_mask: 0 },
    SmuFeatureDef { bit: 62, name: "SPARE_62",            arch_mask: 0 },
    SmuFeatureDef { bit: 63, name: "SPARE_63",            arch_mask: 0 },
];

pub fn arch_badge(def: &SmuFeatureDef, detected: SiliconArch) -> (&'static str, bool) {
    let supported = match detected {
        SiliconArch::Navi21 => (def.arch_mask & 0b0001) != 0,
        SiliconArch::Navi22 => (def.arch_mask & 0b0010) != 0,
        SiliconArch::Navi23 => (def.arch_mask & 0b0100) != 0,
        SiliconArch::Navi24 => (def.arch_mask & 0b1000) != 0,
        SiliconArch::Unknown => def.arch_mask != 0,
    };
    let tag = match def.arch_mask {
        M_ALL => "N21-24",
        M_N21_ONLY => "N21",
        M_N21_N22 => "N21/22",
        M_N21_N22_N23 => "N21-23",
        M_N23_N24 => "N23/24",
        M_N24_ONLY => "N24",
        _ => "SPARE",
    };
    (tag, supported)
}

pub fn detect_silicon_arch(buf: &[u8]) -> SiliconArch {
    if buf.get(2) == Some(&0x13) || buf.len() == 2690 {
        return SiliconArch::Navi24;
    }
    if buf.len() >= 0x330 {
        let pwr = u16::from_le_bytes([buf[0x32E], buf[0x32F]]);
        if pwr >= 200 {
            return SiliconArch::Navi21;
        } else if pwr >= 145 {
            return SiliconArch::Navi22;
        } else if pwr > 0 {
            return SiliconArch::Navi23;
        }
    }
    SiliconArch::Unknown
}
