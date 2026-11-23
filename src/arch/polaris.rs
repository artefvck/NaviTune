pub type PolarisDpmPowertune = (i32, i32, i32, i32, i32, i32, Vec<(i32, i32, i32)>);

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
        let off = ({ self.hard_limit_table_offset }) as usize;
        if off >= 0x4D && off + 0x16 <= buf.len() && buf[off] >= 1 && buf[off + 1] >= 1 {
            return off;
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

    #[allow(clippy::too_many_arguments)]
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

    pub fn read_polaris_dpm_and_powertune(&self, buf: &[u8]) -> PolarisDpmPowertune {
        let mut sclk_min = 0i32;
        let mut sclk_max = 0i32;
        let soff = if buf.len() >= 0x2F { u16::from_le_bytes([buf[0x2D], buf[0x2E]]) as usize } else { ({ self.sclk_dep_table_offset }) as usize };
        if soff >= 0x4D && soff + 2 <= buf.len() {
            let n = buf[soff + 1] as usize;
            let stride = 15usize;
            if n > 0 && soff + 2 + n * stride <= buf.len() {
                let first = soff + 2;
                let last = soff + 2 + (n - 1) * stride;
                let c0 = u32::from_le_bytes([buf[first + 3], buf[first + 4], buf[first + 5], buf[first + 6]]) / 100;
                let cn = u32::from_le_bytes([buf[last + 3], buf[last + 4], buf[last + 5], buf[last + 6]]) / 100;
                sclk_min = c0 as i32;
                sclk_max = cn as i32;
            }
        }

        let mut vddc_min = 0i32;
        let mut vddc_max = 0i32;
        let voff = if buf.len() >= 0x31 { u16::from_le_bytes([buf[0x2F], buf[0x30]]) as usize } else { ({ self.vddc_lookup_table_offset }) as usize };
        if voff >= 0x4D && voff + 2 <= buf.len() {
            let n = buf[voff + 1] as usize;
            let stride = 8usize;
            if n > 0 && voff + 2 + n * stride <= buf.len() {
                let mut lo = i32::MAX;
                let mut hi = 0i32;
                for i in 0..n {
                    let mv = u16::from_le_bytes([buf[voff + 2 + i * stride], buf[voff + 3 + i * stride]]) as i32;
                    if (500..=1500).contains(&mv) {
                        lo = lo.min(mv);
                        hi = hi.max(mv);
                    }
                }
                if hi > 0 {
                    vddc_min = if lo == i32::MAX { 0 } else { lo };
                    vddc_max = hi;
                }
            }
        }

        let mut mclk_states = Vec::new();
        let moff = if buf.len() >= 0x2D { u16::from_le_bytes([buf[0x2B], buf[0x2C]]) as usize } else { ({ self.mclk_dep_table_offset }) as usize };
        if moff >= 0x4D && moff + 2 <= buf.len() {
            let n = buf[moff + 1] as usize;
            let stride = 13usize;
            if n > 0 && moff + 2 + n * stride <= buf.len() {
                for i in 0..n.min(4) {
                    let eoff = moff + 2 + i * stride;
                    let vddci = u16::from_le_bytes([buf[eoff + 1], buf[eoff + 2]]) as i32;
                    let mvdd = u16::from_le_bytes([buf[eoff + 5], buf[eoff + 6]]) as i32;
                    let mclk = (u32::from_le_bytes([buf[eoff + 7], buf[eoff + 8], buf[eoff + 9], buf[eoff + 10]]) / 100) as i32;
                    mclk_states.push((mclk, vddci, mvdd));
                }
            }
        }

        let mut pwr_w = 0i32;
        let mut tdc_a = 0i32;
        for i in 0x4D..buf.len().saturating_sub(0x32) {
            if buf[i] == 0x04 {
                let tdp = u16::from_le_bytes([buf[i + 1], buf[i + 2]]) as i32;
                let tdc = u16::from_le_bytes([buf[i + 5], buf[i + 6]]) as i32;
                let max_p = u16::from_le_bytes([buf[i + 0x2D], buf[i + 0x2E]]) as i32;
                if (30..=500).contains(&tdp) && (20..=400).contains(&tdc) && (30..=600).contains(&max_p) {
                    pwr_w = max_p;
                    tdc_a = tdc;
                    break;
                }
            }
        }

        (sclk_min, sclk_max, vddc_min, vddc_max, pwr_w, tdc_a, mclk_states)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_polaris_dpm_and_powertune(buf: &mut [u8], sclk_min: i32, sclk_max: i32, vddc_max: i32, pwr_w: i32, tdc_a: i32, mem_dpm_freq: &[[i32; 2]; 4], mem_dpm_volt: &[[i32; 2]; 4]) {
        let (soff, voff, moff) = {
            let Some(t) = Self::from_bytes(buf) else { return; };
            (({ t.sclk_dep_table_offset }) as usize, ({ t.vddc_lookup_table_offset }) as usize, ({ t.mclk_dep_table_offset }) as usize)
        };
        if soff >= 0x4D && soff + 2 <= buf.len() {
            let n = buf[soff + 1] as usize;
            let stride = 15usize;
            if n > 0 && soff + 2 + n * stride <= buf.len() {
                if sclk_min > 0 {
                    buf[soff + 5..soff + 9].copy_from_slice(&((sclk_min as u32) * 100).to_le_bytes());
                }
                if sclk_max > 0 {
                    let last = soff + 2 + (n - 1) * stride;
                    buf[last + 3..last + 7].copy_from_slice(&((sclk_max as u32) * 100).to_le_bytes());
                }
            }
        }
        if vddc_max > 0 && voff >= 0x4D && voff + 2 <= buf.len() {
            let n = buf[voff + 1] as usize;
            let stride = 8usize;
            if n > 0 && voff + 2 + n * stride <= buf.len() {
                let mut best_idx = None;
                let mut hi = 0i32;
                for i in 0..n {
                    let mv = u16::from_le_bytes([buf[voff + 2 + i * stride], buf[voff + 3 + i * stride]]) as i32;
                    if (500..=1500).contains(&mv) && mv >= hi {
                        hi = mv;
                        best_idx = Some(i);
                    }
                }
                if let Some(bi) = best_idx {
                    buf[voff + 2 + bi * stride..voff + 4 + bi * stride].copy_from_slice(&(vddc_max as u16).to_le_bytes());
                }
            }
        }
        if moff >= 0x4D && moff + 2 <= buf.len() {
            let n = buf[moff + 1] as usize;
            let stride = 13usize;
            if n > 0 && moff + 2 + n * stride <= buf.len() {
                for i in 0..n.min(4) {
                    let eoff = moff + 2 + i * stride;
                    if mem_dpm_volt[i][0] > 0 {
                        buf[eoff + 1..eoff + 3].copy_from_slice(&(mem_dpm_volt[i][0] as u16).to_le_bytes());
                    }
                    if mem_dpm_volt[i][1] > 0 {
                        buf[eoff + 5..eoff + 7].copy_from_slice(&(mem_dpm_volt[i][1] as u16).to_le_bytes());
                    }
                    if mem_dpm_freq[i][0] > 0 {
                        buf[eoff + 7..eoff + 11].copy_from_slice(&((mem_dpm_freq[i][0] as u32) * 100).to_le_bytes());
                    }
                }
            }
        }
        for i in 0x4D..buf.len().saturating_sub(0x32) {
            if buf[i] == 0x04 {
                let tdp = u16::from_le_bytes([buf[i + 1], buf[i + 2]]) as i32;
                let tdc = u16::from_le_bytes([buf[i + 5], buf[i + 6]]) as i32;
                let max_p = u16::from_le_bytes([buf[i + 0x2D], buf[i + 0x2E]]) as i32;
                if (30..=500).contains(&tdp) && (20..=400).contains(&tdc) && (30..=600).contains(&max_p) {
                    if tdc_a > 0 {
                        buf[i + 5..i + 7].copy_from_slice(&(tdc_a as u16).to_le_bytes());
                    }
                    if pwr_w > 0 {
                        buf[i + 0x2D..i + 0x2F].copy_from_slice(&(pwr_w as u16).to_le_bytes());
                    }
                    break;
                }
            }
        }
    }
}
