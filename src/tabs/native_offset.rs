use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name as off};
use crate::pptable_rdna3::{AtomSmcDpmInfoV4_12, field_offset_by_name as off3};
use crate::PowerPlayApp;
use eframe::egui;

pub(crate) type NativeStructEntries = (&'static str, Vec<(&'static str, &'static str, usize, usize, String)>);

pub(crate) fn collect_native_struct_entries(b: &[u8]) -> Option<NativeStructEntries> {
    let hdr_sz = if b.len() >= 2 { u16::from_le_bytes([b[0], b[1]]) as usize } else { 0 };
    let (frev, crev, trev) = (b.get(2).copied().unwrap_or(0), b.get(3).copied().unwrap_or(0), b.get(4).copied().unwrap_or(0));
    if let Some(t3) = AtomSmcDpmInfoV4_12::from_bytes(b) {
        let (vg_lo, vg_hi) = if t3.min_voltage_gfx > t3.max_voltage_gfx {
            ({t3.max_voltage_gfx} / 4, {t3.min_voltage_gfx} / 4)
        } else {
            ({t3.min_voltage_gfx} / 4, {t3.max_voltage_gfx} / 4)
        };
        let (gfx_off, gfx_lo) = if t3.header.format_revision >= 0x16 && t3.freq_gfx_min < 500 && b.len() > 0x695 {
            (0x694usize, u16::from_le_bytes([b[0x694], b[0x695]]))
        } else {
            (off3("freq_gfx"), {t3.freq_gfx_min})
        };
        return Some(("RDNA 3 (V4_12)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_feature_flags[0..20]", off3("od_features"), 20, format!("{:?}", t3.od_feature_flags)),
            ("OD8", "od_gfx_max / od_gfx_max_2", off3("od_gfx"), 8, format!("{} / {} MHz", {t3.od_gfx_max}, {t3.od_gfx_max_2})),
            ("OD8", "od_mem_max / od_mem_max_2", off3("od_mem"), 8, format!("{} / {} MHz", {t3.od_mem_max}, {t3.od_mem_max_2})),
            ("OD8", "od_pwr_max", off3("od_pwr"), 4, format!("+{} %", {t3.od_pwr_max})),
            ("OD8", "od_fan_max / od_acoustic_max", off3("od_fan"), 8, format!("{} / {} RPM", {t3.od_fan_max}, {t3.od_acoustic_max})),
            ("OD8", "od_temp_target / od_temp_max", off3("od_temp"), 8, format!("{} / {} °C", {t3.od_temp_target}, {t3.od_temp_max})),
            ("OD8", "od_gfx_min / od_gfx_min_2", off3("od_gfx_min"), 8, format!("{} / {} MHz", {t3.od_gfx_min}, {t3.od_gfx_min_2})),
            ("OD8", "od_mem_min / od_mem_min_2", off3("od_mem_min"), 8, format!("{} / {} MHz", {t3.od_mem_min}, {t3.od_mem_min_2})),
            ("OD8", "od_pwr_min", off3("od_pwr_min"), 4, format!("-{} %", {t3.od_pwr_min})),
            ("OD8", "od_fan_min / od_acoustic_min", off3("od_fan_min"), 8, format!("{} / {} RPM", {t3.od_fan_min}, {t3.od_acoustic_min})),
            ("SMU", "smc_pptable_if_version", off3("smc_if_ver"), 4, format!("0x{:08X}", {t3.smc_pptable_if_version})),
            ("SMU", "features_to_run (u64)", off3("smu_features"), 8, format!("0x{:016X}", {t3.features_to_run})),
            ("Power", "socket_power_limit_ac / ac_2", off3("pwr_limit_w"), 4, format!("{} / {} W", {t3.socket_power_limit_ac}, {t3.socket_power_limit_ac_2})),
            ("Power", "tdc_limit_gfx / soc", off3("tdc_gfx"), 4, format!("{} A / {} A", {t3.tdc_limit_gfx}, {t3.tdc_limit_soc})),
            ("Thermal", "temp_limit[0..9]", off3("temp_limit"), 18, format!("{:?} °C", {t3.temp_limit})),
            ("Voltage", "min/max_voltage_gfx", off3("volt_gfx"), 4, format!("{}..{} mV", vg_lo, vg_hi)),
            ("Voltage", "min/max_voltage_soc", off3("volt_soc"), 4, format!("{}..{} mV", {t3.min_voltage_soc} / 4, {t3.max_voltage_soc} / 4)),
            ("Voltage", "ulv_voltage_vmin_gfx / soc", off3("ulv_gfx"), 4, format!("GFX={}mV, Soc={}mV", {t3.ulv_voltage_vmin_gfx} / 4, {t3.ulv_voltage_vmin_soc} / 4)),
            ("Clocks", "freq_gfx_min / max", gfx_off, 4, format!("{}..{} MHz", gfx_lo, {t3.freq_gfx_max})),
            ("Clocks", "freq_soc_max", off3("freq_soc"), 2, format!("{} MHz", {t3.freq_soc_max})),
            ("Clocks", "freq_fclk_min / max", off3("freq_fclk"), 4, format!("{}..{} MHz", {t3.freq_fclk_min}, {t3.freq_fclk_max})),
            ("Memory", "freq_uclk[0..3]", off3("mem_dpm_freq"), 8, format!("{:?} MHz", {t3.freq_uclk})),
            ("Memory", "mem_vddci[0..3]", off3("mem_dpm_volt"), 8, format!("DPM3={} mV", {t3.mem_vddci[3]} / 4)),
            ("Memory", "mem_mvdd[0..3]", 0x7DC, 8, format!("DPM3={} mV", {t3.mem_mvdd[3]} / 4)),
            ("PCIe", "pcie_gen / lanes / lclk", off3("pcie"), 12, format!("Gen {:?}, Lanes {:?}, LCLK {:?}", {t3.pcie_gen_speed}, {t3.pcie_lane_count}, {t3.lclk_freq})),
            ("Fan", "fan_stop_temp", off3("fan_stop_temp"), 2, format!("{} °C", {t3.fan_stop_temp})),
            ("Fan", "fan_start_temp", off3("fan_start_temp"), 2, format!("{} °C", {t3.fan_start_temp})),
            ("Fan", "fan_throttle / pwm / acoustic / max", off3("fan_throttling"), 10, format!("Throttle={} RPM, PWM={}%, Ac={} RPM, Max={} RPM", {t3.fan_throttling_rpm}, {t3.fan_pwm_min}, {t3.fan_acoustic_limit}, {t3.fan_max_rpm})),
            ("Fan", "fan_target_gfx / temp / zero_rpm", 0x876, 7, format!("TgtGFX={} MHz, Tgt={} °C, ZeroRPM={}", {t3.fan_target_gfx_clock}, { let tt = t3.fan_target_temp; if tt > 200 { tt / 10 } else { tt } }, {t3.fan_zero_rpm_enable})),
        ]));
    }
    if let Some(r1) = crate::pptable_rdna1::Rdna1ParsedState::parse(b) {
        let inn = r1.outer_sz;
        return Some(("RDNA 1 (1674 B · Navi 10 / 14)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_feature_flags[0..16]", 0x0C2, 16, format!("{:?}", r1.od_feature_flags)),
            ("OD8", "od_settings_max[0..5]", 0x0E2, 48, format!("GFX={}, Mem={}, Pwr=+{}%, Fan={}, Ac={}", r1.od_settings_max[0], r1.od_settings_max[1], r1.od_settings_max[2], r1.od_settings_max[3], r1.od_settings_max[4])),
            ("OD8", "od_mem_timing / pwr_mode / zero_rpm", 0x11A, 12, format!("MemTiming={}, PwrMode={}, ZeroRPM={}", r1.od_mem_timing_ctrl, r1.od_pwr_mode_ctrl, r1.od_zero_rpm_ctrl)),
            ("SubTable", "PP_PhmSoftPowerPlayTable", inn, (b.len() - inn).min(16), format!("SMC DPM inner table @ 0x{:03X}", inn)),
            ("SMU", "features_to_run (u64)", inn + 0x004, 8, format!("0x{:016X}", r1.smu_features)),
            ("Power", "power_limit_w / tdc_gfx / tdc_soc", inn + 0x00C, 38, format!("{} W | TDC GFX={} A, Soc={} A", r1.power_limit_w, r1.tdc_gfx, r1.tdc_soc)),
            ("Thermal", "temp_limit[0..9]", inn + 0x034, 18, format!("{:?} °C", r1.temp_limit)),
            ("OD8", "od_settings_min[0..5]", 0x162, 48, format!("GFX={}, Mem={}, Pwr=-{}%, Fan={}, Ac={}", r1.od_settings_min[0], r1.od_settings_min[1], r1.od_settings_min[2], r1.od_settings_min[3], r1.od_settings_min[4])),
            ("Voltage", "ulv_vmin_gfx / soc", inn + 0x064, 4, format!("GFX={} mV, Soc={} mV", r1.ulv_vmin_gfx / 4, r1.ulv_vmin_soc / 4)),
            ("Voltage", "min/max_voltage_gfx / soc", inn + 0x068, 8, format!("GFX={}..{}mV, Soc={}..{}mV", r1.min_volt_gfx / 4, r1.max_volt_gfx / 4, r1.min_volt_soc / 4, r1.max_volt_soc / 4)),
            ("Curve", "curve_gfx (a,b,c)", inn + 0x080, 12, format!("a={:.6}, b={:.6}, c={:.6}", r1.curve_gfx[0], r1.curve_gfx[1], r1.curve_gfx[2])),
            ("Clocks", "gfx_min / gfx_max", inn + 0x14C, 4, format!("{} .. {} MHz", r1.gfx_min, r1.gfx_max)),
            ("Clocks", "soc_min / soc_max", inn + 0x16C, 4, format!("{} .. {} MHz", r1.soc_min, r1.soc_max)),
            ("Memory", "uclk_dpm[0..3]", inn + 0x19C, 8, format!("{:?} MHz", r1.uclk_dpm)),
            ("Clocks", "fclk_min / fclk_max", inn + 0x1B4, 4, format!("{} .. {} MHz", r1.fclk_min, r1.fclk_max)),
            ("Memory", "mem_vddci / mem_mvdd[0..3]", inn + 0x240, 16, format!("DPM3 VDDCI={}mV, MVDD={}mV", r1.mem_vddci[3] / 4, r1.mem_mvdd[3] / 4)),
            ("Fan", "fan_stop_temp / fan_start_temp", inn + 0x27C, 4, format!("Stop={} °C, Start={} °C", r1.fan_stop_temp, r1.fan_start_temp)),
            ("Fan", "fan_pwm_min / throttle / max / tgt", inn + 0x294, 16, format!("PWM {}%, Throttle {} RPM, Max {} RPM, Target {} °C, ZeroRPM={}", r1.fan_pwm_min, r1.fan_throttling_rpm, r1.fan_max_rpm, r1.fan_target_temp, r1.fan_zero_rpm)),
        ]));
    }
    if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(b) {
        let od_max = { t1.od_settings_max };
        let od_min = { t1.od_settings_min };
        let od_flags = { t1.od_feature_flags };
        let ppt_ac = { t1.socket_power_limit_ac };
        let ft_gfx = { t1.freq_table_gfx };
        let ft_soc = { t1.freq_table_socclk };
        let ft_uclk = { t1.freq_table_uclk };
        let ft_fclk = { t1.freq_table_fclk };
        let tl = { t1.temp_limit };
        return Some(("RDNA 1 (V4_5)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_settings_max[0..5]", 0x02F, 20, format!("GFX={}, Mem={}, Pwr=+{}%, Fan={}", od_max[0], od_max[1], od_max[2], od_max[3])),
            ("OD8", "od_settings_min[0..5]", 0x063, 20, format!("GFX={}, Mem={}, Pwr=-{}%, Fan={}", od_min[0], od_min[1], od_min[2], od_min[3])),
            ("OD8", "od_feature_flags[0..16]", 0x09B, 16, format!("{:?}", od_flags)),
            ("SMU", "features_to_run (u64)", 0x0AF, 8, format!("0x{:016X}", {t1.features_to_run})),
            ("Power", "socket_power_limit_ac[0..3]", 0x0B7, 8, format!("{:?} W", ppt_ac)),
            ("Power", "tdc_limit_gfx / soc", 0x0D7, 4, format!("{} A / {} A", {t1.tdc_limit_gfx}, {t1.tdc_limit_soc})),
            ("Memory", "vddci_max / mvdd_max", 0x0DF, 4, format!("{} / {} mV", {t1.vddci_max}/4, {t1.mvdd_max}/4)),
            ("Voltage", "min_voltage_ulv_gfx / soc", 0x0EF, 4, format!("GFX={}mV, Soc={}mV", {t1.min_voltage_ulv_gfx}/4, {t1.min_voltage_ulv_soc}/4)),
            ("Voltage", "GFX / Soc Vmin..Vmax", 0x0F3, 8, format!("GFX={}..{}mV, Soc={}..{}mV", {t1.min_voltage_gfx}/4, {t1.max_voltage_gfx}/4, {t1.min_voltage_soc}/4, {t1.max_voltage_soc}/4)),
            ("Clocks", "freq_table_gfx[0..15]", 0x0FF, 32, format!("{}..{} MHz", ft_gfx[0], ft_gfx[15])),
            ("Clocks", "freq_table_socclk[0..7]", 0x13F, 16, format!("{}..{} MHz", ft_soc[0], ft_soc[7])),
            ("Clocks", "freq_table_uclk[0..3]", 0x14F, 8, format!("{:?} MHz", ft_uclk)),
            ("Clocks", "freq_table_fclk[0..7]", 0x197, 16, format!("{}..{} MHz", ft_fclk[0], ft_fclk[7])),
            ("Thermal", "temp_limit[0..7]", 0x66B, 14, format!("{:?} °C", tl)),
            ("Fan", "fan_stop_temp / fan_start_temp", 0x679, 4, format!("Stop={}°C, Start={}°C", {t1.fan_stop_temp}, {t1.fan_start_temp})),
            ("Fan", "fan_pwm_min..zero_rpm", 0x68B, 14, format!("PWM={}%, MaxRPM={}, Tgt={}°C, ZeroRPM={}", {t1.fan_pwm_min}, {t1.fan_maximum_rpm}, {t1.fan_target_temperature}, t1.fan_zero_rpm_enable)),
        ]));
    }
    if b.len() == 1534 && frev == 0x0B {
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        let u32_at = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        return Some(("Vega 20 Pro (0x0B · 1534B)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("Power", "BoostPower / MaxCurrent", 0x016, 6, format!("Boost={}W, TDC_GFX={}A, TDC_Soc={}A", u16_at(0x016), u16_at(0x018), u16_at(0x01A))),
            ("OD8", "od_settings_max (GFX/Mem)", 0x027, 8, format!("GFX={}MHz, Mem={}MHz", u32_at(0x027), u32_at(0x02B))),
            ("Voltage", "GFX Vmin..Vmax (OD8)", 0x03B, 12, format!("GFX={}..{}mV (Ceiling={}mV)", u32_at(0x03B), u32_at(0x043), u32_at(0x047))),
            ("Thermal", "temp_limit[0..7]", 0x1FA, 14, format!("[{}, {}, {}, {}, {}, {}, {}] °C", u16_at(0x1FA), u16_at(0x1FC), u16_at(0x1FE), u16_at(0x200), u16_at(0x202), u16_at(0x204), u16_at(0x206))),
            ("Clocks", "freq_table_gfx[0..7]", 0x33A, 16, format!("GFX={}..{}MHz", u16_at(0x33A), u16_at(0x348))),
            ("Memory", "freq_table_uclk[0..3]", 0x38A, 8, format!("UCLK={}..{}MHz", u16_at(0x3C4), u16_at(0x3CA))),
        ]));
    }
    if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(b) {
        let od_max = { v20.od_settings_max };
        let od_min = { v20.od_settings_min };
        let od_flags = { v20.od_feature_flags };
        let ppt_ac = { v20.socket_power_limit_ac };
        let ft_gfx = { v20.freq_table_gfx };
        let ft_soc = { v20.freq_table_socclk };
        let ft_uclk = { v20.freq_table_uclk };
        let ft_fclk = { v20.freq_table_fclk };
        let tl = { v20.temp_limit };
        return Some(("Vega 20 (0x0B)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_settings_max[0..4]", 0x055, 16, format!("GFX={}, Mem={}, Pwr=+{}%, Fan={}", od_max[0], od_max[1], od_max[2], od_max[3])),
            ("OD8", "od_settings_min[0..4]", 0x0A9, 16, format!("GFX={}, Mem={}, Pwr=-{}%, Fan={}", od_min[0], od_min[1], od_min[2], od_min[3])),
            ("OD8", "od_feature_flags[0..16]", 0x101, 16, format!("{:?}", od_flags)),
            ("SMU", "features_to_run (u64)", 0x115, 8, format!("0x{:016X}", {v20.features_to_run})),
            ("Power", "socket_power_limit_ac[0..3]", 0x11D, 8, format!("{:?} W", ppt_ac)),
            ("Power", "tdc_limit_gfx / soc", 0x13D, 4, format!("{} A / {} A", {v20.tdc_limit_gfx}, {v20.tdc_limit_soc})),
            ("Memory", "vddci_max / mvdd_max", 0x145, 4, format!("{} / {} mV", {v20.vddci_max}/4, {v20.mvdd_max}/4)),
            ("Voltage", "min_voltage_ulv_gfx / soc", 0x155, 4, format!("GFX={}mV, Soc={}mV", {v20.min_voltage_ulv_gfx}/4, {v20.min_voltage_ulv_soc}/4)),
            ("Voltage", "GFX / Soc Vmin..Vmax", 0x159, 8, format!("GFX={}..{}mV, Soc={}..{}mV", {v20.min_voltage_gfx}/4, {v20.max_voltage_gfx}/4, {v20.min_voltage_soc}/4, {v20.max_voltage_soc}/4)),
            ("Clocks", "freq_table_gfx[0..15]", 0x165, 32, format!("{}..{} MHz", ft_gfx[0], ft_gfx[15])),
            ("Clocks", "freq_table_socclk[0..7]", 0x1A5, 16, format!("{}..{} MHz", ft_soc[0], ft_soc[7])),
            ("Clocks", "freq_table_uclk[0..3]", 0x1B5, 8, format!("{:?} MHz", ft_uclk)),
            ("Clocks", "freq_table_fclk[0..7]", 0x1FD, 16, format!("{}..{} MHz", ft_fclk[0], ft_fclk[7])),
            ("Thermal", "temp_limit[0..7]", 0x655, 14, format!("{:?} °C", tl)),
            ("Fan", "fan_stop_temp / fan_start_temp", 0x663, 4, format!("Stop={}°C, Start={}°C", {v20.fan_stop_temp}, {v20.fan_start_temp})),
            ("Fan", "fan_pwm_min..zero_rpm", 0x675, 14, format!("PWM={}%, MaxRPM={}, Tgt={}°C, ZeroRPM={}", {v20.fan_pwm_min}, {v20.fan_maximum_rpm}, {v20.fan_target_temperature}, v20.fan_zero_rpm_enable)),
        ]));
    }
    if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(b) {
        let fan_off = u16::from_le_bytes([b[0x034], b[0x035]]) as usize;
        let therm_off = u16::from_le_bytes([b[0x036], b[0x037]]) as usize;
        let hard_off = u16::from_le_bytes([b[0x048], b[0x049]]) as usize;
        let hard_desc = if let Some((soc_hi, gfx_hi, mem_hi, vddc_hi)) = v10.read_hard_limits(b) {
            let (vddci, mvdd) = v10.read_hard_limit_mem_volt(b).unwrap_or((0, 0));
            format!("off=0x{:03X} | SOCCLK={}MHz, GFXCLK={}MHz, MCLK={}MHz, VDDC={}mV, VDDCI={}mV, MVDD={}mV", hard_off, soc_hi, gfx_hi, mem_hi, vddc_hi, vddci, mvdd)
        } else {
            format!("off=0x{:03X} (sub-table out of range)", hard_off)
        };
        let fan_desc = if let Some((pwm, rpm, tgt, gfx, zrpm, stp, strt)) = v10.read_fan_subtable(b) {
            format!("off=0x{:03X} | PWM={}%, MaxRPM={}, Tgt={}C, TgtGFX={}MHz, ZeroRPM={} ({}..{}C)", fan_off, pwm, rpm, tgt, gfx, zrpm, stp, strt)
        } else {
            format!("off=0x{:03X} (sub-table out of range)", fan_off)
        };
        return Some(("Vega 10 (0x08)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OverDrive", "max_od_engine / memory / pwr", 0x017, 10, format!("GFX={}MHz, Mem={}MHz, Pwr=+{}%", {v10.max_od_engine_clock}/100, {v10.max_od_memory_clock}/100, {v10.power_control_limit})),
            ("SubTables", "state / fan / thermal / hard_limit", 0x030, 28, format!("State=0x{:03X}, Fan=0x{:03X}, Therm=0x{:03X}, HardLim=0x{:03X}", {v10.state_array_offset}, fan_off, therm_off, hard_off)),
            ("Fan SubTable", "ATOM_Vega10_Fan_Table", fan_off.min(b.len()), 32, fan_desc),
            ("HardLimit SubTable", "ATOM_Vega10_Hard_Limit_Table", hard_off.min(b.len()), 18, hard_desc),
        ]));
    }
    if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(b) {
        let (s_min, s_max, v_min, v_max, pwr_w, tdc_a, m_states) = pol.read_polaris_dpm_and_powertune(b);
        let m_desc = m_states.iter().enumerate().map(|(i, (mc, vci, mvd))| format!("DPM{}:{}MHz({}/{}mV)", i, mc, vci, mvd)).collect::<Vec<_>>().join(", ");
        let fan_off = ({ pol.fan_table_offset }) as usize;
        let fan_desc = if let Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, _, _, _)) = pol.read_fan_subtable(b) {
            format!("PWM={}% MaxRPM={} Tgt={}°C TgtGFX={}MHz", pwm_min, max_rpm, tgt_temp, tgt_gfx)
        } else {
            format!("off=0x{:03X} (absent)", fan_off)
        };
        let hard_off = pol.resolved_hard_limit_offset(b);
        let hard_desc = if let Some((gfx_hi, mem_hi, vddc_hi)) = pol.read_hard_limits(b) {
            let (vddci, mvdd) = pol.read_hard_limit_mem_volt(b).unwrap_or((0, 0));
            format!("off=0x{:03X} | GFXCLK={}MHz, MCLK={}MHz, VDDC={}mV, VDDCI={}mV, MVDD={}mV", hard_off, gfx_hi, mem_hi, vddc_hi, vddci, mvdd)
        } else {
            format!("off=0x{:03X} (absent in rev 0x{:02X}.{:02X})", hard_off, frev, crev)
        };
        let mut pol_rows = vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OverDrive", "max_od_engine / memory / pwr", 0x017, 10, format!("GFX={}MHz, Mem={}MHz, Pwr=+{}%", {pol.max_od_engine_clock}/100, {pol.max_od_memory_clock}/100, {pol.power_control_limit})),
            ("VDDC Lookup", "vddc_lookup_table", ({ pol.vddc_lookup_table_offset }) as usize, 10, format!("VDDC={}..{} mV", v_min, v_max)),
            ("SCLK DPM", "sclk_dep_table", ({ pol.sclk_dep_table_offset }) as usize, 15, format!("GFXCLK={}..{} MHz", s_min, s_max)),
            ("MCLK DPM", "mclk_dep_table", ({ pol.mclk_dep_table_offset }) as usize, 17, m_desc),
            ("Fan SubTable", "ATOM_Tonga_Fan_Table", fan_off.min(b.len()), 29, fan_desc),
            ("PowerTune", "ATOM_Polaris_PowerTune", if b.len() >= 0x3B { u16::from_le_bytes([b[0x39], b[0x3A]]) as usize } else { 0x2D1 }, 16, format!("MaxPwr={}W TDC={}A", pwr_w, tdc_a)),
        ];
        if hard_off > 0 {
            pol_rows.push(("HardLimit SubTable", "ATOM_Polaris_Hard_Limit_Table", hard_off.min(b.len()), 14, hard_desc));
        }
        return Some(("Polaris (0x07)", pol_rows));
    }
    let t = AtomSmcDpmInfoV4_9::from_bytes(b)?;
    Some(("RDNA 2 (V4_9)", vec![
        ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
        ("OD8", "od_feature_flags[0..16]", 0x0C2, 16, format!("{:?}", {t.od_feature_flags})),
        ("OD8", "od_gfx_max[0..1]", off("od_gfx"), 8, format!("{} / {} MHz", {t.od_gfx_max[0]}, {t.od_gfx_max[1]})),
        ("OD8", "od_mem_max[0..1]", off("od_mem"), 8, format!("{} / {} MHz", {t.od_mem_max[0]}, {t.od_mem_max[1]})),
        ("OD8", "od_pwr_max", off("od_pwr"), 4, format!("+{} %", {t.od_pwr_max})),
        ("OD8", "od_fan_max", off("od_fan"), 4, format!("{} RPM", {t.od_fan_max})),
        ("OD8", "od_acoustic_max", off("od_acoustic"), 4, format!("{} RPM", {t.od_acoustic_max})),
        ("OD8", "od_mem_timing / pwr_mode", off("od_mem_timing"), 8, format!("MemTiming={}, PwrMode={}", {t.od_mem_timing_ctrl}, {t.od_pwr_mode_ctrl})),
        ("OD8", "od_zero_rpm_ctrl", off("od_zero_rpm"), 4, format!("{}", {t.od_zero_rpm_ctrl})),
        ("OD8", "od_fan_curve_max[0..5]", off("od_fan_temp"), 40, format!("({}°C, {}%)..({}°C, {}%)", {t.od_fan_curve_max[0].temp}, {t.od_fan_curve_max[0].pwm}, {t.od_fan_curve_max[4].temp}, {t.od_fan_curve_max[4].pwm})),
        ("OD8", "od_gfx_min[0..1]", 0x1E2, 8, format!("{} / {} MHz", {t.od_gfx_min[0]}, {t.od_gfx_min[1]})),
        ("OD8", "od_mem_min[0..1]", 0x1FA, 8, format!("{} / {} MHz", {t.od_mem_min[0]}, {t.od_mem_min[1]})),
        ("OD8", "od_pwr_min", 0x202, 4, format!("-{} %", {t.od_pwr_min})),
        ("OD8", "od_fan_min", 0x206, 4, format!("{} RPM", {t.od_fan_min})),
        ("OD8", "od_acoustic_min", 0x20A, 4, format!("{} RPM", {t.od_acoustic_min})),
        ("OD8", "od_fan_curve_min[0..5]", 0x22A, 40, format!("({}°C, {}%)..({}°C, {}%)", {t.od_fan_curve_min[0].temp}, {t.od_fan_curve_min[0].pwm}, {t.od_fan_curve_min[4].temp}, {t.od_fan_curve_min[4].pwm})),
        ("PowerModes", "pm_quiet..rage_limit", off("pm_quiet"), 16, format!("{} / {} / {} / {} W", {t.pm_quiet_limit}, {t.pm_balanced_limit}, {t.pm_turbo_limit}, {t.pm_rage_limit} & 0xFFFF)),
        ("PowerModes", "pm_quiet..rage_temp", 0x2EA, 8, format!("{} / {} / {} / {} °C", {t.pm_quiet_temp}, {t.pm_balanced_temp}, {t.pm_turbo_temp}, {t.pm_rage_temp})),
        ("PowerModes", "ac_quiet..rage", off("ac_quiet"), 16, format!("Q={:?}, B={:?}, T={:?}, R={:?}", {t.ac_quiet}, {t.ac_balanced}, {t.ac_turbo}, {t.ac_rage})),
        ("SMU", "features_to_run (u64)", 0x326, 8, format!("0x{:016X}", {t.features_to_run})),
        ("Power", "socket_power_limit_ac[0..3]", 0x32E, 8, format!("{:?} W", {t.socket_power_limit_ac})),
        ("Power", "socket_power_limit_dc", 0x33E, 2, format!("{} W", {t.socket_power_limit_dc})),
        ("Power", "power_limit_ac[0..3]", 0x346, 8, format!("{:?} W", {t.power_limit_ac})),
        ("Power", "tdc_limit_gfx / soc", off("tdc_gfx"), 4, format!("{} A / {} A", {t.tdc_limit_gfx}, {t.tdc_limit_soc})),
        ("Thermal", "temp_limit[0..9]", 0x356, 18, format!("{:?} °C", {t.temp_limit})),
        ("Voltage", "ulv_voltage_vmin / vmax", off("ulv_gfx"), 8, format!("GFX={}..{}mV, Soc={}..{}mV", {t.ulv_voltage_vmin_gfx}/4, {t.ulv_voltage_vmax_gfx}/4, {t.ulv_voltage_vmin_soc}/4, {t.ulv_voltage_vmax_soc}/4)),
        ("Voltage", "min/max_voltage_gfx", off("volt_gfx"), 8, format!("{}..{} mV", {t.min_voltage_gfx} / 4, {t.max_voltage_gfx} / 4)),
        ("Voltage", "min/max_voltage_soc", off("volt_soc"), 8, format!("{}..{} mV", {t.min_voltage_soc} / 4, {t.max_voltage_soc} / 4)),
        ("Voltage", "tvmin / vmin_low / vmin_high", off("tvmin_gfx"), 16, format!("T={}°C, Low={}mV, High={}mV", {t.tvmin_gfx}, {t.vmin_low_gfx}/4, {t.vmin_high_gfx}/4)),
        ("Clocks", "freq_gfx_min / max", 0x52E, 4, format!("{}..{} MHz", {t.freq_gfx_min}, {t.freq_gfx_max})),
        ("Clocks", "freq_soc[0..1]", off("freq_soc"), 4, format!("{}..{} MHz", {t.freq_soc[0]}, {t.freq_soc[1]})),
        ("Clocks", "freq_fclk[0..1]", off("freq_fclk"), 4, format!("{}..{} MHz", {t.freq_fclk[0]}, {t.freq_fclk[1]})),
        ("Clocks", "freq_gfx_min / max_mirror", 0x624, 4, format!("{}..{} MHz", {t.freq_gfx_min_mirror}, {t.freq_gfx_max_mirror})),
        ("Memory", "freq_uclk[0..3]", off("mem_dpm_freq"), 8, format!("{:?} MHz", {t.freq_uclk})),
        ("Memory", "mem_clk_div[0..3]", 0x65A, 4, format!("{:?}", {t.mem_clk_div})),
        ("Memory", "mem_vddci[0..3]", 0x66A, 8, format!("DPM3={} mV", {t.mem_vddci[3]}/4)),
        ("Memory", "mem_mvdd[0..3]", 0x672, 8, format!("DPM3={} mV", {t.mem_mvdd[3]}/4)),
        ("PCIe", "pcie_gen / lanes / lclk", 0x6F6, 8, format!("Gen {:?}, Lanes {:?}, LCLK {:?}", {t.pcie_gen_speed}, {t.pcie_lane_count}, {t.lclk_freq})),
        ("Fan", "fan_stop_temp / fan_start_temp", 0x6FE, 4, format!("Stop={}°C, Start={}°C", {t.fan_stop_temp}, {t.fan_start_temp})),
        ("Fan", "fan_pwm_min..target_gfx", off("fan_pwm_min"), 17, format!("PWM={}%, MaxRPM={}, Tgt={}°C, ZeroRPM={}", {t.fan_pwm_min}, {t.fan_max_rpm}, {t.fan_target_temp}, {t.fan_zero_rpm_enable})),
        ("Curve", "static_volt_gfx (a,b,c)", off("curve_gfx"), 12, format!("a={:.6}, b={:.6}, c={:.6}", {t.static_volt_gfx.a}, {t.static_volt_gfx.b}, {t.static_volt_gfx.c})),
        ("DcBtc", "dctol / dcbtc_max / gb", off("dctol"), 20, format!("DcTol={}/{} | Max={}/{} mV | GB={}/{} mV", {t.dctol_gfx}, {t.dctol_soc}, {t.dcbtc_max_gfx}/4, {t.dcbtc_max_soc}/4, {t.dcbtc_gb_gfx}/4, {t.dcbtc_gb_soc}/4)),
    ]))
}

