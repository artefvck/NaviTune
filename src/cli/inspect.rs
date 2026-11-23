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
    let (mut app, vbios_off) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let arch = PptArch::detect(&app.raw_buffer);
    let (frev, crev, trev) = (
        app.raw_buffer.get(2).copied().unwrap_or(0),
        app.raw_buffer.get(3).copied().unwrap_or(0),
        app.raw_buffer.get(4).copied().unwrap_or(0),
    );
    println!("File         : {}", path);
    if let Some(off) = vbios_off {
        if let Some(vi) = app.vbios_buffer.as_deref().and_then(|vb| vbios::inspect_vbios_info(vb, off)) {
            println!("VBIOS Info   : Offset 0x{:06X} | DID 0x{:04X} | {}", off, vi.device_id, vi.bios_version);
        } else {
            println!("VBIOS Offset : 0x{:06X}", off);
        }
    }
    println!("Architecture : {:?} (rev 0x{:02X}.{:02X}.{:02X}, {} B)", arch, frev, crev, trev, app.raw_buffer.len());
    println!("Status       : {}", app.status_msg);
    println!("GFX Clocks   : {} .. {} MHz (OD: {} .. {} MHz)", app.freq_gfx[0], app.freq_gfx[1], app.oc_gfx_clock[0], app.oc_gfx_clock[1]);
    println!("SOC / FCLK   : SOC {} .. {} MHz | FCLK {} .. {} MHz (Boost: {} MHz)", app.freq_soc[0], app.freq_soc[1], app.freq_fclk[0], app.freq_fclk[1], app.fclk_boost_freq);
    let uclk_peak = app.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
    println!("Memory/UCLK  : DPM0={} MHz .. Peak={} MHz (OD: {} .. {} MHz)", app.mem_dpm_freq[0][0], uclk_peak, app.oc_mem_clock[0], app.oc_mem_clock[1]);
    println!("GFX / SOC mV : GFX {} .. {} mV (ULV {} mV) | SOC {} .. {} mV", app.volt_gfx[0], app.volt_gfx[1], app.ulv_gfx[0], app.volt_soc[0], app.volt_soc[1]);
    println!("Power & TDC  : {} W (OD: {} .. {}%) | TDC GFX={} A, Soc={} A", app.power_limit_w, app.oc_power_limit[0], app.oc_power_limit[1], app.tdc_gfx, app.tdc_soc);
    println!("Fan Targets  : PWM Min={}%, Max={} RPM, Stop/Start={}/{} C, Target={} C, ZeroRPM={}", app.fan_pwm_min, app.fan_max_rpm, app.fan_stop_temp, app.fan_start_temp, app.fan_target_temp, app.fan_zero_rpm_enable);
    let warns = app.validate_tunables();
    if !warns.is_empty() {
        println!("Warnings     : {}", warns.join("; "));
    }
    Ok(())
}

