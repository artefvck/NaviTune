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
        } else if crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(buf).is_some() {
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
}
