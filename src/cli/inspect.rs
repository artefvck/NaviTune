#![allow(unused_imports)]
use super::*;
use crate::ppt_arch::PptArch;
use crate::vbios;

pub(super) fn print_help() {
    println!("NaviTune (pptool) - AMD Radeon PowerPlayTable & VBIOS CLI / GUI\n  pptool --inspect <file.rom|file.bin>\n  pptool --dump-all <file.rom|file.bin>\n  pptool --export-oc <file> [out.plist]\n  pptool --export-reg <file> [out.reg]\n  pptool --preset <quiet-undervolt|balanced-oc|auto-fix> <file> [--export-bin out.bin] [--export-oc out.plist] [--export-reg out.reg]
  pptool --diff <file_a> <file_b> [--json]
  pptool --export-bin <file.rom> <out.bin>");
}

pub(super) fn cmd_inspect(path: &str) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    print!("{}", app.format_inspect_report(path));
    Ok(())
}

pub(super) fn cmd_dump_all(path: &str) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    print!("{}", app.format_inspect_report(path));
    Ok(())
}

pub(super) fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

pub(super) fn cmd_dump_all_json(path: &str) -> Result<(), String> {
    let (mut app, vbios_off) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let arch = PptArch::detect(&app.raw_buffer);
    let (frev, crev, trev) = (
        app.raw_buffer.get(2).copied().unwrap_or(0),
        app.raw_buffer.get(3).copied().unwrap_or(0),
        app.raw_buffer.get(4).copied().unwrap_or(0),
    );
    let mut smu_mask = 0u64;
    let mut bits = Vec::new();
    for (bit, &en) in app.smu_features.iter().enumerate() {
        if en {
            smu_mask |= 1u64 << bit;
            bits.push(bit.to_string());
        }
    }
    let vb = vbios_off
        .map(|off| off.to_string())
        .unwrap_or_else(|| "null".to_string());
    println!(
        r#"{{
  "file": "{}",
  "vbios_offset": {},
  "architecture": "{:?}",
  "revision": "{:02X}.{:02X}.{:02X}",
  "size_bytes": {},
  "smu_mask_hex": "0x{:016X}",
  "smu_enabled_bits": [{}],
  "gfx_mhz": [{}, {}],
  "soc_mhz": [{}, {}],
  "fclk_mhz": [{}, {}],
  "uclk_dpm_mhz": [{}, {}, {}, {}],
  "volt_gfx_mv": [{}, {}],
  "volt_soc_mv": [{}, {}],
  "power_limit_w": {},
  "tdc_a": [{}, {}],
  "temp_limits_c": {:?},
  "fan": {{ "pwm_min": {}, "max_rpm": {}, "target_temp_c": {}, "zero_rpm": {} }}
}}"#,
        json_esc(path),
        vb,
        arch,
        frev,
        crev,
        trev,
        app.raw_buffer.len(),
        smu_mask,
        bits.join(", "),
        app.freq_gfx[0],
        app.freq_gfx[1],
        app.freq_soc[0],
        app.freq_soc[1],
        app.freq_fclk[0],
        app.freq_fclk[1],
        app.mem_dpm_freq[0][0],
        app.mem_dpm_freq[1][0],
        app.mem_dpm_freq[2][0],
        app.mem_dpm_freq[3][0],
        app.volt_gfx[0],
        app.volt_gfx[1],
        app.volt_soc[0],
        app.volt_soc[1],
        app.power_limit_w,
        app.tdc_gfx,
        app.tdc_soc,
        app.temp_limits,
        app.fan_pwm_min,
        app.fan_max_rpm,
        app.fan_target_temp,
        app.fan_zero_rpm_enable
    );
    Ok(())
}
