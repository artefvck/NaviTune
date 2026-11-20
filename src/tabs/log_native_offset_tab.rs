use crate::pptable_rdna2::{AtomSmcDpmInfoV4_9, field_offset_by_name as off};
use crate::pptable_rdna3::{AtomSmcDpmInfoV4_12, field_offset_by_name as off3};
use crate::PowerPlayApp;
use eframe::egui;

pub(crate) fn collect_native_struct_entries(b: &[u8]) -> Option<(&'static str, Vec<(&'static str, &'static str, usize, usize, String)>)> {
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
            ("Fan", "stop / start / target / zero", off3("fan_stop_temp"), 4, format!("Stop={}°C, Start={}°C, Target={}°C, MaxRPM={}, ZeroRPM={}", {t3.fan_stop_temp}, {t3.fan_start_temp}, {t3.fan_target_temp} / 10, {t3.fan_max_rpm}, {t3.fan_zero_rpm_enable})),
        ]));
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
        if self.raw_buffer.is_empty() { ui.label("Load a PowerPlay table first."); return; }
        let b = &self.raw_buffer;
        let mut lines = vec![format!("=== NATIVE RUST OFFSET DEBUG ({} bytes) ===\n", b.len())];
        let arch = if let Some((a, keys)) = collect_native_struct_entries(b) {
            for (_s, name, offset, len, val) in keys {
                let hex: String = b[offset..(offset + len).min(b.len())].iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                lines.push(format!("  @ 0x{:03X} ({:4}) | {:<28} | {:<23} -> {}", offset, offset, name, hex, val));
            }
            a
        } else { lines.push("  [Unsupported buffer size]".into()); "Unknown" };
        let report = lines.join("\n");
        ui.horizontal(|ui| {
            if ui.button("Copy Native Offset Debug").clicked() { ui.ctx().copy_text(report.clone()); }
            ui.colored_label(egui::Color32::from_rgb(150, 165, 190), format!("{} bytes | {}", b.len(), arch));
        });
        ui.add_space(4.0);
        egui::ScrollArea::both().id_salt("native_offset_full_scroll").auto_shrink([false, false]).max_height(ui.available_height()).show(ui, |ui| { ui.monospace(report); });
    }
}