pub(super) fn cmd_dump_all(path: &str) -> Result<(), String> {
    cmd_inspect(path)?;
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    println!("\n--- OverDrive Feature Flags ---");
    println!("gpu_clock_limit={} gpu_clock_curve={} memory_clock_limit={} zero_rpm={} min_fan_speed={} fan_target_temp={} operating_temp={} fan_acoustic={} fan_curve={} power_limit={} timing_tuning={} power_mode={}", app.gpu_clock_limit, app.gpu_clock_curve, app.memory_clock_limit, app.zero_rpm, app.min_fan_speed, app.fan_target_temp_limit, app.operating_temp_limit, app.fan_acoustic_rpm, app.fan_curve, app.power_limit_feat, app.timing_tuning, app.power_mode);
    println!("auto_uv_gpu={} auto_oc_gpu={} auto_oc_mem={} auto_fan_acoustic={}", app.auto_undervolt_gpu, app.auto_overclock_gpu, app.auto_overclock_mem, app.auto_fan_acoustic_limit);
    let mut smu_mask = 0u64;
    let mut bits = Vec::new();
    for (bit, &en) in app.smu_features.iter().enumerate() {
        if en { smu_mask |= 1u64 << bit; bits.push(bit.to_string()); }
    }
    println!("\n--- SMU Features ---\nMask         : 0x{:016X} ({} enabled)\nEnabled Bits : {}", smu_mask, bits.len(), if bits.is_empty() { "none".to_string() } else { bits.join(", ") });
    println!("\n--- OverDrive Limits & Controls ---\noc_gfx_clock : {:?} MHz | oc_mem_clock : {:?} MHz | oc_power_limit : {:?} %\noc_fan_rpm   : {:?} RPM | oc_acoustic  : {:?} RPM\nControls     : mem_timing={} power_mode={} zero_rpm={}", app.oc_gfx_clock, app.oc_mem_clock, app.oc_power_limit, app.oc_fan_rpm, app.oc_fan_acoustic_rpm, app.oc_mem_timing_ctrl, app.oc_power_mode_ctrl, app.oc_zero_rpm_ctrl);
    println!("\n--- Power, TDC, Thermal & Power Modes ---\npower_limit_w: {} W | power_limit_ac : {:?} W\ntdc_gfx      : {} A | tdc_soc        : {} A\ntemp_limits  : {:?} C\nPM Presets   : Quiet={:?} Balanced={:?} Turbo={:?} Rage={:?}\nAC Acoustic  : Quiet={:?} Balanced={:?} Turbo={:?} Rage={:?}", app.power_limit_w, app.power_limit_ac, app.tdc_gfx, app.tdc_soc, app.temp_limits, app.pm_quiet, app.pm_balanced, app.pm_turbo, app.pm_rage, app.ac_quiet, app.ac_balanced, app.ac_turbo, app.ac_rage);
    println!("\n--- Voltages, ULV, TVmin, AVFS & DCBTC ---\nvolt_gfx     : {:?} mV | volt_soc : {:?} mV\nulv_gfx      : {:?} mV | ulv_soc  : {:?} mV\ntvmin_gfx    : {:?} | tvmin_soc : {:?}\nvmin_lh_gfx  : {:?} mV | vmin_lh_soc : {:?} mV\nStatic V GFX : a={:.6} b={:.6} c={:.6} | Off: a={:.6} b={:.6} c={:.6}\nStatic V SOC : a={:.6} b={:.6} c={:.6} | Off: a={:.6} b={:.6} c={:.6}\nDCBTC        : gfx_en={} soc_en={} min={:?} max={:?} gb={:?} dctol={:?}", app.volt_gfx, app.volt_soc, app.ulv_gfx, app.ulv_soc, app.tvmin_gfx, app.tvmin_soc, app.vmin_lh_gfx, app.vmin_lh_soc, app.static_volt_a[0], app.static_volt_b[0], app.static_volt_c[0], app.static_off_a[0], app.static_off_b[0], app.static_off_c[0], app.static_volt_a[1], app.static_volt_b[1], app.static_volt_c[1], app.static_off_a[1], app.static_off_b[1], app.static_off_c[1], app.dcbtc_gfx_en, app.dcbtc_soc_en, app.dcbtc_min, app.dcbtc_max, app.dcbtc_gb, app.dctol);
    println!("\n--- Clock Domains & PCIe ---\nGFX          : {:?} MHz | SOC  : {:?} MHz | FCLK : {:?} MHz (boost {} MHz)\nVCLK         : {:?} MHz | DCLK : {:?} MHz | LCLK : {:?} MHz\nDCEFCLK      : {:?} MHz | DISP : {:?} MHz | PIX  : {:?} MHz | PHY : {:?} MHz | DTB : {:?} MHz\nPCIe         : Gen {:?} | Lanes {:?}", app.freq_gfx, app.freq_soc, app.freq_fclk, app.fclk_boost_freq, app.freq_vclk, app.freq_dclk, app.lclk_mhz, app.freq_dcefclk, app.freq_dispclk, app.freq_pixclk, app.freq_phyclk, app.freq_dtbclk, app.pcie_gen_speed, app.pcie_lane_count);
    println!("\n--- Memory / UCLK DPM States ---");
    for i in 0..4 {
        println!("DPM{}         : UCLK={} MHz (div={}) | VDDCI={} mV | MVDD={} mV", i, app.mem_dpm_freq[i][0], app.mem_dpm_freq[i][1], app.mem_dpm_volt[i][0], app.mem_dpm_volt[i][1]);
    }
    println!("\n--- Fan Controller & 5-Point Curve ---\nPWM Min={}% | Acoustic={} RPM | Throttling={} RPM | Max={} RPM | Mode={} | Hyst={}\nTarget Temp={} C | Target GFX={} MHz | ZeroRPM={} | Stop={} C | Start={} C\nOD Fan Range : Temp={:?} C | Speed={:?}% | Curve Temp={:?} C | Curve PWM={:?}%", app.fan_pwm_min, app.fan_acoustic_limit, app.fan_throttling_rpm, app.fan_max_rpm, app.fan_mode, app.fan_hyst_ctrl, app.fan_target_temp, app.fan_target_gfx_clock, app.fan_zero_rpm_enable, app.fan_stop_temp, app.fan_start_temp, app.od_fan_temp, app.od_fan_speed, app.fan_curve_temp, app.fan_curve_pwm);
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
