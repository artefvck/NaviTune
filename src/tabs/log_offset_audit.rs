use crate::arch::detect::PptArch;
use crate::arch::polaris::AtomPolarisPowerPlayTable;
use crate::arch::rdna1::AtomRdna1PowerPlayTable;
use crate::arch::rdna2::AtomRdna2PowerPlayTable;
use crate::arch::rdna3::AtomRdna3PowerPlayTable;
use crate::arch::vega::{AtomVega20PowerPlayTable, VegaParsedState};
use crate::PowerPlayApp;
use eframe::egui;

struct AuditEntry {
    field: &'static str,
    offset: usize,
    width: &'static str,
    ui_val: i32,
    struct_val: i32,
}

impl PowerPlayApp {
    pub(crate) fn draw_offset_audit_subtab(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(175, 190, 210),
                "No PPTable / VBIOS loaded. Open a .bin or .rom file to run Struct vs UI Offset Audit.",
            );
            return;
        }

        // Sync UI -> buffer first so buffer reflects latest edits, then parse struct directly from buffer
        self.sync_fields_to_buffer();
        let b = &self.raw_buffer;
        let arch = PptArch::detect(b);
        let frev = b.get(2).copied().unwrap_or(0);
        let vbios_base = self.vbios_ppt_offset.unwrap_or(0);

        let mut rows: Vec<AuditEntry> = Vec::new();
        let mut push = |field: &'static str, offset: usize, width: &'static str, ui_val: i32, struct_val: i32| {
            rows.push(AuditEntry { field, offset, width, ui_val, struct_val });
        };

        match arch {
            PptArch::Polaris => {
                if let Some(pol) = AtomPolarisPowerPlayTable::from_bytes(b) {
                    push("OD Max GFX Clock (MHz)", 0x019, "u32 / 100", self.oc_gfx_clock, (pol.max_od_engine_clock / 100) as i32);
                    push("OD Max Mem Clock (MHz)", 0x01D, "u32 / 100", self.oc_mem_clock, (pol.max_od_memory_clock / 100) as i32);
                    push("OD Power Limit (%)", 0x021, "i16", self.oc_power_limit, pol.power_control_limit as i32);
                }
            }
            PptArch::Vega10 => {
                if let Some(st) = VegaParsedState::parse(b, frev) {
                    let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as usize;
                    let pt_off = u16_at(0x042);
                    push("GFX Clock Min (MHz)", u16_at(0x03A) + 2, "u32 / 100", self.freq_gfx, st.freq_gfx);
                    push("GFX Clock Max (MHz)", u16_at(0x03A) + 2, "u32 / 100", self.freq_gfx, st.freq_gfx);
                    push("SOCCLK Max (MHz)", u16_at(0x038) + 2, "u32 / 100", self.freq_soc, st.freq_soc);
                    push("HBM2 DPM3 Clock (MHz)", u16_at(0x03C) + 2, "u32 / 100", self.mem_dpm_freq, st.mem_dpm_freq);
                    push("GFX Voltage Min (mV)", u16_at(0x03E) + 2, "u16", self.volt_gfx, st.volt_gfx);
                    push("GFX Voltage Max (mV)", u16_at(0x03E) + 2, "u16", self.volt_gfx, st.volt_gfx);
                    push("OD Max GFX Clock (MHz)", 0x017, "u32 / 100", self.oc_gfx_clock, st.oc_gfx_clock);
                    push("OD Max Mem Clock (MHz)", 0x01B, "u32 / 100", self.oc_mem_clock, st.oc_mem_clock);
                    push("OD Power Limit (%)", 0x01F, "i16", self.oc_power_limit, st.oc_power_limit);
                    push("TDC GFX Limit (A)", pt_off + 0x55, "u16", self.tdc_gfx, st.tdc_gfx);
                    push("Max Power Limit (W)", pt_off + 0x57, "u16", self.power_limit_w, st.power_limit_w);
                    push("Hotspot Temp Limit (°C)", u16_at(0x048) + 0x15, "u8", self.temp_limits, st.temp_limit_0);
                }
            }
            PptArch::Vega20 => {
                if let Some(st) = VegaParsedState::parse(b, frev) {
                    push("Max Power Limit (W)", 0x016, "u16", self.power_limit_w, st.power_limit_w);
                    push("TDC GFX Limit (A)", 0x018, "u16", self.tdc_gfx, st.tdc_gfx);
                    push("TDC SoC Limit (A)", 0x01A, "u16", self.tdc_soc, st.tdc_soc);
                    push("OD Max GFX Clock (MHz)", 0x027, "u32", self.oc_gfx_clock, st.oc_gfx_clock);
                    push("OD Max Mem Clock (MHz)", 0x02B, "u32", self.oc_mem_clock, st.oc_mem_clock);
                    push("GFX Voltage Min (mV)", 0x03B, "u32", self.volt_gfx, st.volt_gfx);
                    push("GFX Voltage Max (mV)", 0x043, "u32", self.volt_gfx, st.volt_gfx);
                    push("GFX Clock Min (MHz)", 0x33A, "u16", self.freq_gfx, st.freq_gfx);
                    push("GFX Clock Max (MHz)", 0x348, "u16", self.freq_gfx, st.freq_gfx);
                    push("UCLK DPM0 (MHz)", 0x3C4, "u16", self.mem_dpm_freq, st.mem_dpm_freq);
                    push("UCLK DPM3 (MHz)", 0x3CA, "u16", self.mem_dpm_freq, st.mem_dpm_freq);
                    push("Fan Acoustic Limit (RPM)", 0x536, "u16", self.fan_acoustic_limit, st.fan_acoustic_limit);
                    push("Fan Throttling (RPM)", 0x538, "u16", self.fan_throttling_rpm, st.fan_throttling_rpm);
                    push("Fan Max RPM (RPM)", 0x53A, "u16", self.fan_max_rpm, st.fan_max_rpm);
                } else if let Some(v20) = AtomVega20PowerPlayTable::from_bytes_padded(b) {
                    push("OD Max GFX Clock (MHz)", 0x027, "u32", self.oc_gfx_clock, v20.od_settings_max as i32);
                    push("OD Max Mem Clock (MHz)", 0x02B, "u32", self.oc_mem_clock, v20.od_settings_max as i32);
                    push("GFX Voltage Min (mV)", 0x03B, "u32", self.volt_gfx, v20.od_settings_min as i32);
                    push("GFX Voltage Max (mV)", 0x043, "u32", self.volt_gfx, v20.od_settings_max as i32);
                }
            }
            PptArch::Rdna1 => {
                if let Some(r1) = AtomRdna1PowerPlayTable::from_bytes(b) {
                    push("OD Max GFX Clock (MHz)", 0x02C, "u32", self.oc_gfx_clock, r1.od_settings_max as i32);
                    push("OD Max Mem Clock (MHz)", 0x034, "u32", self.oc_mem_clock, r1.od_settings_max as i32);
                    push("GFX Voltage Min (mV)", 0x03C, "u32", self.volt_gfx, r1.od_settings_min as i32);
                    push("GFX Voltage Max (mV)", 0x044, "u32", self.volt_gfx, r1.od_settings_max as i32);
                }
            }
            PptArch::Rdna2 => {
                if let Some(r2) = AtomRdna2PowerPlayTable::from_bytes(b) {
                    push("OD Max GFX Clock (MHz)", 0x030, "u32", self.oc_gfx_clock, r2.od_settings_max as i32);
                    push("OD Max Mem Clock (MHz)", 0x038, "u32", self.oc_mem_clock, r2.od_settings_max as i32);
                    push("GFX Voltage Min (mV)", 0x040, "u32", self.volt_gfx, r2.od_settings_min as i32);
                    push("GFX Voltage Max (mV)", 0x048, "u32", self.volt_gfx, r2.od_settings_max as i32);
                }
            }
            PptArch::Rdna3 => {
                if let Some(r3) = AtomRdna3PowerPlayTable::from_bytes(b) {
                    push("OD Max GFX Clock (MHz)", 0x038, "u32", self.oc_gfx_clock, r3.od_settings_max as i32);
                    push("OD Max Mem Clock (MHz)", 0x040, "u32", self.oc_mem_clock, r3.od_settings_max as i32);
                }
            }
            PptArch::Unknown => {}
        }

        let mismatches = rows.iter().filter(|r| r.ui_val != r.struct_val).count();
        ui.horizontal(|ui| {
            ui.strong(format!("Struct vs UI Parity Audit — {:?} ({} B)", arch, b.len()));
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