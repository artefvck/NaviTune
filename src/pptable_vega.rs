use core::mem::{offset_of, size_of};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AtomCommonTableHeaderVega {
    pub structure_size: u16,
    pub table_format_revision: u8,
    pub table_content_revision: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VegaChip {
    Vega10,
    Vega20,
    UnknownVega,
}

impl VegaChip {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Vega10 => "Vega 10 (RX Vega 56 / 64 / Frontier / Pro WX 9100)",
            Self::Vega20 => "Vega 20 (Radeon VII / Pro Vega II)",
            Self::UnknownVega => "Vega (GCN 5)",
        }
    }

    pub fn from_did_and_rev(did: u16, frev: u8) -> Self {
        match did {
            0x6860 | 0x6861 | 0x6862 | 0x6863 | 0x6864 | 0x6867 | 0x6868 | 0x686C | 0x687F => Self::Vega10,
            0x66A0 | 0x66A1 | 0x66A2 | 0x66A3 | 0x66A7 | 0x66AF => Self::Vega20,
            _ if frev == 0x0B => Self::Vega20,
            _ if frev == 0x08 => Self::Vega10,
            _ => Self::UnknownVega,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomVega10PowerPlayTable {
    pub header: AtomCommonTableHeaderVega,
    pub table_revision: u8,
    pub table_size: u16,
    pub golden_pp_id: u32,
    pub golden_revision: u32,
    pub format_id: u16,
    pub voltage_time_in_us: u16,
    pub platform_caps: u32,
    pub max_od_engine_clock: u32,
    pub max_od_memory_clock: u32,
    pub power_control_limit: u16,
    pub ulv_voltage_offset: u16,
    pub ulv_smnclk_did: u16,
    pub ulv_mp1clk_did: u16,
    pub ulv_gfxclk_bypass: u16,
    pub gfxclk_slew_rate_quiescent: u16,
    pub gfxclk_slew_rate_dep_vddgfx: u16,
    pub gfxclk_slew_rate_indep_vddgfx: u16,
    pub socclk_slew_rate_quiescent: u16,
    pub socclk_slew_rate_dep_vddgfx: u16,
    pub socclk_slew_rate_indep_vddgfx: u16,
    pub dcefclk_slew_rate_quiescent: u16,
    pub dcefclk_slew_rate_dep_vddgfx: u16,
    pub dcefclk_slew_rate_indep_vddgfx: u16,
    pub state_array_offset: u16,
    pub fan_table_offset: u16,
    pub thermal_controller_offset: u16,
    pub socclk_dep_table_offset: u16,
    pub gfxclk_dep_table_offset: u16,
    pub dcefclk_dep_table_offset: u16,
    pub vddc_lookup_table_offset: u16,
    pub vddmem_lookup_table_offset: u16,
    pub mm_dep_table_offset: u16,
    pub vce_state_table_offset: u16,
    pub reserved_offsets: [u16; 2],
    pub hard_limit_table_offset: u16,
    pub vddci_lookup_table_offset: u16,
    pub pcie_table_offset: u16,
    pub pixclk_dep_table_offset: u16,
    pub dispclk_dep_table_offset: u16,
    pub phyclk_dep_table_offset: u16,
    pub mclk_dep_table_offset: u16,
    pub payload: [u8; 415],
}

const _: () = {
    assert!(size_of::<AtomVega10PowerPlayTable>() == 512);
    assert!(offset_of!(AtomVega10PowerPlayTable, max_od_engine_clock) == 0x017);
    assert!(offset_of!(AtomVega10PowerPlayTable, max_od_memory_clock) == 0x01B);
    assert!(offset_of!(AtomVega10PowerPlayTable, power_control_limit) == 0x01F);
};

impl AtomVega10PowerPlayTable {
    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x08) {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x08) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomVega20PowerPlayTable {
    pub header: AtomCommonTableHeaderVega,
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
    pub power_saving_clock_max: [u32; 5],
    pub power_saving_clock_min: [u32; 5],
    pub od_feature_capabilities: [u32; 2],
    pub od_settings_max: [u32; 21],
    pub od_settings_min: [u32; 21],
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
    pub dpm_descriptor: [u8; 16],
    pub avfs_mid_payload: [u8; 1080],
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
    pub board_tail_payload: [u8; 257],
}

const _: () = {
    assert!(size_of::<AtomVega20PowerPlayTable>() == 1924);
    assert!(offset_of!(AtomVega20PowerPlayTable, od_settings_max) == 0x055);
    assert!(offset_of!(AtomVega20PowerPlayTable, od_settings_min) == 0x0A9);
    assert!(offset_of!(AtomVega20PowerPlayTable, smc_version) == 0x111);
    assert!(offset_of!(AtomVega20PowerPlayTable, socket_power_limit_ac) == 0x11D);
    assert!(offset_of!(AtomVega20PowerPlayTable, tdc_limit_gfx) == 0x13D);
    assert!(offset_of!(AtomVega20PowerPlayTable, min_voltage_gfx) == 0x159);
    assert!(offset_of!(AtomVega20PowerPlayTable, freq_table_gfx) == 0x165);
    assert!(offset_of!(AtomVega20PowerPlayTable, freq_table_uclk) == 0x1B5);
    assert!(offset_of!(AtomVega20PowerPlayTable, temp_limit) == 0x655);
    assert!(offset_of!(AtomVega20PowerPlayTable, fan_pwm_min) == 0x675);
};

impl AtomVega20PowerPlayTable {
    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x0B) {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x0B) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }
}
