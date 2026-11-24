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
    FanLimits,
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
        if buf.len() >= 1674 && (buf[2] == 0x0C || buf[2] == 0x0D) {
            return Self::Rdna1;
        }
        if crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(buf).is_some() {
            Self::Rdna3
        } else if crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(buf).is_some() {
            Self::Rdna2
        } else if crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(buf).is_some() {
            Self::Rdna1
        } else if buf.len() >= 1500 && buf.get(2) == Some(&0x0B) {
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
        matches!(self, Self::Rdna2 | Self::Rdna3)
    }

    pub fn clock_limit_offset_labels(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => (
                "hard_limit_table_offset GFXCLK (MHz)",
                "hard_limit_table_offset SOCCLK (MHz)",
                "N/A (GCN4/5)",
            ),
            Self::Vega20 => ("freq_table_gfx (0x165)", "freq_table_socclk (0x1A5)", "freq_table_fclk (0x1F5)"),
            Self::Rdna1 => ("freq_table_gfx (0x32E / 0x0FF)", "freq_table_socclk (0x34E / 0x13F)", "freq_table_fclk (0x396 / 0x187)"),
            Self::Rdna2 => ("freq_gfx (0x52E)", "freq_soc (0x56E)", "freq_fclk (0x5D6)"),
            Self::Rdna3 => ("freq_gfx (0x5EC)", "freq_soc (0x60E)", "freq_fclk (0x62C)"),
            Self::Unknown => ("GFX Clock", "Soc Clock", "FCLK"),
        }
    }

    pub fn power_offset_labels(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Vega20 => (
                "socket_power_limit_ac (0x11D)",
                "tdc_limit_gfx (0x13D)",
                "tdc_limit_soc (0x13F)",
                "vddci_max / mvdd_max (0x145 / 0x147)",
            ),
            Self::Rdna1 => (
                "socket_power_limit_ac (0x1EE / 0x0B7)",
                "tdc_limit_gfx (0x212 / 0x0D7)",
                "tdc_limit_soc (0x20E / 0x0D9)",
                "mem_vddci/mvdd (0x422 / 0x0DF)",
            ),
            Self::Rdna2 => (
                "socket_power_limit_ac (0x32E)",
                "tdc_limit_gfx (0x34E)",
                "tdc_limit_soc (0x350)",
                "mem_vddci / mem_mvdd (0x66A / 0x672)",
            ),
            Self::Rdna3 => (
                "socket_power_limit_ac (0x350)",
                "tdc_limit_gfx (0x366)",
                "tdc_limit_soc (0x368)",
                "mem_vddci / mem_mvdd (0x7D4 / 0x7DC)",
            ),
            _ => (
                "Power Limit (W)",
                "TDC GFX (A)",
                "TDC Soc (A)",
                "Memory Voltage (mV)",
            ),
        }
    }

    pub fn memory_voltage_offset_label(self) -> &'static str {
        match self {
            Self::Rdna1 => "mem_vddci / mem_mvdd (0x422 / 0x42A)",
            Self::Rdna2 => "mem_vddci / mem_mvdd (0x66A / 0x672)",
            Self::Rdna3 => "mem_vddci / mem_mvdd (0x7D4 / 0x7DC)",
            _ => "vddci / mvdd",
        }
    }

    pub fn memory_uclk_offset_label(self, raw_len: usize) -> &'static str {
        match self {
            Self::Vega20 => "freq_table_uclk (0x1B5)",
            Self::Rdna1 => if raw_len < 1924 { "freq_table_uclk (0x37E)" } else { "freq_table_uclk (0x14F)" },
            Self::Rdna2 => "freq_uclk (0x57E)",
            Self::Rdna3 => "freq_uclk (0x63C)",
            _ => "freq_uclk",
        }
    }

    pub fn ulv_offset_labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Vega20 => ("min_voltage_ulv_gfx (0x155)", "min_voltage_ulv_soc (0x157)"),
            Self::Rdna1 => ("min_voltage_ulv_gfx (0x246 / 0x0EF)", "min_voltage_ulv_soc (0x248 / 0x0F1)"),
            Self::Rdna2 => ("ulv_voltage_vmin_gfx / vmax_gfx (0x380 / 0x384)", "ulv_voltage_vmin_soc / vmax_soc (0x382 / 0x386)"),
            Self::Rdna3 => ("ulv_voltage_vmin_gfx (0x3C4)", "ulv_voltage_vmin_soc (0x3C6)"),
            _ => ("ULV GFX (mV)", "ULV Soc (mV)"),
        }
    }

    pub fn core_voltage_offset_labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => ("hard_limit_table_offset VDDC (mV)", "N/A (GCN4/5)"),
            Self::Vega20 => ("min/max_voltage_gfx (0x159)", "min/max_voltage_soc (0x15B)"),
            Self::Rdna1 => ("min/max_voltage_gfx (0x24A / 0x0F3)", "min/max_voltage_soc (0x24C / 0x0F5)"),
            Self::Rdna2 => ("min/max_voltage_gfx (0x3A6)", "min/max_voltage_soc (0x3A8)"),
            Self::Rdna3 => ("min/max_voltage_gfx (0x3B4)", "min/max_voltage_soc (0x3B8)"),
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
            Self::Vega20 => (
                "od_settings_min / max (0x0A9 / 0x055)",
                "od_settings_min / max (0x0AD / 0x059)",
                "od_settings_min / max (0x0B1 / 0x05D)",
                "od_settings_min / max (0x0B5 / 0x061)",
                "od_settings_min / max (0x0B9 / 0x065)",
            ),
            Self::Rdna1 => (
                "od_settings_min / max (0x162 / 0x0E2)",
                "od_settings_min / max (0x16E / 0x0EE)",
                "od_settings_min / max (0x186 / 0x106)",
                "od_settings_min / max (0x18A / 0x10A)",
                "od_settings_min / max (0x18E / 0x10E)",
            ),
            Self::Rdna2 => (
                "od_gfx_min / max (0x1E2 / 0x0E2)",
                "od_mem_min / max (0x1FA / 0x0FA)",
                "od_pwr_min / max (0x202 / 0x102)",
                "od_fan_min / max (0x206 / 0x106)",
                "od_acoustic_min / max (0x20A / 0x10A)",
            ),
            Self::Rdna3 => (
                "od_gfx_min / max (0x1FF / 0x0FF)",
                "od_mem_min / max (0x207 / 0x107)",
                "od_pwr_min / max (0x20F / 0x10F)",
                "od_fan_min / max (0x213 / 0x113)",
                "od_acoustic_min / max (0x217 / 0x117)",
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

    #[allow(clippy::type_complexity)]
    pub fn fan_offset_labels(self) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Polaris | Self::Vega10 => (
                "fan_table_offset PWM Min (%)",
                "N/A (GCN4/5)",
                "N/A (GCN4/5)",
                "fan_table_offset Max RPM",
                "fan_table_offset Target Temp (°C)",
                "fan_table_offset Target GFX (MHz)",
                "fan_table_offset Stop Temp (°C)",
                "fan_table_offset Start Temp (°C)",
            ),
            Self::Vega20 => (
                "fan_pwm_min (0x675)",
                "fan_acoustic_limit_rpm (0x677)",
                "fan_throttling_rpm (0x679)",
                "fan_maximum_rpm (0x67B)",
                "fan_target_temperature (0x67D)",
                "fan_target_gfxclk (0x67F)",
                "fan_stop_temp (0x663)",
                "fan_start_temp (0x665)",
            ),
            Self::Rdna1 => (
                "fan_pwm_min (0x476 / 0x68B)",
                "fan_acoustic_rpm (0x478 / 0x68D)",
                "fan_throttling_rpm (0x47A / 0x68F)",
                "fan_max_rpm (0x47C / 0x691)",
                "fan_target_temp (0x47E / 0x693)",
                "fan_target_gfx (0x480 / 0x695)",
                "fan_stop_temp (0x45E / 0x679)",
                "fan_start_temp (0x460 / 0x67B)",
            ),
            Self::Rdna2 => (
                "fan_pwm_min (0x716)",
                "fan_acoustic_limit (0x718)",
                "fan_throttling_rpm (0x71A)",
                "fan_max_rpm (0x71C)",
                "fan_target_temp (0x720)",
                "fan_target_gfx_clock (0x722)",
                "fan_stop_temp (0x6FE)",
                "fan_start_temp (0x700)",
            ),
            Self::Rdna3 => (
                "fan_pwm_min (0x86C)",
                "fan_acoustic_limit (0x86E)",
                "fan_throttling_rpm (0x86A)",
                "fan_max_rpm (0x870)",
                "fan_target_temp (0x878)",
                "fan_target_gfx_clock (0x876)",
                "fan_stop_temp (0x81E)",
                "fan_start_temp (0x838)",
            ),
            Self::Unknown => (
                "Fan PWM Min",
                "Fan Acoustic Limit",
                "Fan Throttling RPM",
                "Fan Max RPM",
                "Fan Target Temp",
                "Fan Target GFX Clock",
                "Fan Stop Temp",
                "Fan Start Temp",
            ),
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

    #[allow(dead_code)]
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

    pub fn effective_temp_entries(self, buf: &[u8]) -> usize {
        if crate::pptable_rdna1::Rdna1ParsedState::parse(buf).is_some() {
            9
        } else {
            self.max_temp_entries()
        }
    }

    pub fn temp_limit_base_offset(self, buf: &[u8]) -> Option<usize> {
        if let Some(r1) = crate::pptable_rdna1::Rdna1ParsedState::parse(buf) {
            return Some(r1.outer_sz + 0x034);
        }
        match self {
            Self::Rdna1 => Some(0x66B),
            Self::Vega20 => Some(if buf.len() >= 1924 { 0x655 } else { 0x1FA }),
            Self::Rdna2 => Some(0x356),
            Self::Rdna3 => Some(0x372),
            _ => None,
        }
    }

    pub fn pcie_and_lclk_offset_labels(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Rdna2 => ("pcie_gen_speed (0x6F6)", "pcie_lane_count (0x6F8)", "lclk_freq (0x6FA)"),
            Self::Rdna3 => ("pcie_gen_speed (0x810)", "pcie_lane_count (0x813)", "lclk_freq (0x816)"),
            _ => ("pcie_gen_speed", "pcie_lane_count", "lclk_freq"),
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
                4 => "DYNAMIC_PCIE_GEN2",
                5 => "DYNAMIC_PCIE_GEN3",
                6 => "SCLK_DEEP_SLEEP",
                7 => "MCLK_DEEP_SLEEP",
                8 => "DYNAMIC_MCLK_SWITCH",
                9 => "DYNAMIC_UVD_STATE",
                10 => "DYNAMIC_VCE_STATE",
                11 => "FAN_SPEED_IN_RPM",
                12 => "UVD_DPM",
                13 => "VCE_DPM",
                14 => "DYNAMIC_ACP_STATE",
                15 => "ENABLE_SMU_7_CONTAINMENT",
                16 => "ENABLE_DYNAMIC_ACT",
                17 => "DISABLE_VOLTAGE_ISLAND",
                18 => "ODN_AC_SUPPORT",
                19 => "ODN_DC_SUPPORT",
                20 => "ENABLE_HW_DC",
                21 => "BACO",
                22 => "ENABLE_ULPS",
                23 => "ENABLE_THROTTLER",
                24 => "FALCON_QUICK_TRANSITION",
                25 => "SMU_CLOCK_STRETCHING",
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

    pub fn supports_tab(&self, tab_idx: usize) -> bool {
        // Tab indices: 0=Features, 1=OC Limits, 2=Power, 3=Frequency, 4=Curve, 5=Fan, 6=Safe Tune, 7=Log, 8=Advance
        !(tab_idx == 4 && self.is_gcn_subtable())
    }

    pub fn has_per_dpm_mem_voltage(self) -> bool {
        matches!(self, Self::Rdna1 | Self::Rdna2 | Self::Rdna3)
    }

    pub fn uses_single_rail_mem_voltage(self, buf: &[u8]) -> bool {
        !self.has_per_dpm_mem_voltage() || (self == Self::Rdna1 && buf.len() >= 1924)
    }

    pub fn memory_voltage_offset_label_for_buf(self, buf: &[u8]) -> &'static str {
        if self == Self::Rdna1 && buf.len() >= 1924 {
            "vddci_max / mvdd_max (0x0DF / 0x0E1)"
        } else {
            self.memory_voltage_offset_label()
        }
    }

    pub fn has_ulv_vmax(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn has_od_ctrl_toggles(self) -> bool {
        matches!(self, Self::Rdna1 | Self::Rdna2)
    }

    pub fn od_ctrl_offset_labels(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Rdna1 => ("od_mem_timing_ctrl (0x11A)", "od_pwr_mode_ctrl (0x11E)", "od_zero_rpm_ctrl (0x122)"),
            Self::Rdna2 => ("od_mem_timing_ctrl (0x116)", "od_pwr_mode_ctrl (0x11A)", "od_zero_rpm_ctrl (0x126)"),
            _ => ("od_mem_timing_ctrl", "od_pwr_mode_ctrl", "od_zero_rpm_ctrl"),
        }
    }

    pub fn has_dynamic_power_modes_only(self) -> bool {
        matches!(self, Self::Rdna3)
    }

    pub fn has_compact_power_modes(self, buf: &[u8]) -> bool {
        matches!(self, Self::Rdna1 | Self::Vega20) && buf.len() < 1924
    }

    pub fn has_static_vid_coeffs(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn has_fan_mode_hyst(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn has_od_fan_temp_speed(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn has_5point_fan_curve_slots(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn fan_curve_note(self) -> &'static str {
        match self {
            Self::Rdna3 => "RDNA 3 (SMU 13.0) computes dynamic fan PWM from Target Temp & Acoustic Limit; 5-point curve slots are RDNA 2 only.",
            _ => "SMU 11.0 (Vega 20 / RDNA 1) drives fan PWM from PWM Min, Target Temp, and Acoustic RPM; 5-point curve slots are RDNA 2 only.",
        }
    }

    pub fn zero_rpm_enable_offset_label(self) -> &'static str {
        match self {
            Self::Rdna2 => "fan_zero_rpm_enable (0x726)",
            Self::Rdna3 => "fan_zero_rpm_enable (0x87C)",
            Self::Rdna1 => "fan_zero_rpm_enable (0x485 / 0x697)",
            Self::Vega20 => "fan_zero_rpm_enable (0x681)",
            _ => "fan_zero_rpm_enable",
        }
    }

    pub fn has_uclk_dividers(self) -> bool {
        matches!(self, Self::Vega20 | Self::Rdna2)
    }

    pub fn has_extended_media_clocks(self) -> bool {
        matches!(self, Self::Rdna2)
    }

    pub fn supports_card_in_buf(self, card: PptCard, buf: &[u8]) -> bool {
        match card {
            PptCard::FanLimits => match self {
                Self::Rdna1 | Self::Rdna2 | Self::Rdna3 => true,
                Self::Vega20 => buf.len() >= 1924,
                _ => false,
            },
            PptCard::UlvVoltage => {
                self.supports_card(card) && !(self == Self::Vega20 && buf.len() < 1900)
            }
            _ => self.supports_card(card),
        }
    }

    pub fn supports_card(self, card: PptCard) -> bool {
        match card {
            PptCard::CoreVoltage | PptCard::MemoryVoltage | PptCard::ClockLimits | PptCard::MemoryDpm | PptCard::OverDrive | PptCard::FanSettingsAndLimits => true,
            PptCard::FanLimits => matches!(self, Self::Vega20 | Self::Rdna1 | Self::Rdna2 | Self::Rdna3),
            PptCard::ZeroRpm => !matches!(self, Self::Polaris),
            PptCard::PowerAndCurrent => !matches!(self, Self::Vega10),
            PptCard::UlvVoltage => !self.is_gcn_subtable(),
            PptCard::PcieAndLinkClocks => self.has_pcie_and_lclk_limits(),
            PptCard::VfCurve => self.has_vf_curve_points(),
            PptCard::VidCurveAndOffset => self.has_vid_and_voltage_offset(),
            PptCard::FanCurve => self.has_fan_curve_points() && self.has_fan_acoustic_and_throttling(),
            PptCard::ThermalLimits => self.has_thermal_limits_array(),
            PptCard::ThermalVmin => matches!(self, Self::Rdna2),
            PptCard::PowerModeAcoustic => self.has_power_mode_acoustic(),
            PptCard::PowerModes => self.has_multi_power_modes(),
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
            PptCard::FanLimits => "Fan Limits",
            PptCard::ZeroRpm => "Zero RPM",
            PptCard::FanCurve => "Fan Curve",
            PptCard::ThermalLimits => "Thermal Limits",
        }
    }


}

#[cfg(test)]
mod arch_matrix_tests {
    use super::{PptArch, PptCard};

    #[test]
    fn test_six_arch_tab_and_card_matrix() {
        // Polaris & Vega 10 hide Curve tab and RDNA/SMU11-only cards
        for gcn in [PptArch::Polaris, PptArch::Vega10] {
            assert!(gcn.supports_tab(0));
            assert!(!gcn.supports_tab(4));
            assert!(gcn.supports_card(PptCard::OverDrive));
            assert!(!gcn.supports_card(PptCard::PowerModes));
            assert!(!gcn.supports_card(PptCard::PowerModeAcoustic));
            assert!(!gcn.supports_card(PptCard::PcieAndLinkClocks));
            assert!(!gcn.supports_card(PptCard::VfCurve));
            assert!(!gcn.supports_card(PptCard::ThermalVmin));
        }
        assert!(!PptArch::Polaris.supports_card(PptCard::ZeroRpm));
        assert!(PptArch::Vega10.supports_card(PptCard::ZeroRpm));

        // Vega 20, RDNA 1, RDNA 2, RDNA 3 show Curve tab and SMU 11/13 cards
        for modern in [PptArch::Vega20, PptArch::Rdna1, PptArch::Rdna2, PptArch::Rdna3] {
            assert!(modern.supports_tab(4));
            assert!(modern.supports_card(PptCard::VfCurve));
            assert!(modern.supports_card(PptCard::ZeroRpm));
        }
        assert!(!PptArch::Vega20.supports_card(PptCard::PcieAndLinkClocks));
        assert!(!PptArch::Rdna1.supports_card(PptCard::PcieAndLinkClocks));
        assert!(PptArch::Rdna2.supports_card(PptCard::PcieAndLinkClocks));
        assert!(PptArch::Rdna3.supports_card(PptCard::PcieAndLinkClocks));

        // RDNA 2 exclusive cards vs RDNA 3
        assert!(PptArch::Rdna2.supports_card(PptCard::PowerModes));
        assert!(PptArch::Rdna2.supports_card(PptCard::PowerModeAcoustic));
        assert!(PptArch::Rdna2.supports_card(PptCard::ThermalVmin));

        assert!(PptArch::Rdna3.supports_card(PptCard::PowerModes));
        assert!(!PptArch::Rdna3.supports_card(PptCard::PowerModeAcoustic));
        assert!(!PptArch::Rdna3.supports_card(PptCard::ThermalVmin));
    }
}

#[cfg(test)]
mod detect_six_arch_tests {
    fn header_bytes(table_rev: u8, format_rev: u8, content_rev: u8, size: usize) -> Vec<u8> {
        let mut b = vec![0u8; size];
        let sz = (size as u16).to_le_bytes();
        b[0] = sz[0];
        b[1] = sz[1];
        b[2] = format_rev;
        b[3] = content_rev;
        b[4] = table_rev;
        b
    }

    #[test]
    fn detect_handles_truncated_buffers_without_panic() {
        for len in 0..=8 {
            let buf = vec![0u8; len];
            let _ = super::PptArch::detect(&buf);
        }
    }

    #[test]
    fn detect_recognizes_all_six_amd_gpu_revisions() {
        let cases = [
            (0x07u8, 0x07u8, 0x01u8, 384usize),
            (0x08u8, 0x08u8, 0x01u8, 512usize),
            (0x0Bu8, 0x0Bu8, 0x01u8, 1924usize),
            (0x0Du8, 0x0Du8, 0x01u8, 1924usize),
            (0x0Fu8, 0x0Fu8, 0x01u8, 2470usize),
            (0x14u8, 0x14u8, 0x01u8, 5424usize),
        ];
        for (trev, frev, crev, size) in cases {
            let buf = header_bytes(trev, frev, crev, size);
            let arch = super::PptArch::detect(&buf);
            assert!(
                !format!("{:?}", arch).contains("Unknown"),
                "Expected known architecture for rev 0x{:02X} ({} B), got {:?}",
                trev,
                size,
                arch
            );
        }
    }
}
