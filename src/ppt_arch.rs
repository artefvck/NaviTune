#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PptArch {
    Polaris,
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
        } else if crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(buf).is_some() {
            Self::Rdna1
        } else if crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(buf).is_some() {
            Self::Vega20
        } else if crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(buf).is_some() {
            Self::Vega10
        } else if crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(buf).is_some() {
            Self::Polaris
        } else {
            Self::Unknown
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Polaris => "Polaris",
            Self::Vega10 => "Vega 10",
            Self::Vega20 => "Vega 20",
            Self::Rdna1 => "RDNA 1",
            Self::Rdna2 => "RDNA 2",
            Self::Rdna3 => "RDNA 3",
            Self::Unknown => "Unknown",
        }
    }

    pub fn is_rdna3(self) -> bool {
        matches!(self, Self::Rdna3)
    }

    pub fn is_rdna2(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn is_rdna1(self) -> bool {
        matches!(self, Self::Rdna1)
    }

    pub fn is_vega(self) -> bool {
        matches!(self, Self::Vega10 | Self::Vega20)
    }

    pub fn is_gcn_subtable(self) -> bool {
        matches!(self, Self::Polaris | Self::Vega10)
    }

    pub fn has_hard_limit_subtable(self) -> bool {
        matches!(self, Self::Polaris | Self::Vega10)
    }

    pub fn has_od8_and_smu_masks(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn has_multi_power_modes(self) -> bool {
        matches!(self, Self::Rdna2 | Self::Rdna3)
    }

    pub fn has_vf_curve_points(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn table_struct_name(self) -> &'static str {
        match self {
            Self::Polaris => "AtomPolarisPowerPlayTable",
            Self::Vega10 => "AtomVega10PowerPlayTable",
            Self::Vega20 => "AtomVega20PowerPlayTable",
            Self::Rdna1 => "AtomSmcDpmInfoV4_5",
            Self::Rdna2 => "AtomSmcDpmInfoV4_9",
            Self::Rdna3 => "AtomSmcDpmInfoV4_12",
            Self::Unknown => "Unknown",
        }
    }

    pub fn smu_generation(self) -> &'static str {
        match self {
            Self::Polaris => "SMU 8.0",
            Self::Vega10 => "SMU 9.0",
            Self::Vega20 | Self::Rdna1 | Self::Rdna2 => "SMU 11.0",
            Self::Rdna3 => "SMU 13.0",
            Self::Unknown => "Unknown",
        }
    }

    pub fn new_blank_table(self) -> Vec<u8> {
        let (sz, rev) = match self {
            Self::Polaris => (384u16, 0x07u8),
            Self::Vega10 => (512u16, 0x08u8),
            Self::Vega20 => (1924u16, 0x0Bu8),
            Self::Rdna1 => (1924u16, 0x0Du8),
            Self::Rdna2 => (2470u16, 0x0Fu8),
            Self::Rdna3 => (5424u16, 0x14u8),
            Self::Unknown => return Vec::new(),
        };
        let mut buf = vec![0u8; sz as usize];
        buf[0..2].copy_from_slice(&sz.to_le_bytes());
        buf[2] = rev;
        buf
    }

    pub fn gcn_subtable_hint(self, tab: &str) -> Option<&'static str> {
        match (self, tab) {
            (Self::Polaris, "power") => Some("Polaris (SMU 8.0, rev 0x07): GFX Vmax edits HardLimit VDDC (hard_limit_table_offset); use OC Limits for Power Control (+%)."),
            (Self::Polaris, "freq") => Some("Polaris (SMU 8.0, rev 0x07): GFXCLK Max and Memory DPM 3 edit HardLimit sub-table bounds in-place."),
            (Self::Polaris, "fan") => Some("Polaris (SMU 8.0, rev 0x07): PWM Min, Max RPM, Target Temp, Target GFX Clock, and V3 Zero RPM edit Fan sub-table in-place."),
            (Self::Vega10, "power") => Some("Vega 10 (SMU 9.0, rev 0x08): GFX Vmax edits HardLimit VDDC (hard_limit_table_offset); use OC Limits for Power Control (+%)."),
            (Self::Vega10, "freq") => Some("Vega 10 (SMU 9.0, rev 0x08): GFXCLK Max, SOCCLK Max, and Memory DPM 3 edit HardLimit sub-table bounds in-place."),
            (Self::Vega10, "fan") => Some("Vega 10 (SMU 9.0, rev 0x08): PWM Min, Max RPM, Target Temp, Target GFX Clock, and V3 Zero RPM edit Fan sub-table in-place."),
            _ => None,
        }
    }

    pub fn max_temp_entries(self) -> usize {
        match self {
            Self::Vega20 | Self::Rdna1 => 7,
            Self::Rdna2 | Self::Rdna3 => 9,
            Self::Polaris | Self::Vega10 | Self::Unknown => 0,
        }
    }

    pub fn default_table_size(self) -> usize {
        match self {
            Self::Polaris => 384,
            Self::Vega10 => 512,
            Self::Vega20 | Self::Rdna1 => 1924,
            Self::Rdna2 => 2470,
            Self::Rdna3 => 5424,
            Self::Unknown => 0,
        }
    }
}