impl PowerPlayApp {
    pub(crate) fn draw_debug_native_offset(&self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            ui.weak("No table loaded");
            return;
        }
        let b = &self.raw_buffer;
        let filter_id = egui::Id::new("offset_map_filter");
        let mut filter = ui.ctx().data(|d| d.get_temp::<String>(filter_id).unwrap_or_default());

        let Some((arch, entries)) = collect_native_struct_entries(b) else {
            ui.colored_label(egui::Color32::from_rgb(245, 185, 65), format!("Unsupported table size ({} B)", b.len()));
            return;
        };

        let mut report_lines = vec![format!("# PPTable Offset Map — {} ({} B)", arch, b.len())];
        for (sec, name, offset, len, val) in &entries {
            let hex: String = b[*offset..(*offset + *len).min(b.len())]
                .iter()
                .take(8)
                .map(|x| format!("{:02X}", x))
                .collect::<Vec<_>>()
                .join(" ");
            report_lines.push(format!("0x{:03X} | {:<10} | {:<26} | {:<23} | {}", offset, sec, name, hex, val));
        }
        let report = report_lines.join("\n");

        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(95, 200, 145), format!("{} · {} B", arch, b.len()));
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut filter)
                    .hint_text("Filter section, field, or 0x offset...")
                    .desired_width(220.0),
            );
            ui.ctx().data_mut(|d| d.insert_temp(filter_id, filter.clone()));
            if !filter.is_empty() && ui.small_button("✕").clicked() {
                ui.ctx().data_mut(|d| d.insert_temp(filter_id, String::new()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Copy Map").clicked() {
                    ui.ctx().copy_text(report);
                }
            });
        });
        ui.add_space(6.0);

        let q = filter.trim().to_lowercase();
        egui::ScrollArea::both()
            .id_salt("native_offset_full_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height())
            .show(ui, |ui| {
                egui::Grid::new("native_offset_map_grid")
                    .striped(true)
                    .spacing([14.0, 6.0])
                    .show(ui, |ui| {
                        for hdr in ["Section", "Offset", "Field", "Raw Hex", "Value"] {
                            ui.label(
                                egui::RichText::new(hdr)
                                    .size(11.5)
                                    .strong()
                                    .color(egui::Color32::from_rgb(160, 182, 214)),
                            );
                        }
                        ui.end_row();

                        for (sec, name, offset, len, val) in entries {
                            let off_str = format!("0x{:03X}", offset);
                            if !q.is_empty()
                                && !sec.to_lowercase().contains(&q)
                                && !name.to_lowercase().contains(&q)
                                && !off_str.to_lowercase().contains(&q)
                                && !val.to_lowercase().contains(&q)
                            {
                                continue;
                            }
                            let hex: String = b[offset..(offset + len).min(b.len())]
                                .iter()
                                .take(8)
                                .map(|x| format!("{:02X}", x))
                                .collect::<Vec<_>>()
                                .join(" ");

                            let sec_col = match sec {
                                "Header" | "SMU" => egui::Color32::from_rgb(145, 175, 225),
                                "OD8" | "Clocks" => egui::Color32::from_rgb(95, 200, 145),
                                "Power" | "Voltage" => egui::Color32::from_rgb(245, 185, 75),
                                "Thermal" | "Fan" => egui::Color32::from_rgb(235, 115, 125),
                                _ => egui::Color32::from_rgb(185, 155, 230),
                            };

                            ui.colored_label(sec_col, egui::RichText::new(sec).strong());
                            ui.monospace(off_str);
                            ui.monospace(name);
                            ui.colored_label(
                                egui::Color32::from_rgb(155, 168, 188),
                                egui::RichText::new(hex).monospace(),
                            );
                            ui.monospace(val);
                            ui.end_row();
                        }
                    });
            });
    }
}

