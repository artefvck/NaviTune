#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PptCard {
    PowerAndCurrent,
    CoreVoltage,
    UlvVoltage,
    MemoryVoltage,
    ThermalVmin,
    ClockLimits,
    MemoryDpm,
    PcieAndLinkClocks,
    OverDrive,
    PowerModes,
    PowerModeAcoustic,
    VfCurve,
    VidCurveAndOffset,
    FanSettingsAndLimits,
    ZeroRpm,
    FanCurve,
    ThermalLimits,
}

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
    pub fn field_box_spec(label: &str) -> (i32, i32, f64, &'static str) {
        let l = label.to_ascii_lowercase();
        if l.contains("mv") || l.contains("vddc") || l.contains("vddci") || l.contains("mvdd") || l.contains("vmin") || l.contains("voltage") {
            (0, 1600, 1.0, " mV")
        } else if l.contains("mhz") || l.contains("clk") || l.contains("clock") || l.contains("dpm ") {
            (0, 4500, 5.0, " MHz")
        } else if l.contains("rpm") {
            (0, 6000, 25.0, " RPM")
        } else if l.contains("pwm") || l.contains("%") || (l.contains("power limit") && l.contains("+")) {
            (-100, 100, 1.0, " %")
        } else if l.contains("temp") || l.contains("°c") || l.contains("hyst") {
            (0, 125, 1.0, " °C")
        } else if l.contains("(w)") || l.contains("quiet") || l.contains("balanced") || l.contains("turbo") || l.contains("rage") {
            (0, 1000, 1.0, " W")
        } else if l.contains("(a)") || l.contains("tdc") {
            (0, 1000, 1.0, " A")
        } else if l.contains("gen") {
            (0, 5, 1.0, "")
        } else if l.contains("lane") || l.contains("width") {
            (0, 16, 1.0, "x")
        } else {
            (-100000, 100000, 1.0, "")
        }
    }

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

    pub fn has_power_mode_acoustic(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn has_pcie_and_lclk_limits(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn clock_limit_offset_labels(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => (
                "hard_limit_table_offset GFXCLK (MHz)",
                "hard_limit_table_offset SOCCLK (MHz)",
                "N/A (GCN4/5)",
            ),
            Self::Vega20 => ("freq_table_gfx (0x185)", "freq_table_socclk (0x195)", "freq_table_fclk (0x1C5)"),
            Self::Rdna1 => ("freq_table_gfx (0x11F)", "freq_table_socclk (0x12F)", "freq_table_fclk (0x15F)"),
            Self::Rdna2 => ("freq_gfx (0x52E)", "freq_soc (0x56E)", "freq_fclk (0x5D6)"),
            Self::Rdna3 => ("freq_gfx (0x5EC)", "freq_soc (0x60E)", "freq_fclk (0x62C)"),
            Self::Unknown => ("GFX Clock", "Soc Clock", "FCLK"),
        }
    }

    pub fn core_voltage_offset_labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => ("hard_limit_table_offset VDDC (mV)", "N/A (GCN4/5)"),
            Self::Vega20 => ("min/max_voltage_gfx (0x11F)", "min/max_voltage_soc (0x123)"),
            Self::Rdna1 => ("min/max_voltage_gfx (0x0B9)", "min/max_voltage_soc (0x0BD)"),
            Self::Rdna2 => ("min/max_voltage_gfx (0x378)", "min/max_voltage_soc (0x37C)"),
            Self::Rdna3 => ("min/max_voltage_gfx (0x3BC)", "min/max_voltage_soc (0x3C0)"),
            Self::Unknown => ("GFX Voltage (mV)", "Soc Voltage (mV)"),
        }
    }

    pub fn od_offset_labels(self) -> (&'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => (
                "max_od_engine_clock (0x017)",
                "max_od_memory_clock (0x01B)",
                "power_control_limit (0x01F)",
                "N/A (ATOM Fan SubTable)",
                "N/A (ATOM Fan SubTable)",
            ),
            Self::Vega20 | Self::Rdna1 => (
                "od_settings_min / max (0x02F / 0x053)",
                "od_settings_min / max (0x033 / 0x057)",
                "od_settings_min / max (0x037 / 0x05B)",
                "od_settings_min / max (0x03B / 0x05F)",
                "od_settings_min / max (0x03F / 0x063)",
            ),
            Self::Rdna2 => (
                "od_gfx_min / max (0x04F / 0x097)",
                "od_mem_min / max (0x053 / 0x09B)",
                "od_pwr_min / max (0x057 / 0x09F)",
                "od_fan_min / max (0x05B / 0x0A3)",
                "od_acoustic_min / max (0x05F / 0x0A7)",
            ),
            Self::Rdna3 => (
                "od_gfx_min / max (0x051 / 0x099)",
                "od_mem_min / max (0x055 / 0x09D)",
                "od_pwr_min / max (0x059 / 0x0A1)",
                "od_fan_min / max (0x05D / 0x0A5)",
                "od_acoustic_min / max (0x061 / 0x0A9)",
            ),
            Self::Unknown => ("GFX Clock", "Memory Clock", "Power Limit", "Fan Max RPM", "Fan Acoustic RPM"),
        }
    }

    pub fn has_vid_and_voltage_offset(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn safe_tune_bounds(self) -> (i32, i32, i32, i32, i32) {
        match self {
            Self::Polaris => (1550, 2350, 1200, 3500, 75),
            Self::Vega10 => (1850, 1200, 1200, 3600, 75),
            Self::Vega20 => (2000, 1300, 1200, 3600, 78),
            Self::Rdna1 => (2150, 960, 1200, 3600, 80),
            Self::Rdna2 => (2700, 1150, 1175, 3600, 82),
            Self::Rdna3 => (3100, 1350, 1150, 3600, 85),
            Self::Unknown => (2500, 1100, 1150, 3500, 80),
        }
    }

    pub fn has_fan_curve_points(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn has_fan_acoustic_and_throttling(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn fan_offset_labels(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Polaris => (
                "fan_table_offset PWM Min (%)",
                "fan_table_offset Max RPM",
                "fan_table_offset Target Temp (°C)",
                "fan_table_offset Target GFX (MHz)",
            ),
            Self::Vega10 => (
                "fan_table_offset PWM Min (%)",
                "fan_table_offset Max RPM",
                "fan_table_offset Target Temp (°C)",
                "fan_table_offset Stop/Start Temp (°C)",
            ),
            Self::Vega20 | Self::Rdna1 => (
                "fan_pwm_min (SMU 11.0)",
                "fan_max_rpm (SMU 11.0)",
                "fan_target_temp (SMU 11.0)",
                "fan_zero_rpm_enable (SMU 11.0)",
            ),
            Self::Rdna2 => (
                "fan_pwm_min (0x8B8)",
                "fan_max_rpm (0x8BA)",
                "fan_target_temp (0x8BE)",
                "fan_zero_rpm_enable (0x8AC)",
            ),
            Self::Rdna3 => (
                "fan_pwm_min (0x1438)",
                "fan_max_rpm (0x143A)",
                "fan_target_temp (0x143E)",
                "fan_zero_rpm_enable (0x142C)",
            ),
            Self::Unknown => ("Fan PWM Min", "Fan Max RPM", "Fan Target Temp", "Zero RPM"),
        }
    }

    pub fn has_thermal_limits_array(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
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

    pub fn feature_bit_name(self, bit: usize) -> &'static str {
        match self {
            Self::Polaris | Self::Vega10 => match bit {
                0 => "POWER_CONTAINMENT",
                1 => "VDDC_CONTROL",
                2 => "THERMAL_CONTROLLER",
                3 => "ENABLE_CLOCK_GATING",
                6 => "SCLK_DEEP_SLEEP",
                7 => "MCLK_DEEP_SLEEP",
                11 => "FAN_SPEED_IN_RPM",
                12 => "UVD_DPM",
                13 => "VCE_DPM",
                18 => "ODN_AC_SUPPORT",
                21 => "BACO",
                _ => "PLAT_CAP_RSVD",
            },
            Self::Rdna3 => match bit {
                0 => "FW_DATA_READ",
                1 => "DPM_GFXCLK",
                2 => "DPM_GFX_POWER_OPTIMIZER",
                3 => "DPM_UCLK",
                4 => "DPM_FCLK",
                5 => "DPM_SOCCLK",
                6 => "DPM_MP0CLK",
                7 => "DPM_LINK",
                8 => "DPM_DCN",
                9 => "VMEMP_SCALING",
                10 => "VDDIO_MEM_SCALING",
                11 => "DS_GFXCLK",
                12 => "DS_SOCCLK",
                13 => "DS_FCLK",
                14 => "DS_LCLK",
                15 => "DS_DCFCLK",
                16 => "DS_UCLK",
                17 => "GFX_ULV",
                18 => "FW_DSTATE",
                19 => "GFXOFF",
                20 => "BACO",
                21 => "MM_DPM",
                22 => "SOC_MPCLK_DS",
                23 => "BACO_MPCLK_DS",
                24 => "THROTTLERS",
                25 => "SMARTSHIFT",
                26 => "GTHR",
                27 => "ACDC",
                28 => "VR0HOT",
                29 => "FW_CTF",
                30 => "FAN_CONTROL",
                31 => "GFX_EDC",
                32 => "GFX_READ_MARGIN",
                33 => "LED_DISPLAY",
                34 => "GFX_DPM_DRIFT_TUNING",
                35 => "OUT_OF_BAND_MONITOR",
                36 => "TEMP_DEPENDENT_VMIN",
                _ => "SMU13_FEAT_RSVD",
            },
            Self::Vega20 | Self::Rdna1 | Self::Rdna2 => match bit {
                0 => "DPM_PREFETCHER",
                1 => "DPM_GFXCLK",
                2 => "DPM_GFX_PACE",
                3 => "DPM_UCLK",
                4 => "DPM_SOCCLK",
                5 => "DPM_MP0CLK",
                6 => "DPM_LINK",
                7 => "DPM_DCEFCLK",
                8 => "MEM_VDDCI_SCALING",
                9 => "MEM_MVDD_SCALING",
                10 => "DS_GFXCLK",
                11 => "DS_SOCCLK",
                12 => "DS_LCLK",
                13 => "DS_DCEFCLK",
                14 => "DS_UCLK",
                15 => "GFX_ULV",
                16 => "FW_DSTATE",
                17 => "GFXOFF",
                18 => "BACO",
                19 => "VCN_PG",
                20 => "MMHUB_PG",
                21 => "ATHUB_PG",
                22 => "APCC_DFLL",
                23 => "APCC_PLUS",
                24 => "THD",
                25 => "SMARTSHIFT",
                26 => "GTHR",
                27 => "ACDC",
                28 => "VR0HOT",
                29 => "FW_CTF",
                30 => "FAN_CONTROL",
                31 => "THERMAL",
                32 => "GFX_DCS",
                33 => "RM",
                34 => "LED_DISPLAY",
                35 => "GFX_SS",
                36 => "OUT_OF_BAND_MONITOR",
                37 => "TEMP_DEPENDENT_VMIN",
                _ => "SMU11_FEAT_RSVD",
            },
            Self::Unknown => "UNKNOWN_FEAT",
        }
    }

    pub fn decode_feature_bits(self, mask: u64) -> Vec<(usize, &'static str)> {
        let max_bits = if self.is_gcn_subtable() { 32 } else { 64 };
        (0..max_bits)
            .filter(|&bit| (mask & (1u64 << bit)) != 0)
            .map(|bit| (bit, self.feature_bit_name(bit)))
            .collect()
    }

    pub fn supports_card(self, card: PptCard) -> bool {
        match card {
            PptCard::CoreVoltage | PptCard::MemoryVoltage | PptCard::ClockLimits | PptCard::MemoryDpm | PptCard::OverDrive | PptCard::FanSettingsAndLimits | PptCard::ZeroRpm => true,
            PptCard::PowerAndCurrent | PptCard::UlvVoltage | PptCard::PcieAndLinkClocks | PptCard::VfCurve | PptCard::VidCurveAndOffset | PptCard::FanCurve | PptCard::ThermalLimits => !self.is_gcn_subtable(),
            PptCard::ThermalVmin | PptCard::PowerModeAcoustic => matches!(self, Self::Rdna2),
            PptCard::PowerModes => matches!(self, Self::Rdna2 | Self::Rdna3),
        }
    }

    pub fn card_title(self, card: PptCard) -> &'static str {
        match card {
            PptCard::PowerAndCurrent => "Power & Current",
            PptCard::CoreVoltage => if self.is_gcn_subtable() { "Core Voltage — HardLimit VDDC (mV)" } else { "Core Voltage (mV)" },
            PptCard::UlvVoltage => "ULV Voltage (mV)",
            PptCard::MemoryVoltage => if self.is_gcn_subtable() { "Memory Voltage — HardLimit (mV)" } else { "Memory Voltage (mV)" },
            PptCard::ThermalVmin => "Thermal Vmin",
            PptCard::ClockLimits => if self.is_gcn_subtable() { "Clock Limits — HardLimit (MHz)" } else { "Clock Limits" },
            PptCard::MemoryDpm => if self.is_gcn_subtable() { "Memory MCLK — HardLimit (MHz)" } else { "Memory DPM" },
            PptCard::PcieAndLinkClocks => "PCIe & Link Clocks",
            PptCard::OverDrive => if self.is_gcn_subtable() { "OverDrive Ceilings (0x017)" } else { "OverDrive" },
            PptCard::PowerModes => "Power Modes",
            PptCard::PowerModeAcoustic => "Power Mode Acoustic",
            PptCard::VfCurve => "V/F Curve",
            PptCard::VidCurveAndOffset => "Voltage Curve (VID) & Offset",
            PptCard::FanSettingsAndLimits => if self.is_gcn_subtable() { "ATOM Fan Settings & Limits" } else { "Fan Settings & Limits" },
            PptCard::ZeroRpm => "Zero RPM",
            PptCard::FanCurve => "Fan Curve",
            PptCard::ThermalLimits => "Thermal Limits",
        }
    }

