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

pub(super) fn cmd_audit(path: &str, as_json: bool) -> Result<(), String> {
    let (mut app, vbios_off) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let (arch, rows) = app.collect_offset_audit_rows();
    let mismatches = rows.iter().filter(|r| r.ui_val != r.struct_val).count();
    let vb_info = app
        .vbios_buffer
        .as_deref()
        .and_then(|rom| vbios::inspect_vbios_info(rom, vbios_off.unwrap_or(0)));

    if as_json {
        let mut rows_json = Vec::with_capacity(rows.len());
        for r in &rows {
            let voff = vbios_off
                .map(|b| format!("\"0x{:06X}\"", b + r.offset))
                .unwrap_or_else(|| "null".to_string());
            rows_json.push(format!(
                r#"{{"field":"{}","ppt_offset":"0x{:04X}","vbios_offset":{},"width":"{}","ui_val":{},"struct_val":{},"match":{}}}"#,
                json_esc(r.field),
                r.offset,
                voff,
                json_esc(r.width),
                r.ui_val,
                r.struct_val,
                r.ui_val == r.struct_val
            ));
        }
        let psp_json = vb_info
            .as_ref()
            .and_then(|v| v.psp_dir_offset)
            .map(|o| format!("\"0x{:X}\"", o))
            .unwrap_or_else(|| "null".to_string());
        let csum_ok = vb_info
            .as_ref()
            .map(|v| !v.has_legacy_opt_rom || v.checksum_sum == 0)
            .unwrap_or(true);
        println!(
            r#"{{
  "file": "{}",
  "architecture": "{:?}",
  "size_bytes": {},
  "vbios_ppt_offset": {},
  "psp_dir_offset": {},
  "rom_checksum_valid": {},
  "audit_fields": {},
  "mismatches": {},
  "rows": [{}]
}}"#,
            json_esc(path),
            arch,
            app.raw_buffer.len(),
            vbios_off.map(|o| format!("\"0x{:X}\"", o)).unwrap_or_else(|| "null".to_string()),
            psp_json,
            csum_ok,
            rows.len(),
            mismatches,
            rows_json.join(", ")
        );
    } else {
        println!("Audit Target : {} ({:?}, {} B)", path, arch, app.raw_buffer.len());
        if let Some(ref vi) = vb_info {
            let psp_str = vi
                .psp_dir_offset
                .map(|o| format!("0x{:X} ({} entries)", o, vi.psp_entry_count))
                .unwrap_or_else(|| "None".to_string());
            let csum_str = if !vi.has_legacy_opt_rom {
                "N/A (SPI Flash)".to_string()
            } else if vi.checksum_sum == 0 {
                format!("VALID (byte[0x21]=0x{:02X})", vi.checksum_byte)
            } else {
                format!("INVALID (byte[0x21]=0x{:02X}, sum=0x{:02X})", vi.checksum_byte, vi.checksum_sum)
            };
            println!("VBIOS Info   : {} [{:04X}:{:04X}] | PPTable @ 0x{:X}", vi.device_name, vi.vendor_id, vi.device_id, vi.ppt_offset);
            println!("$PSP / Csum  : $PSP={} | Checksum={}", psp_str, csum_str);
        }
        println!("Parity Check : {}/{} fields match ({} mismatch(es))", rows.len().saturating_sub(mismatches), rows.len(), mismatches);
        for r in &rows {
            let voff_tag = vbios_off
                .map(|b| format!(" | VBIOS 0x{:06X}", b + r.offset))
                .unwrap_or_default();
            let status = if r.ui_val == r.struct_val { "MATCH" } else { "MISMATCH" };
            println!(
                "  [{:<8}] {:<26} @ 0x{:04X}{} ({}): UI={} vs Struct={}",
                status, r.field, r.offset, voff_tag, r.width, r.ui_val, r.struct_val
            );
        }
    }

    if mismatches > 0 {
        return Err(format!("Offset parity audit detected {} mismatch(es)", mismatches));
    }
    Ok(())
}

#[cfg(test)]
mod cli_audit_tests {
    use super::cmd_audit;

    #[test]
    fn cli_audit_passes_on_rdna1_1674b_and_rdna2_buffers() {
        let mut r1 = vec![0u8; 1674];
        r1[0..2].copy_from_slice(&1674u16.to_le_bytes());
        r1[2] = 0x0C;
        r1[5..7].copy_from_slice(&0x01E2u16.to_le_bytes());
        let p1 = std::env::temp_dir().join(format!("navitune_audit_r1_{}.bin", std::process::id()));
        std::fs::write(&p1, &r1).expect("write r1 temp");
        let res1 = cmd_audit(p1.to_str().unwrap(), false);
        let _ = std::fs::remove_file(&p1);
        assert!(res1.is_ok(), "cmd_audit failed on RDNA1 1674 B: {:?}", res1);

        let mut r2 = vec![0u8; 2470];
        r2[0..2].copy_from_slice(&2470u16.to_le_bytes());
        r2[2] = 0x0F;
        let p2 = std::env::temp_dir().join(format!("navitune_audit_r2_{}.bin", std::process::id()));
        std::fs::write(&p2, &r2).expect("write r2 temp");
        let res2 = cmd_audit(p2.to_str().unwrap(), true);
        let _ = std::fs::remove_file(&p2);
        assert!(res2.is_ok(), "cmd_audit --json failed on RDNA2 2470 B: {:?}", res2);
    }
}
