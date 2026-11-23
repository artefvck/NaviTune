#![allow(unused_imports)]
use super::*;
use crate::export_formats;
use crate::ppt_arch::PptArch;
use crate::vbios;
use crate::PowerPlayApp;


pub(super) fn print_help() {
    println!(
        "NaviTune (pptool) — AMD Radeon PowerPlayTable & VBIOS CLI / GUI\n\n\
         USAGE:\n\
           pptool                                      Launch interactive desktop GUI\n\
           pptool --inspect <file.rom|file.bin>        Inspect PPTable architecture, clocks, voltages & limits\n\
           pptool --export-oc <file> [out.plist]       Export OpenCore DeviceProperties PP,PhmSoftPowerPlayTable\n\
           pptool --export-reg <file> [out.reg]        Export Windows Registry PP_PhmSoftPowerPlayTable .reg\n\
           pptool --export-bin <file.rom> <out.bin>    Extract raw PP_PhmSoftPowerPlayTable binary (Linux sysfs/MPT)\n"
    );
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
        println!("VBIOS Offset : 0x{:06X}", off);
    }
    println!("Architecture : {:?} (rev 0x{:02X}.{:02X}.{:02X}, {} B)", arch, frev, crev, trev, app.raw_buffer.len());
    println!("Status       : {}", app.status_msg);
    println!("GFX Clocks   : {} .. {} MHz (OD Max: {} MHz)", app.freq_gfx[0], app.freq_gfx[1], app.oc_gfx_clock[1]);
    let uclk_peak = app.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
    println!("Memory/UCLK  : DPM0={} MHz .. Peak={} MHz (OD Max: {} MHz)", app.mem_dpm_freq[0][0], uclk_peak, app.oc_mem_clock[1]);
    println!("GFX Voltage  : {} .. {} mV", app.volt_gfx[0], app.volt_gfx[1]);
    println!("Power & TDC  : {} W | TDC GFX={} A, Soc={} A", app.power_limit_w, app.tdc_gfx, app.tdc_soc);
    println!("Fan Targets  : PWM Min={}%, Max={} RPM, Target={}°C, ZeroRPM={}", app.fan_pwm_min, app.fan_max_rpm, app.fan_target_temp, app.fan_zero_rpm_enable);
    Ok(())
}

