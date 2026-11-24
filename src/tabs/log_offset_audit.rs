use crate::arch::detect::PptArch;
use crate::arch::polaris::AtomPolarisPowerPlayTable;
use crate::arch::rdna1::AtomSmcDpmInfoV4_5;
use crate::arch::rdna3::AtomSmcDpmInfoV4_12;
use crate::arch::vega::VegaParsedState;
use crate::PowerPlayApp;
use eframe::egui;

pub(crate) struct AuditEntry {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) width: &'static str,
    pub(crate) ui_val: i32,
    pub(crate) struct_val: i32,
}

impl PowerPlayApp {
    pub(crate) fn collect_offset_audit_rows(&self) -> (PptArch, Vec<AuditEntry>) {
        let buf = &self.raw_buffer;
        let arch = PptArch::detect(buf);
        let frev = buf.get(2).copied().unwrap_or(0);

        let u16_at = |o: usize| if o + 2 <= buf.len() { u16::from_le_bytes([buf[o], buf[o + 1]]) as i32 } else { 0 };
        let u32_at = |o: usize| if o + 4 <= buf.len() { u32::from_le_bytes([buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]) as i32 } else { 0 };

        let mut rows: Vec<AuditEntry> = Vec::new();
        let mut push = |field: &'static str, offset: usize, width: &'static str, ui_val: i32, struct_val: i32| {
            rows.push(AuditEntry { field, offset, width, ui_val, struct_val });
        };

        match arch {
            PptArch::Polaris => {
                if let Some(pol) = AtomPolarisPowerPlayTable::from_bytes(buf) {
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomPolarisPowerPlayTable, max_od_engine_clock), "u32 / 100", self.oc_gfx_clock[1], (pol.max_od_engine_clock / 100) as i32);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomPolarisPowerPlayTable, max_od_memory_clock), "u32 / 100", self.oc_mem_clock[1], (pol.max_od_memory_clock / 100) as i32);
                    push("OD Power Limit (%)", std::mem::offset_of!(AtomPolarisPowerPlayTable, power_control_limit), "i16", self.oc_power_limit[1], pol.power_control_limit as i32);
                }
            }
            PptArch::Vega10 => {
                use crate::arch::vega::AtomVega10PowerPlayTable;
                if let Some(st) = VegaParsedState::parse(buf, frev) {
                    let pt_off = u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, hard_limit_table_offset)) as usize;
                    push("GFX Clock Min (MHz)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, gfxclk_dep_table_offset)) as usize + 2, "u32 / 100", self.freq_gfx[0], st.freq_gfx[0]);
                    push("GFX Clock Max (MHz)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, gfxclk_dep_table_offset)) as usize + 2, "u32 / 100", self.freq_gfx[1], st.freq_gfx[1]);
                    push("SOCCLK Max (MHz)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, socclk_dep_table_offset)) as usize + 2, "u32 / 100", self.freq_soc[1], st.freq_soc[1]);
                    push("HBM2 DPM3 Clock (MHz)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, mclk_dep_table_offset)) as usize + 2, "u32 / 100", self.mem_dpm_freq[3][0], st.mem_dpm_freq[3][0]);
                    push("GFX Voltage Min (mV)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, vddc_lookup_table_offset)) as usize + 2, "u16", self.volt_gfx[0], st.volt_gfx[0]);
                    push("GFX Voltage Max (mV)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, vddc_lookup_table_offset)) as usize + 2, "u16", self.volt_gfx[1], st.volt_gfx[1]);
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomVega10PowerPlayTable, max_od_engine_clock), "u32 / 100", self.oc_gfx_clock[1], st.oc_gfx_clock[1]);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomVega10PowerPlayTable, max_od_memory_clock), "u32 / 100", self.oc_mem_clock[1], st.oc_mem_clock[1]);
                    push("OD Power Limit (%)", std::mem::offset_of!(AtomVega10PowerPlayTable, power_control_limit), "i16", self.oc_power_limit[1], st.oc_power_limit[1]);
                    push("TDC GFX Limit (A)", pt_off + 0x55, "u16", self.tdc_gfx, st.tdc_gfx);
                    push("Max Power Limit (W)", pt_off + 0x57, "u16", self.power_limit_w, st.power_limit_w);
                    push("Hotspot Temp Limit (°C)", u16_at(std::mem::offset_of!(AtomVega10PowerPlayTable, hard_limit_table_offset)) as usize + 0x15, "u8", self.temp_limits[0], st.temp_limit_0);
                }
            }
            PptArch::Vega20 => {
                use crate::arch::vega::AtomVega20PowerPlayTable;
                if let Some(st) = VegaParsedState::parse(buf, frev) {
                    push("Max Power Limit (W)", std::mem::offset_of!(AtomVega20PowerPlayTable, socket_power_limit_ac), "u16", self.power_limit_w, st.power_limit_w);
                    push("TDC GFX Limit (A)", std::mem::offset_of!(AtomVega20PowerPlayTable, tdc_limit_gfx), "u16", self.tdc_gfx, st.tdc_gfx);
                    push("TDC SoC Limit (A)", std::mem::offset_of!(AtomVega20PowerPlayTable, tdc_limit_soc), "u16", self.tdc_soc, st.tdc_soc);
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max), "u32", self.oc_gfx_clock[1], st.oc_gfx_clock[1]);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max) + 4, "u32", self.oc_mem_clock[1], st.oc_mem_clock[1]);
                    push("GFX Voltage Min (mV)", std::mem::offset_of!(AtomVega20PowerPlayTable, min_voltage_gfx), "u32", self.volt_gfx[0], st.volt_gfx[0]);
                    push("GFX Voltage Max (mV)", std::mem::offset_of!(AtomVega20PowerPlayTable, max_voltage_gfx), "u32", self.volt_gfx[1], st.volt_gfx[1]);
                    push("GFX Clock Min (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, freq_table_gfx), "u16", self.freq_gfx[0], st.freq_gfx[0]);
                    push("GFX Clock Max (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, freq_table_gfx) + 14 * 2, "u16", self.freq_gfx[1], st.freq_gfx[1]);
                    push("UCLK DPM0 (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, freq_table_uclk), "u16", self.mem_dpm_freq[0][0], st.mem_dpm_freq[0][0]);
                    push("UCLK DPM3 (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, freq_table_uclk) + 3 * 2, "u16", self.mem_dpm_freq[3][0], st.mem_dpm_freq[3][0]);
                    push("Fan Acoustic Limit (RPM)", std::mem::offset_of!(AtomVega20PowerPlayTable, fan_acoustic_limit_rpm), "u16", self.fan_acoustic_limit, st.fan_acoustic_limit);
                    push("Fan Throttling (RPM)", std::mem::offset_of!(AtomVega20PowerPlayTable, fan_throttling_rpm), "u16", self.fan_throttling_rpm, st.fan_throttling_rpm);
                    push("Fan Max RPM (RPM)", std::mem::offset_of!(AtomVega20PowerPlayTable, fan_maximum_rpm), "u16", self.fan_max_rpm, st.fan_max_rpm);
                } else {
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max), "u32", self.oc_gfx_clock[1], u32_at(std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max)));
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max) + 4, "u32", self.oc_mem_clock[1], u32_at(std::mem::offset_of!(AtomVega20PowerPlayTable, od_settings_max) + 4));
                    push("GFX Voltage Min (mV)", std::mem::offset_of!(AtomVega20PowerPlayTable, min_voltage_gfx), "u32", self.volt_gfx[0], u32_at(std::mem::offset_of!(AtomVega20PowerPlayTable, min_voltage_gfx)));
                    push("GFX Voltage Max (mV)", std::mem::offset_of!(AtomVega20PowerPlayTable, max_voltage_gfx), "u32", self.volt_gfx[1], u32_at(std::mem::offset_of!(AtomVega20PowerPlayTable, max_voltage_gfx)));
                }
            }
            PptArch::Rdna1 => {
                let mut matched_v45 = false;
                if let Some(t) = AtomSmcDpmInfoV4_5::from_bytes(buf) {
                    matched_v45 = true;
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, od_settings_max), "u32", self.oc_gfx_clock[1], t.od_settings_max[0] as i32);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, od_settings_max) + 8, "u32", self.oc_mem_clock[1], t.od_settings_max[2] as i32);
                    push("Socket Power Limit (W)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, socket_power_limit_ac), "u16", self.power_limit_w, t.socket_power_limit_ac[0] as i32);
                    push("TDC GFX Limit (A)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, tdc_limit_gfx), "u16", self.tdc_gfx, t.tdc_limit_gfx as i32);
                    push("GFX Voltage Min (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, min_voltage_gfx), "u16 / 4", self.volt_gfx[0], (t.min_voltage_gfx / 4) as i32);
                    push("GFX Voltage Max (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, max_voltage_gfx), "u16 / 4", self.volt_gfx[1], (t.max_voltage_gfx / 4) as i32);
                    push("GFX Clock Min (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, freq_table_gfx), "u16", self.freq_gfx[0], t.freq_table_gfx[0] as i32);
                    push("GFX Clock Max (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, freq_table_gfx) + 2, "u16", self.freq_gfx[1], t.freq_table_gfx[1] as i32);
                    push("UCLK DPM0 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, freq_table_uclk), "u16", self.mem_dpm_freq[0][0], t.freq_table_uclk[0] as i32);
                    push("UCLK DPM3 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, freq_table_uclk) + 6, "u16", self.mem_dpm_freq[3][0], t.freq_table_uclk[3] as i32);
                    push("Fan Max RPM (RPM)", std::mem::offset_of!(AtomSmcDpmInfoV4_5, fan_maximum_rpm), "u16", self.fan_max_rpm, t.fan_maximum_rpm as i32);
                }
                if !matched_v45 && buf.len() >= 1674 {
                    if let Some(r1) = crate::arch::rdna1::Rdna1ParsedState::parse(buf) {
                        let inn = r1.outer_sz;
                        push("OD Max GFX Clock (MHz)", 0x0E2, "u32", self.oc_gfx_clock.last().copied().unwrap_or(0), r1.od_settings_max[0] as i32);
                        push("OD Max Mem Clock (MHz)", 0x0EE, "u32", self.oc_mem_clock.last().copied().unwrap_or(0), r1.od_settings_max[1] as i32);
                        push("od_mem_timing_ctrl", 0x11A, "u32", self.oc_mem_timing_ctrl, r1.od_mem_timing_ctrl as i32);
                        push("od_pwr_mode_ctrl", 0x11E, "u32", self.oc_power_mode_ctrl, r1.od_pwr_mode_ctrl as i32);
                        push("od_zero_rpm_ctrl", 0x122, "u32", self.oc_zero_rpm_ctrl, r1.od_zero_rpm_ctrl as i32);
                        push("socket_power_limit_ac", inn + 0x00C, "u16", self.power_limit_w, r1.power_limit_w as i32);
                        push("temp_limit_edge (°C)", inn + 0x034, "u16", self.temp_limits[0], r1.temp_limit[0] as i32);
                        push("temp_limit_hotspot (°C)", inn + 0x036, "u16", self.temp_limits[1], r1.temp_limit[1] as i32);
                        push("min_voltage_gfx (mV)", inn + 0x068, "u16/4", self.volt_gfx.first().copied().unwrap_or(0), (r1.min_volt_gfx / 4) as i32);
                        push("max_voltage_gfx (mV)", inn + 0x06C, "u16/4", self.volt_gfx.last().copied().unwrap_or(0), (r1.max_volt_gfx / 4) as i32);
                        push("freq_gfx_min (MHz)", inn + 0x14C, "u16", self.freq_gfx.first().copied().unwrap_or(0), r1.gfx_min as i32);
                        push("freq_gfx_max (MHz)", inn + 0x14E, "u16", self.freq_gfx.last().copied().unwrap_or(0), r1.gfx_max as i32);
                        push("fan_throttling_rpm", inn + 0x298, "u16", self.fan_throttling_rpm, r1.fan_throttling_rpm as i32);
                        push("fan_max_rpm", inn + 0x29A, "u16", self.fan_max_rpm, r1.fan_max_rpm as i32);
                        push("fan_target_temperature", inn + 0x29C, "u16", self.fan_target_temp, r1.fan_target_temp as i32);
                    } else {
                        let inn = 0x01E2usize;
                        push("socket_power_limit_ac", inn + 0x00C, "u16", self.power_limit_w, u16_at(inn + 0x00C));
                        push("min_voltage_gfx (mV)", inn + 0x068, "u16/4", self.volt_gfx.first().copied().unwrap_or(0), u16_at(inn + 0x068) / 4);
                        push("max_voltage_gfx (mV)", inn + 0x06C, "u16/4", self.volt_gfx.last().copied().unwrap_or(0), u16_at(inn + 0x06C) / 4);
                        push("freq_gfx_min (MHz)", inn + 0x14C, "u16", self.freq_gfx.first().copied().unwrap_or(0), u16_at(inn + 0x14C));
                        push("freq_gfx_max (MHz)", inn + 0x14E, "u16", self.freq_gfx.last().copied().unwrap_or(0), u16_at(inn + 0x14E));
                        push("fan_target_temperature", inn + 0x29C, "u16", self.fan_target_temp, u16_at(inn + 0x29C));
                    }
                }
            }
            PptArch::Rdna2 => {
                use crate::arch::rdna2::AtomSmcDpmInfoV4_9;
                if let Some(t) = AtomSmcDpmInfoV4_9::from_bytes(buf) {
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, od_gfx_max), "u32", self.oc_gfx_clock[1], t.od_gfx_max[0] as i32);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, od_mem_max), "u32", self.oc_mem_clock[1], t.od_mem_max[0] as i32);
                    push("Socket Power Limit (W)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, socket_power_limit_ac), "u16", self.power_limit_w, t.socket_power_limit_ac[0] as i32);
                    push("TDC GFX Limit (A)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, tdc_limit_gfx), "u16", self.tdc_gfx, t.tdc_limit_gfx as i32);
                    push("GFX Voltage Min (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, min_voltage_gfx), "u16 / 4", self.volt_gfx[0], (t.min_voltage_gfx / 4) as i32);
                    push("GFX Voltage Max (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, max_voltage_gfx), "u16 / 4", self.volt_gfx[1], (t.max_voltage_gfx / 4) as i32);
                    push("GFX Clock Min (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, freq_gfx_min), "u16", self.freq_gfx[0], t.freq_gfx_min as i32);
                    push("GFX Clock Max (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, freq_gfx_max), "u16", self.freq_gfx[1], t.freq_gfx_max as i32);
                    push("UCLK DPM0 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, freq_uclk), "u16", self.mem_dpm_freq[0][0], t.freq_uclk[0] as i32);
                    push("UCLK DPM3 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, freq_uclk) + 6, "u16", self.mem_dpm_freq[3][0], t.freq_uclk[3] as i32);
                    push("Fan Max RPM (RPM)", std::mem::offset_of!(AtomSmcDpmInfoV4_9, fan_max_rpm), "u16", self.fan_max_rpm, t.fan_max_rpm as i32);
                }
            }
            PptArch::Rdna3 => {
                if let Some(t) = AtomSmcDpmInfoV4_12::from_bytes(buf) {
                    push("OD Max GFX Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, od_gfx_max), "u32", self.oc_gfx_clock[1], t.od_gfx_max as i32);
                    push("OD Max Mem Clock (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, od_mem_max), "u32", self.oc_mem_clock[1], t.od_mem_max as i32);
                    push("Socket Power Limit (W)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, socket_power_limit_ac), "u16", self.power_limit_w, t.socket_power_limit_ac as i32);
                    push("TDC GFX Limit (A)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, tdc_limit_gfx), "u16", self.tdc_gfx, t.tdc_limit_gfx as i32);
                    push("GFX Voltage Min (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, min_voltage_gfx), "u16 / 4", self.volt_gfx[0], (t.min_voltage_gfx / 4) as i32);
                    push("GFX Voltage Max (mV)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, max_voltage_gfx), "u16 / 4", self.volt_gfx[1], (t.max_voltage_gfx / 4) as i32);
                    push("GFX Clock Min (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_gfx_min), "u16", self.freq_gfx[0], t.freq_gfx_min as i32);
                    push("GFX Clock Max (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_gfx_max), "u16", self.freq_gfx[1], t.freq_gfx_max as i32);
                    push("SOCCLK Max (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_soc_max), "u16", self.freq_soc[1], t.freq_soc_max as i32);
                    push("FCLK Max (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_fclk_max), "u16", self.freq_fclk[1], t.freq_fclk_max as i32);
                    push("UCLK DPM0 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_uclk), "u16", self.mem_dpm_freq[0][0], t.freq_uclk[0] as i32);
                    push("UCLK DPM3 (MHz)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, freq_uclk) + 6, "u16", self.mem_dpm_freq[3][0], t.freq_uclk[3] as i32);
                    push("Fan Max RPM (RPM)", std::mem::offset_of!(AtomSmcDpmInfoV4_12, fan_max_rpm), "u16", self.fan_max_rpm, t.fan_max_rpm as i32);
                }
            }
            PptArch::Unknown => {}
        }

        (arch, rows)
    }

    pub(crate) fn draw_debug_audit(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(175, 190, 210),
                "No PPTable / VBIOS loaded. Open a .bin or .rom file to run Struct vs UI Offset Audit.",
            );
            return;
        }

        self.sync_fields_to_buffer();
        let buf = &self.raw_buffer;
        let vbios_base = self.vbios_ppt_offset.unwrap_or(0);
        let (arch, rows) = self.collect_offset_audit_rows();
        let mismatches = rows.iter().filter(|r| r.ui_val != r.struct_val).count();
        ui.horizontal(|ui| {
            ui.strong(format!("Struct vs UI Parity Audit — {:?} ({} B)", arch, buf.len()));
            ui.separator();
            if mismatches == 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(80, 215, 130),
                    format!("SYNC OK ({}/{} fields match)", rows.len(), rows.len()),
                );
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(245, 85, 95),
                    format!("DRIFT DETECTED ({} mismatch(es))", mismatches),
                );
            }
        });
        ui.add_space(6.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("offset_audit_parity_grid")
                    .num_columns(6)
                    .striped(true)
                    .spacing([14.0, 5.0])
                    .show(ui, |ui| {
                        ui.strong("Field");
                        ui.strong("PPTable Off");
                        ui.strong("VBIOS Off");
                        ui.strong("UI Value");
                        ui.strong("Struct Value");
                        ui.strong("Status");
                        ui.end_row();

                        for r in &rows {
                            let ok = r.ui_val == r.struct_val;
                            ui.label(r.field);
                            ui.monospace(format!("0x{:04X} ({})", r.offset, r.width));
                            ui.monospace(if self.vbios_ppt_offset.is_some() {
                                format!("0x{:06X}", vbios_base + r.offset)
                            } else {
                                "—".to_string()
                            });
                            ui.monospace(format!("{}", r.ui_val));
                            ui.monospace(format!("{}", r.struct_val));
                            if ok {
                                ui.colored_label(egui::Color32::from_rgb(80, 215, 130), "MATCH");
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(245, 85, 95), "MISMATCH");
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}

#[cfg(test)]
mod offset_audit_tab_tests {
    use crate::app::PowerPlayApp;

    #[test]
    fn collect_offset_audit_rows_covers_all_six_architectures() {
        for (frev, len) in [(0x07u8, 833usize), (0x08, 654), (0x0B, 1534), (0x0C, 1674), (0x0F, 2470), (0x14, 5424)] {
            let mut app = PowerPlayApp::default();
            let mut buf = std::iter::repeat_n(0u8, len).collect::<Vec<u8>>();
            let sz = (len as u16).to_le_bytes();
            if let Some(b0) = buf.get_mut(0) { *b0 = sz.first().copied().unwrap_or(0); }
            if let Some(b1) = buf.get_mut(1) { *b1 = sz.last().copied().unwrap_or(0); }
            if let Some(b2) = buf.get_mut(2) { *b2 = frev; }
            if let Some(b4) = buf.get_mut(4) { *b4 = 0x03; }
            app.raw_buffer = buf;
            let (_arch, rows) = app.collect_offset_audit_rows();
            assert!(!rows.is_empty(), "expected audit rows for frev=0x{:02X} len={}", frev, len);
        }
    }
}
