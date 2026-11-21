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
