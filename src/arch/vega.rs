pub(crate) fn read_gcn_hard_limit_full(buf: &[u8], off: usize, has_soc: bool) -> Option<(i32, i32, i32, i32, i32, i32)> {
    let need = if has_soc { 0x1A } else { 0x16 };
    if off == 0 || off + need > buf.len() || buf[off + 1] == 0 { return None; }
    let base = if has_soc { off + 0x06 } else { off + 0x02 };
    let soc = if has_soc { (u32::from_le_bytes([buf[off + 0x02], buf[off + 0x03], buf[off + 0x04], buf[off + 0x05]]) / 100) as i32 } else { 0 };
    let gfx = (u32::from_le_bytes([buf[base], buf[base + 1], buf[base + 2], buf[base + 3]]) / 100) as i32;
    let mem = (u32::from_le_bytes([buf[base + 4], buf[base + 5], buf[base + 6], buf[base + 7]]) / 100) as i32;
    let vddc = u32::from_le_bytes([buf[base + 8], buf[base + 9], buf[base + 10], buf[base + 11]]) as i32;
    let vddci = u32::from_le_bytes([buf[base + 12], buf[base + 13], buf[base + 14], buf[base + 15]]) as i32;
    let mvdd = u32::from_le_bytes([buf[base + 16], buf[base + 17], buf[base + 18], buf[base + 19]]) as i32;
    Some((soc, gfx, mem, vddc, vddci, mvdd))
}

pub(crate) fn write_gcn_hard_limit_full(buf: &mut [u8], off: usize, has_soc: bool, soc: i32, gfx: i32, mem: i32, vddc: i32, vddci: i32, mvdd: i32) {
    let need = if has_soc { 0x1A } else { 0x16 };
    if off == 0 || off + need > buf.len() || buf[off + 1] == 0 { return; }
    let base = if has_soc { off + 0x06 } else { off + 0x02 };
    if has_soc { buf[off + 0x02..off + 0x06].copy_from_slice(&((soc.max(0) as u32) * 100).to_le_bytes()); }
    buf[base..base + 4].copy_from_slice(&((gfx.max(0) as u32) * 100).to_le_bytes());
    buf[base + 4..base + 8].copy_from_slice(&((mem.max(0) as u32) * 100).to_le_bytes());
    buf[base + 8..base + 12].copy_from_slice(&(vddc.max(0) as u32).to_le_bytes());
    buf[base + 12..base + 16].copy_from_slice(&(vddci.max(0) as u32).to_le_bytes());
    buf[base + 16..base + 20].copy_from_slice(&(mvdd.max(0) as u32).to_le_bytes());
}

pub(crate) fn read_gcn_hard_limit_entry(buf: &[u8], off: usize) -> Option<(i32, i32, i32, i32)> {
    if off == 0 || off + 0x12 > buf.len() || buf[off + 1] == 0 { return None; }
    let soc = (u32::from_le_bytes([buf[off + 0x02], buf[off + 0x03], buf[off + 0x04], buf[off + 0x05]]) / 100) as i32;
    let gfx = (u32::from_le_bytes([buf[off + 0x06], buf[off + 0x07], buf[off + 0x08], buf[off + 0x09]]) / 100) as i32;
    let mem = (u32::from_le_bytes([buf[off + 0x0A], buf[off + 0x0B], buf[off + 0x0C], buf[off + 0x0D]]) / 100) as i32;
    let vddc = u32::from_le_bytes([buf[off + 0x0E], buf[off + 0x0F], buf[off + 0x10], buf[off + 0x11]]) as i32;
    Some((soc, gfx, mem, vddc))
}

pub(crate) fn write_gcn_hard_limit_entry(buf: &mut [u8], off: usize, soc: Option<i32>, gfx: i32, mem: i32, vddc: i32) {
    if off == 0 || off + 0x12 > buf.len() || buf[off + 1] == 0 { return; }
    if let Some(s) = soc { buf[off + 0x02..off + 0x06].copy_from_slice(&((s.max(0) as u32) * 100).to_le_bytes()); }
    buf[off + 0x06..off + 0x0A].copy_from_slice(&((gfx.max(0) as u32) * 100).to_le_bytes());
    buf[off + 0x0A..off + 0x0E].copy_from_slice(&((mem.max(0) as u32) * 100).to_le_bytes());
    buf[off + 0x0E..off + 0x12].copy_from_slice(&(vddc.max(0) as u32).to_le_bytes());
}

