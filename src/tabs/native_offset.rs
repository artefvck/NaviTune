use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name as off};
use crate::pptable_rdna3::{AtomSmcDpmInfoV4_12, field_offset_by_name as off3};
use crate::PowerPlayApp;
use eframe::egui;

pub(crate) type NativeStructEntries = (&'static str, Vec<(&'static str, &'static str, usize, usize, String)>);

pub(crate) fn collect_native_struct_entries(b: &[u8]) -> Option<NativeStructEntries> {
    let hdr_sz = if b.len() >= 2 { u16::from_le_bytes([b[0], b[1]]) as usize } else { 0 };
    let (frev, crev, trev) = (b.get(2).copied().unwrap_or(0), b.get(3).copied().unwrap_or(0), b.get(4).copied().unwrap_or(0));
    if let Some(t3) = AtomSmcDpmInfoV4_12::from_bytes(b) {
        return Some(("RDNA 3 (V4_12)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_feature_flags[0..16]", off3("od_features"), 16, format!("{:?}", &t3.od_feature_flags[..16])),
            ("OD8", "od_gfx_max / od_gfx_max_2", off3("od_gfx"), 8, format!("{} / {} MHz", {t3.od_gfx_max}, {t3.od_gfx_max_2})),
            ("OD8", "od_mem_max / od_mem_max_2", off3("od_mem"), 8, format!("{} / {} MHz", {t3.od_mem_max}, {t3.od_mem_max_2})),
            ("OD8", "od_pwr_max", off3("od_pwr"), 4, format!("{} %", {t3.od_pwr_max})),
            ("OD8", "od_fan_max / od_acoustic_max", off3("od_fan"), 8, format!("{} / {} RPM", {t3.od_fan_max}, {t3.od_acoustic_max})),
            ("OD8", "od_temp_target / od_temp_max", off3("od_temp"), 8, format!("{} / {} °C", {t3.od_temp_target}, {t3.od_temp_max})),
            ("SMU", "smc_pptable_if_version", off3("smc_if_ver"), 4, format!("0x{:08X}", {t3.smc_pptable_if_version})),
            ("SMU", "features_to_run (u64)", off3("smu_features"), 8, format!("0x{:016X}", {t3.features_to_run})),
            ("Power", "socket_power_limit_ac", off3("pwr_limit_w"), 2, format!("{} W", {t3.socket_power_limit_ac})),
            ("Thermal", "temp_limit[0..5]", off3("temp_limit"), 10, format!("[{}, {}, {}, {}, {}] °C", {t3.temp_limit[0]}, {t3.temp_limit[1]}, {t3.temp_limit[2]}, {t3.temp_limit[3]}, {t3.temp_limit[4]})),
            ("Voltage", "min/max_voltage_gfx", off3("volt_gfx"), 4, format!("{}..{} mV", {t3.min_voltage_gfx} / 4, {t3.max_voltage_gfx} / 4)),
            ("Voltage", "min/max_voltage_soc", off3("volt_soc"), 4, format!("{}..{} mV", {t3.min_voltage_soc} / 4, {t3.max_voltage_soc} / 4)),
            ("Clocks", "freq_gfx_min / max", off3("freq_gfx"), 4, format!("{}..{} MHz", {t3.freq_gfx_min}, {t3.freq_gfx_max})),
            ("Clocks", "freq_soc_max", off3("freq_soc"), 2, format!("{} MHz", {t3.freq_soc_max})),
            ("Clocks", "freq_fclk_min / max", off3("freq_fclk"), 4, format!("{}..{} MHz", {t3.freq_fclk_min}, {t3.freq_fclk_max})),
            ("Memory", "freq_uclk[0..3]", off3("mem_dpm_freq"), 8, format!("{:?} MHz", {t3.freq_uclk})),
            ("Power", "tdc_limit_gfx / soc", off3("tdc_gfx"), 4, format!("{} A / {} A", {t3.tdc_limit_gfx}, {t3.tdc_limit_soc})),
            ("Fan", "stop / start / target / zero", off3("fan_stop_temp"), 4, format!("Stop={}°C, Start={}°C, Target={}°C, MaxRPM={}, ZeroRPM={}", {t3.fan_stop_temp}, {t3.fan_start_temp}, { let tt = t3.fan_target_temp; if tt > 200 { tt / 10 } else { tt } }, {t3.fan_max_rpm}, {t3.fan_zero_rpm_enable})),
        ]));
    }
    if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(b) {
        let od_max = { t1.od_settings_max };
        let od_flags = { t1.od_feature_flags };
        let ppt_ac = { t1.socket_power_limit_ac };
        let ft_gfx = { t1.freq_table_gfx };
        let ft_uclk = { t1.freq_table_uclk };
        return Some(("RDNA 1 (V4_5)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_feature_flags[0..12]", 0x09B, 12, format!("{:?}", &od_flags[..12])),
            ("OD8", "od_settings_max[0..5]", 0x02F, 20, format!("GFX={}, Mem={}, Pwr=+{}%, Fan={}", od_max[0], od_max[1], od_max[2], od_max[3])),
            ("Power", "socket_power_limit_ac / tdc", 0x0B7, 12, format!("PPT0={}W, TDC_GFX={}A, TDC_Soc={}A", ppt_ac[0], {t1.tdc_limit_gfx}, {t1.tdc_limit_soc})),
            ("Voltage", "GFX / Soc Vmin..Vmax", 0x0F3, 8, format!("GFX={}..{}mV, Soc={}..{}mV", {t1.min_voltage_gfx}/4, {t1.max_voltage_gfx}/4, {t1.min_voltage_soc}/4, {t1.max_voltage_soc}/4)),
            ("Clocks", "freq_table_gfx / uclk", 0x0FF, 88, format!("GFX={}..{}MHz, UCLK_DPM3={}MHz", ft_gfx[0], ft_gfx[15], ft_uclk[3])),
            ("Fan", "stop / start / target / zero", 0x679, 32, format!("Stop={}C, Start={}C, Target={}C, MaxRPM={}, ZeroRPM={}", {t1.fan_stop_temp}, {t1.fan_start_temp}, {t1.fan_target_temperature}, {t1.fan_maximum_rpm}, t1.fan_zero_rpm_enable)),
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
            ("Clocks", "freq_table_gfx[0..7]", 0x33A, 16, format!("GFX={}..{}MHz", u16_at(0x33A), u16_at(0x348))),
            ("Memory", "freq_table_uclk[0..3]", 0x38A, 8, format!("UCLK={}..{}MHz", u16_at(0x3C4), u16_at(0x3CA))),
        ]));
    }
    if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(b) {
        let od_max = { v20.od_settings_max };
        let ppt_ac = { v20.socket_power_limit_ac };
        return Some(("Vega 20 (0x0B)", vec![
            ("Header", "AtomCommonTableHeader", 0x000, 5, format!("size={}, rev=0x{:02X}.{:02X}.{:02X}", hdr_sz, frev, crev, trev)),
            ("OD8", "od_settings_max[0..4]", 0x055, 16, format!("GFX={}, Mem={}, Pwr=+{}%, Fan={}", od_max[0], od_max[1], od_max[2], od_max[3])),
            ("Power", "socket_power_limit_ac / tdc", 0x11D, 12, format!("PPT0={}W, TDC_GFX={}A, TDC_Soc={}A", ppt_ac[0], {v20.tdc_limit_gfx}, {v20.tdc_limit_soc})),
            ("Voltage", "GFX / Soc Vmin..Vmax", 0x159, 8, format!("GFX={}..{}mV, Soc={}..{}mV", {v20.min_voltage_gfx}/4, {v20.max_voltage_gfx}/4, {v20.min_voltage_soc}/4, {v20.max_voltage_soc}/4)),
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
        ("OD8", "od_gfx_max / min", off("od_gfx"), 4, format!("{} / {} MHz", {t.od_gfx_max[0]}, {t.od_gfx_min[0]})),
        ("OD8", "od_mem_max / min", off("od_mem"), 4, format!("{} / {} MHz", {t.od_mem_max[0]}, {t.od_mem_min[0]})),
        ("OD8", "od_pwr_max / min", off("od_pwr"), 4, format!("{} / {} %", {t.od_pwr_max}, {t.od_pwr_min})),
        ("OD8", "od_fan_max / min", off("od_fan"), 4, format!("{} / {} RPM", {t.od_fan_max}, {t.od_fan_min})),
        ("PowerModes", "Quiet / Balanced Limit", off("pm_quiet"), 8, format!("{}% / {}% ({}°C / {}°C)", {t.pm_quiet_limit}, {t.pm_balanced_limit}, {t.pm_quiet_temp}, {t.pm_balanced_temp})),
        ("PowerModes", "Turbo / Rage Limit", off("pm_turbo"), 8, format!("{}% / {}% ({}°C / {}°C)", {t.pm_turbo_limit}, {t.pm_rage_limit}, {t.pm_turbo_temp}, {t.pm_rage_temp})),
        ("SMU", "features_to_run (u64)", 0x326, 8, format!("0x{:016X}", {t.features_to_run})),
        ("Power", "socket_power_limit_ac", 0x32E, 2, format!("{} W", {t.socket_power_limit_ac[0]})),
        ("Power", "tdc_limit_gfx / soc", off("tdc_gfx"), 4, format!("{} A / {} A", {t.tdc_limit_gfx}, {t.tdc_limit_soc})),
        ("Voltage", "min/max_voltage_gfx", off("volt_gfx"), 8, format!("{}..{} mV", {t.min_voltage_gfx} / 4, {t.max_voltage_gfx} / 4)),
        ("Voltage", "min/max_voltage_soc", off("volt_soc"), 8, format!("{}..{} mV", {t.min_voltage_soc} / 4, {t.max_voltage_soc} / 4)),
        ("Clocks", "freq_gfx_min / max", 0x52E, 4, format!("{}..{} MHz", {t.freq_gfx_min}, {t.freq_gfx_max})),
        ("Clocks", "freq_soc[0..1]", off("freq_soc"), 4, format!("{}..{} MHz", {t.freq_soc[0]}, {t.freq_soc[1]})),
        ("Clocks", "freq_fclk[0..1]", off("freq_fclk"), 4, format!("{}..{} MHz", {t.freq_fclk[0]}, {t.freq_fclk[1]})),
        ("Memory", "freq_uclk[0..3]", off("mem_dpm_freq"), 8, format!("{:?} MHz", {t.freq_uclk})),
        ("Fan", "zero_rpm / stop / start", off("fan_zero_rpm"), 6, format!("en={}, stop={}°C, start={}°C", {t.fan_zero_rpm_enable}, {t.fan_stop_temp}, {t.fan_start_temp})),
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