#[cfg(test)]
mod native_offset_rdna1_1674_tests {
    use super::collect_native_struct_entries;

    #[test]
    fn native_offset_decodes_rdna1_1674b_thermal_fan_and_od_ctrls() {
        let mut b = vec![0u8; 1674];
        b[0..2].copy_from_slice(&1674u16.to_le_bytes());
        b[2] = 0x0C;
        b[5..7].copy_from_slice(&0x01E2u16.to_le_bytes());
        b[0x11A..0x11E].copy_from_slice(&1u32.to_le_bytes());
        b[0x11E..0x122].copy_from_slice(&1u32.to_le_bytes());
        b[0x122..0x126].copy_from_slice(&1u32.to_le_bytes());
        let inn = 0x01E2usize;
        b[inn + 0x034..inn + 0x036].copy_from_slice(&100u16.to_le_bytes());
        b[inn + 0x298..inn + 0x29A].copy_from_slice(&2600u16.to_le_bytes());
        b[inn + 0x29A..inn + 0x29C].copy_from_slice(&3100u16.to_le_bytes());

        let (arch, rows) = collect_native_struct_entries(&b).expect("1674B RDNA1 entries");
        assert!(arch.contains("1674 B"));
        assert!(rows.iter().any(|(_, f, off, _, _)| *f == "temp_limit[0..9]" && *off == inn + 0x034));
        assert!(rows.iter().any(|(_, f, off, _, v)| *f == "fan_pwm_min / throttle / max / tgt" && *off == inn + 0x294 && v.contains("Throttle 2600 RPM") && v.contains("Max 3100 RPM")));
        assert!(rows.iter().any(|(_, f, off, _, _)| *f == "od_mem_timing / pwr_mode / zero_rpm" && *off == 0x11A));
    }
}