pub(crate) fn read_gcn_fan_subtable(buf: &[u8], off: usize) -> Option<(i32, i32, i32, i32, bool, i32, i32)> {
    if off == 0 || off + 0x1C > buf.len() { return None; }
    let rev = buf[off];
    let pwm_min = (u16::from_le_bytes([buf[off + 0x09], buf[off + 0x0A]]) / 100) as i32;
    let tgt_temp = (u16::from_le_bytes([buf[off + 0x0F], buf[off + 0x10]]) / 100) as i32;
    let max_rpm = u16::from_le_bytes([buf[off + 0x16], buf[off + 0x17]]) as i32;
    let tgt_gfx = (u32::from_le_bytes([buf[off + 0x18], buf[off + 0x19], buf[off + 0x1A], buf[off + 0x1B]]) / 100) as i32;
    let (zrpm, stop_t, start_t) = if rev >= 11 && off + 0x2F <= buf.len() {
        (buf[off + 0x2C] != 0, buf[off + 0x2D] as i32, buf[off + 0x2E] as i32)
    } else { (false, 0, 0) };
    Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t))
}

pub(crate) fn write_gcn_fan_subtable(buf: &mut [u8], off: usize, pwm_min: i32, max_rpm: i32, tgt_temp: i32, tgt_gfx: i32, zrpm: bool, stop_t: i32, start_t: i32) {
    if off == 0 || off + 0x1C > buf.len() { return; }
    let rev = buf[off];
    buf[off + 0x09..off + 0x0B].copy_from_slice(&((pwm_min.max(0) as u16) * 100).to_le_bytes());
    buf[off + 0x0F..off + 0x11].copy_from_slice(&((tgt_temp.max(0) as u16) * 100).to_le_bytes());
    buf[off + 0x16..off + 0x18].copy_from_slice(&(max_rpm.max(0) as u16).to_le_bytes());
    buf[off + 0x18..off + 0x1C].copy_from_slice(&((tgt_gfx.max(0) as u32) * 100).to_le_bytes());
    if rev >= 11 && off + 0x2F <= buf.len() {
        buf[off + 0x2C] = u8::from(zrpm);
        buf[off + 0x2D] = stop_t.clamp(0, 255) as u8;
        buf[off + 0x2E] = start_t.clamp(0, 255) as u8;
    }
}

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

    pub fn read_fan_subtable(&self, buf: &[u8]) -> Option<(i32, i32, i32, i32, bool, i32, i32)> {
        read_gcn_fan_subtable(buf, { self.fan_table_offset } as usize)
    }

    pub fn write_fan_subtable(buf: &mut [u8], pwm_min: i32, max_rpm: i32, tgt_temp: i32, tgt_gfx: i32, zrpm: bool, stop_t: i32, start_t: i32) {
        if let Some(t) = Self::from_bytes(buf) {
            write_gcn_fan_subtable(buf, { t.fan_table_offset } as usize, pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t);
        }
    }

    pub fn read_hard_limits(&self, buf: &[u8]) -> Option<(i32, i32, i32, i32)> {
        let (soc, gfx, mem, vddc, _, _) = read_gcn_hard_limit_full(buf, { self.hard_limit_table_offset } as usize, true)?;
        Some((soc, gfx, mem, vddc))
    }

    pub fn read_hard_limit_mem_volt(&self, buf: &[u8]) -> Option<(i32, i32)> {
        let (_, _, _, _, vddci, mvdd) = read_gcn_hard_limit_full(buf, { self.hard_limit_table_offset } as usize, true)?;
        Some((vddci, mvdd))
    }

    pub fn write_hard_limits_full(buf: &mut [u8], soc: i32, gfx: i32, mem: i32, vddc: i32, vddci: i32, mvdd: i32) {
        if let Some(t) = Self::from_bytes(buf) {
            write_gcn_hard_limit_full(buf, { t.hard_limit_table_offset } as usize, true, soc, gfx, mem, vddc, vddci, mvdd);
        }
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

    pub fn from_bytes_padded(buf: &[u8]) -> Option<Self> {
        if buf.len() < 1500 || buf.get(2) != Some(&0x0B) {
            return None;
        }
        let mut raw = [0u8; size_of::<Self>()];
        let n = buf.len().min(size_of::<Self>());
        raw[..n].copy_from_slice(&buf[..n]);
        Some(unsafe { *(raw.as_ptr() as *const Self) })
    }

    pub fn write_back_to_slice(&self, dst: &mut [u8]) {
        let src = unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) };
        let n = dst.len().min(size_of::<Self>());
        dst[..n].copy_from_slice(&src[..n]);
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x0B) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }
}

