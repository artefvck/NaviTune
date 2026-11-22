use core::mem::{offset_of, size_of};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AtomCommonTableHeaderPolaris {
    pub structure_size: u16,
    pub table_format_revision: u8,
    pub table_content_revision: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolarisChip {
    Ellesmere,
    Baffin,
    Lexa,
    UnknownPolaris,
}

impl PolarisChip {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Ellesmere => "Polaris 10/20/30 (RX 470 / 480 / 570 / 580 / 590)",
            Self::Baffin => "Polaris 11/21 (RX 460 / 560 / Pro WX 4100)",
            Self::Lexa => "Polaris 12 (RX 550 / Pro WX 2100 / 3100)",
            Self::UnknownPolaris => "Polaris (GCN 4)",
        }
    }

    pub fn from_did(did: u16) -> Self {
        match did {
            0x67C0..=0x67DF | 0x6FDF => Self::Ellesmere,
            0x67E0..=0x67FF => Self::Baffin,
            0x6980..=0x699F => Self::Lexa,
            _ => Self::UnknownPolaris,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AtomPolarisPowerPlayTable {
    pub header: AtomCommonTableHeaderPolaris,
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
    pub state_array_offset: u16,
    pub fan_table_offset: u16,
    pub thermal_controller_offset: u16,
    pub reserved_0: u16,
    pub mclk_dep_table_offset: u16,
    pub sclk_dep_table_offset: u16,
    pub vddc_lookup_table_offset: u16,
    pub vddgfx_lookup_table_offset: u16,
    pub mm_dep_table_offset: u16,
    pub vce_state_table_offset: u16,
    pub uvd_clock_table_offset: u16,
    pub samu_clock_table_offset: u16,
    pub ppm_table_offset: u16,
    pub acp_table_offset: u16,
    pub cac_tdp_table_offset: u16,
    pub smu_telemetry_table_offset: u16,
    pub pcie_table_offset: u16,
    pub bi_table_offset: u16,
    pub hard_limit_table_offset: u16,
    pub payload: [u8; 311],
}

const _: () = {
    assert!(size_of::<AtomPolarisPowerPlayTable>() == 384);
    assert!(offset_of!(AtomPolarisPowerPlayTable, max_od_engine_clock) == 0x017);
    assert!(offset_of!(AtomPolarisPowerPlayTable, max_od_memory_clock) == 0x01B);
    assert!(offset_of!(AtomPolarisPowerPlayTable, power_control_limit) == 0x01F);
    assert!(offset_of!(AtomPolarisPowerPlayTable, fan_table_offset) == 0x025);
    assert!(offset_of!(AtomPolarisPowerPlayTable, hard_limit_table_offset) == 0x047);
};

impl AtomPolarisPowerPlayTable {
    pub fn from_bytes(buf: &[u8]) -> Option<&Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x07) {
            return None;
        }
        Some(unsafe { &*(buf.as_ptr() as *const Self) })
    }

    pub fn from_bytes_mut(buf: &mut [u8]) -> Option<&mut Self> {
        if buf.len() < size_of::<Self>() || buf.get(2) != Some(&0x07) {
            return None;
        }
        Some(unsafe { &mut *(buf.as_mut_ptr() as *mut Self) })
    }

    pub fn resolved_hard_limit_offset(&self, buf: &[u8]) -> usize {
        for cand in [{ self.hard_limit_table_offset }, { self.cac_tdp_table_offset }, { self.acp_table_offset }] {
            let off = cand as usize;
            if off >= 0x4D && off + 0x16 <= buf.len() && buf[off] >= 1 && buf[off + 1] >= 1 {
                return off;
            }
        }
        0
    }

    pub fn read_hard_limits(&self, buf: &[u8]) -> Option<(i32, i32, i32)> {
        let (_, gfx, mem, vddc, _, _) = crate::pptable_vega::read_gcn_hard_limit_full(buf, self.resolved_hard_limit_offset(buf), false)?;
        Some((gfx, mem, vddc))
    }

    pub fn read_hard_limit_mem_volt(&self, buf: &[u8]) -> Option<(i32, i32)> {
        let (_, _, _, _, vddci, mvdd) = crate::pptable_vega::read_gcn_hard_limit_full(buf, self.resolved_hard_limit_offset(buf), false)?;
        Some((vddci, mvdd))
    }

    pub fn write_hard_limits_full(buf: &mut [u8], gfx: i32, mem: i32, vddc: i32, vddci: i32, mvdd: i32) {
        if let Some(t) = Self::from_bytes(buf) {
            let off = t.resolved_hard_limit_offset(buf);
            crate::pptable_vega::write_gcn_hard_limit_full(buf, off, false, 0, gfx, mem, vddc, vddci, mvdd);
        }
    }

    pub fn read_fan_subtable(&self, buf: &[u8]) -> Option<(i32, i32, i32, i32, bool, i32, i32)> {
        let off = { self.fan_table_offset } as usize;
        if off == 0 || off + 0x1D > buf.len() { return None; }
        let rev = buf[off];
        if rev >= 10 {
            return crate::pptable_vega::read_gcn_fan_subtable(buf, off);
        }
        let pwm_min = (u16::from_le_bytes([buf[off + 0x08], buf[off + 0x09]]) / 100) as i32;
        let max_rpm = u16::from_le_bytes([buf[off + 0x15], buf[off + 0x16]]) as i32;
        let tgt_gfx = (u32::from_le_bytes([buf[off + 0x17], buf[off + 0x18], buf[off + 0x19], buf[off + 0x1A]]) / 100) as i32;
        let tgt_temp = buf[off + 0x1B] as i32;
        Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, false, 0, 0))
    }

    pub fn write_fan_subtable(buf: &mut [u8], pwm_min: i32, max_rpm: i32, tgt_temp: i32, tgt_gfx: i32, zrpm: bool, stop_t: i32, start_t: i32) {
        let Some(t) = Self::from_bytes(buf) else { return; };
        let off = { t.fan_table_offset } as usize;
        if off == 0 || off + 0x1D > buf.len() { return; }
        let rev = buf[off];
        if rev >= 10 {
            crate::pptable_vega::write_gcn_fan_subtable(buf, off, pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t);
            return;
        }
        buf[off + 0x08..off + 0x0A].copy_from_slice(&((pwm_min.max(0) as u16) * 100).to_le_bytes());
        buf[off + 0x15..off + 0x17].copy_from_slice(&(max_rpm.max(0) as u16).to_le_bytes());
        buf[off + 0x17..off + 0x1B].copy_from_slice(&((tgt_gfx.max(0) as u32) * 100).to_le_bytes());
        buf[off + 0x1B] = tgt_temp.clamp(0, 255) as u8;
    }
}
