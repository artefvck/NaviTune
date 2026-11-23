#![allow(unused_imports)]
use super::*;

pub(super) fn cmd_diff(path_a: &str, path_b: &str, as_json: bool) -> Result<(), String> {
    let (mut app_a, off_a) = load_app_from_path(path_a)?;
    let (mut app_b, off_b) = load_app_from_path(path_b)?;
    app_a.sync_fields_to_buffer();
    app_b.sync_fields_to_buffer();

    let (entries, total_bytes) = crate::ui::save_diff_modal::collect_bounded_byte_diffs(
        &app_a.raw_buffer,
        &app_b.raw_buffer,
        64,
    );

    let mut mask_a = 0u64;
    let mut mask_b = 0u64;
    for (bit, (&ea, &eb)) in app_a.smu_features.iter().zip(app_b.smu_features.iter()).enumerate() {
        if ea { mask_a |= 1u64 << bit; }
        if eb { mask_b |= 1u64 << bit; }
    }

    if as_json {
        let mut diffs_json = Vec::new();
        for (off, old_b, new_b) in &entries {
            let o_s = old_b.map(|v| format!(r#""0x{:02X}""#, v)).unwrap_or_else(|| "null".to_string());
            let n_s = new_b.map(|v| format!(r#""0x{:02X}""#, v)).unwrap_or_else(|| "null".to_string());
            diffs_json.push(format!(r#"{{"offset":{},"a":{},"b":{}}}"#, off, o_s, n_s));
        }
        println!(
            r#"{{
  "file_a": "{}",
  "file_b": "{}",
  "vbios_offsets": [{}, {}],
  "sizes": [{}, {}],
  "modified_bytes": {},
  "smu_masks": ["0x{:016X}", "0x{:016X}"],
  "gfx_mhz": [{:?}, {:?}],
  "volt_gfx_mv": [{:?}, {:?}],
  "power_limit_w": [{}, {}],
  "tdc_gfx_a": [{}, {}],
  "fan_target_temp_c": [{}, {}],
  "byte_preview": [{}]
}}"#,
            json_esc(path_a),
            json_esc(path_b),
            off_a.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
            off_b.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
            app_a.raw_buffer.len(),
            app_b.raw_buffer.len(),
            total_bytes,
            mask_a,
            mask_b,
            app_a.freq_gfx, app_b.freq_gfx,
            app_a.volt_gfx, app_b.volt_gfx,
            app_a.power_limit_w, app_b.power_limit_w,
            app_a.tdc_gfx, app_b.tdc_gfx,
            app_a.fan_target_temp, app_b.fan_target_temp,
            diffs_json.join(", ")
        );
        return Ok(());
    }

    println!("Diff A       : {} ({} B)", path_a, app_a.raw_buffer.len());
    println!("Diff B       : {} ({} B)", path_b, app_b.raw_buffer.len());
    println!("Byte Deltas  : {} modified byte(s)", total_bytes);
    println!("GFX Clocks   : {:?} MHz -> {:?} MHz", app_a.freq_gfx, app_b.freq_gfx);
    println!("GFX Voltage  : {:?} mV -> {:?} mV", app_a.volt_gfx, app_b.volt_gfx);
    println!("Power & TDC  : {} W / {} A -> {} W / {} A", app_a.power_limit_w, app_a.tdc_gfx, app_b.power_limit_w, app_b.tdc_gfx);
    println!("Fan Targets  : {} C / {} RPM / ZeroRPM={} -> {} C / {} RPM / ZeroRPM={}", app_a.fan_target_temp, app_a.fan_max_rpm, app_a.fan_zero_rpm_enable, app_b.fan_target_temp, app_b.fan_max_rpm, app_b.fan_zero_rpm_enable);
    println!("SMU Mask     : 0x{:016X} -> 0x{:016X}", mask_a, mask_b);
    if !entries.is_empty() {
        println!("
--- Byte-Level Diff (first {} of {}) ---", entries.len(), total_bytes);
        for (off, old_b, new_b) in &entries {
            let o_s = old_b.map(|v| format!("0x{:02X}", v)).unwrap_or_else(|| "EOF".to_string());
            let n_s = new_b.map(|v| format!("0x{:02X}", v)).unwrap_or_else(|| "EOF".to_string());
            println!("  +0x{:04X}: {} -> {}", off, o_s, n_s);
        }
    }
    Ok(())
}