pub(crate) struct VegaParsedState {
    pub power_limit_w: i32,
    pub tdc_gfx: i32,
    pub tdc_soc: i32,
    pub oc_gfx_clock: [i32; 2],
    pub oc_mem_clock: [i32; 2],
    pub oc_power_limit: [i32; 2],
    pub volt_gfx: [i32; 2],
    pub volt_soc: [i32; 2],
    pub freq_gfx: [i32; 2],
    pub freq_soc: [i32; 2],
    pub freq_fclk: [i32; 2],
    pub mem_dpm_freq: [[i32; 2]; 4],
    pub mem_dpm_volt: [[i32; 2]; 4],
    pub fan_pwm_min: i32,
    pub fan_max_rpm: i32,
    pub fan_acoustic_limit: i32,
    pub fan_throttling_rpm: i32,
    pub fan_target_temp: i32,
    pub fan_target_gfx_clock: i32,
    pub fan_zero_rpm_enable: bool,
    pub fan_stop_temp: i32,
    pub fan_start_temp: i32,
    pub temp_limit_0: i32,
    pub caps: [bool; 7],
}

impl VegaParsedState {
    pub fn parse(b: &[u8], frev: u8) -> Option<Self> {
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as i32;
        let u32_at = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as i32;
        if frev == 0x0B && b.len() == 1534 && b.len() >= 0x53C {
            let fg = [u16_at(0x33A), u16_at(0x348)];
            let (mut mf, mut mv) = ([[0, 1]; 4], [[0, 0]; 4]);
            for i in 0..4 {
                mf[i] = [u16_at(0x3C4 + i * 2), 1];
                mv[i] = [u32_at(0x03B), u32_at(0x043)];
            }
            return Some(Self {
                power_limit_w: u16_at(0x016), tdc_gfx: u16_at(0x018), tdc_soc: u16_at(0x01A),
                oc_gfx_clock: [u32_at(0x04F), u32_at(0x027)], oc_mem_clock: [0, u32_at(0x02B)],
                oc_power_limit: [-50, 20], volt_gfx: [u32_at(0x03B), u32_at(0x043)],
                volt_soc: [u32_at(0x03B), u32_at(0x02F)], freq_gfx: fg,
                freq_soc: [u16_at(0x38A), u16_at(0x390) + 500], freq_fclk: [u16_at(0x38A), u16_at(0x390) + 500],
                mem_dpm_freq: mf, mem_dpm_volt: mv, fan_pwm_min: 20, fan_max_rpm: u16_at(0x53A),
                fan_acoustic_limit: u16_at(0x536), fan_throttling_rpm: u16_at(0x538), fan_target_temp: 75,
                fan_target_gfx_clock: fg[1], fan_zero_rpm_enable: false, fan_stop_temp: 50,
                fan_start_temp: 60, temp_limit_0: 105, caps: [true, true, true, true, true, true, false],
            });
        }
        if frev == 0x08 && b.len() >= 0x4A {
            let mut st = Self {
                power_limit_w: 150, tdc_gfx: 144, tdc_soc: 0,
                oc_gfx_clock: [0, u32_at(0x017) / 100], oc_mem_clock: [0, u32_at(0x01B) / 100],
                oc_power_limit: [-50, i16::from_le_bytes([b[0x01F], b[0x020]]) as i32],
                volt_gfx: [800, 1000], volt_soc: [0, 0], freq_gfx: [852, 1200], freq_soc: [600, 1200],
                freq_fclk: [0, 0], mem_dpm_freq: [[600, 1], [900, 1], [900, 1], [900, 1]],
                mem_dpm_volt: [[900, 1200]; 4], fan_pwm_min: 20, fan_max_rpm: 3000,
                fan_acoustic_limit: 0, fan_throttling_rpm: 0, fan_target_temp: 74,
                fan_target_gfx_clock: 1200, fan_zero_rpm_enable: false, fan_stop_temp: 50,
                fan_start_temp: 60, temp_limit_0: 115, caps: [true, true, true, true, true, true, false],
            };
            let vddc_off = u16_at(0x03E) as usize;
            if vddc_off > 0 && vddc_off + 2 <= b.len() {
                let cnt = b[vddc_off + 1] as usize;
                let mut lut = Vec::new();
                for i in 0..cnt {
                    let eo = vddc_off + 2 + i * 2;
                    if eo + 2 <= b.len() { lut.push(u16_at(eo)); }
                }
                if let (Some(&vmin), Some(&vmax)) = (lut.first(), lut.last()) { st.volt_gfx = [vmin, vmax]; }
            }
            let vmem_off = u16_at(0x040) as usize;
            let (mut vddci_v, mut mvdd_v) = (900, 1200);
            if vmem_off > 0 && vmem_off + 4 <= b.len() {
                mvdd_v = u16_at(vmem_off + 2);
                if vmem_off + 8 <= b.len() { vddci_v = u16_at(vmem_off + 6); }
            }
            let read_dep = |ptr_off: usize, lo: i32, hi: i32| -> Option<[i32; 2]> {
                let off = u16_at(ptr_off) as usize;
                if off == 0 || off + 2 > b.len() { return None; }
                let cnt = b[off + 1] as usize;
                let mut vals = Vec::new();
                for i in 0..cnt {
                    let eo = off + 2 + i * 13;
                    if eo + 4 <= b.len() {
                        let mhz = u32_at(eo) / 100;
                        if (lo..=hi).contains(&mhz) { vals.push(mhz); }
                    }
                }
                Some([*vals.first()?, *vals.last()?])
            };
            if let Some(g) = read_dep(0x03A, 100, 2500) { st.freq_gfx = g; }
            if let Some(s) = read_dep(0x038, 100, 1800) { st.freq_soc = s; }
            let uclk_off = u16_at(0x03C) as usize;
            if uclk_off > 0 && uclk_off + 2 <= b.len() {
                let cnt = b[uclk_off + 1] as usize;
                let mut mclks = Vec::new();
                for i in 0..cnt {
                    let eo = uclk_off + 2 + i * 13;
                    if eo + 4 <= b.len() {
                        let mhz = u32_at(eo) / 100;
                        if (100..=1600).contains(&mhz) { mclks.push(mhz); }
                    }
                }
                for idx in 0..4 {
                    let clk = mclks.get(idx).or_else(|| mclks.last()).copied().unwrap_or(0);
                    st.mem_dpm_freq[idx] = [clk, 1];
                    st.mem_dpm_volt[idx] = [vddci_v, mvdd_v];
                }
            }
            if st.oc_gfx_clock[1] == 0 { st.oc_gfx_clock[1] = st.freq_gfx[1]; }
            if st.oc_mem_clock[1] == 0 { st.oc_mem_clock[1] = st.mem_dpm_freq[3][0]; }
            st.fan_target_gfx_clock = st.freq_gfx[1];
            let hl_off = u16_at(0x048) as usize;
            if hl_off > 0 && hl_off + 0x16 <= b.len() {
                let t_hot = b[hl_off + 0x15] as i32;
                if (60..=125).contains(&t_hot) { st.temp_limit_0 = t_hot; }
                let t_op = b[hl_off + 0x0E] as i32;
                if (50..=95).contains(&t_op) { st.fan_target_temp = t_op; }
            }
            let pt_off = u16_at(0x042) as usize;
            if pt_off > 0 && pt_off + 0x59 <= b.len() {
                let tdc_v = u16_at(pt_off + 0x55);
                let pwr_v = u16_at(pt_off + 0x57);
                if (40..=400).contains(&tdc_v) { st.tdc_gfx = tdc_v; }
                if (50..=450).contains(&pwr_v) { st.power_limit_w = pwr_v; }
            }
            let pcaps = u32_at(0x013) as u32;
            let odn_ac = (pcaps & (1 << 18)) != 0;
            st.caps = [
                odn_ac || st.oc_gfx_clock[1] > 0, odn_ac || st.oc_mem_clock[1] > 0,
                odn_ac || st.oc_power_limit[1] > 0, (pcaps & (1 << 11)) != 0 || st.fan_pwm_min > 0,
                (pcaps & (1 << 12)) != 0 || st.fan_target_temp > 0, (pcaps & (1 << 2)) != 0 || st.temp_limit_0 > 0,
                (pcaps & (1 << 14)) != 0 || st.fan_zero_rpm_enable,
            ];
            return Some(st);
        }
        None
    }
    pub fn write_to_buf(&self, buf: &mut [u8]) -> bool {
        let frev = buf.get(2).copied().unwrap_or(0);
        if frev == 0x0B && buf.len() == 1534 && buf.len() >= 0x53C {
            buf[0x016..0x018].copy_from_slice(&(self.power_limit_w.max(0) as u16).to_le_bytes());
            buf[0x018..0x01A].copy_from_slice(&(self.tdc_gfx.max(0) as u16).to_le_bytes());
            buf[0x01A..0x01C].copy_from_slice(&(self.tdc_soc.max(0) as u16).to_le_bytes());
            buf[0x027..0x02B].copy_from_slice(&(self.oc_gfx_clock[1].max(0) as u32).to_le_bytes());
            buf[0x02B..0x02F].copy_from_slice(&(self.oc_mem_clock[1].max(0) as u32).to_le_bytes());
            buf[0x03B..0x03F].copy_from_slice(&(self.volt_gfx[0].max(0) as u32).to_le_bytes());
            buf[0x043..0x047].copy_from_slice(&(self.volt_gfx[1].max(0) as u32).to_le_bytes());
            buf[0x04F..0x053].copy_from_slice(&(self.oc_gfx_clock[0].max(0) as u32).to_le_bytes());
            buf[0x33A..0x33C].copy_from_slice(&(self.freq_gfx[0].max(0) as u16).to_le_bytes());
            buf[0x348..0x34A].copy_from_slice(&(self.freq_gfx[1].max(0) as u16).to_le_bytes());
            for i in 0..4 {
                let off = 0x3C4 + i * 2;
                buf[off..off + 2].copy_from_slice(&(self.mem_dpm_freq[i][0].max(0) as u16).to_le_bytes());
            }
            buf[0x536..0x538].copy_from_slice(&(self.fan_acoustic_limit.max(0) as u16).to_le_bytes());
            buf[0x538..0x53A].copy_from_slice(&(self.fan_throttling_rpm.max(0) as u16).to_le_bytes());
            buf[0x53A..0x53C].copy_from_slice(&(self.fan_max_rpm.max(0) as u16).to_le_bytes());
            return true;
        }
        if frev == 0x08 && buf.len() >= 0x4A {
            let u16_at = |b: &[u8], o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as usize;
            let mut pcaps = u32::from_le_bytes([buf[0x013], buf[0x014], buf[0x015], buf[0x016]]);
            if self.caps[0] || self.caps[1] || self.caps[2] { pcaps |= 1 << 18; } else { pcaps &= !(1 << 18); }
            if self.caps[3] { pcaps |= 1 << 11; } else { pcaps &= !(1 << 11); }
            if self.caps[4] { pcaps |= 1 << 12; } else { pcaps &= !(1 << 12); }
            if self.caps[5] { pcaps |= 1 << 2; } else { pcaps &= !(1 << 2); }
            if self.caps[6] { pcaps |= 1 << 14; } else { pcaps &= !(1 << 14); }
            buf[0x013..0x017].copy_from_slice(&pcaps.to_le_bytes());
            buf[0x017..0x01B].copy_from_slice(&((self.oc_gfx_clock[1].max(0) * 100) as u32).to_le_bytes());
            buf[0x01B..0x01F].copy_from_slice(&((self.oc_mem_clock[1].max(0) * 100) as u32).to_le_bytes());
            buf[0x01F..0x021].copy_from_slice(&(self.oc_power_limit[1] as i16).to_le_bytes());
            let pt_off = u16_at(buf, 0x042);
            if pt_off > 0 && pt_off + 0x59 <= buf.len() {
                buf[pt_off + 0x55..pt_off + 0x57].copy_from_slice(&(self.tdc_gfx.max(0) as u16).to_le_bytes());
                buf[pt_off + 0x57..pt_off + 0x59].copy_from_slice(&(self.power_limit_w.max(0) as u16).to_le_bytes());
            }
            return true;
        }
        false
    }
}
