use crate::export_formats;
use crate::ppt_arch::PptArch;
use crate::vbios;
use crate::PowerPlayApp;

pub fn try_run_cli() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return false;
    }
    match args[0].as_str() {
        "-h" | "--help" => {
            print_help();
            true
        }
        "--inspect" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: pptool --inspect <file.rom|file.bin>");
                std::process::exit(2);
            };
            if let Err(e) = cmd_inspect(path) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--export-oc" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: pptool --export-oc <file.rom|file.bin> [out.plist]");
                std::process::exit(2);
            };
            if let Err(e) = cmd_export_oc(path, args.get(2).map(String::as_str)) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--export-reg" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: pptool --export-reg <file.rom|file.bin> [out.reg]");
                std::process::exit(2);
            };
            if let Err(e) = cmd_export_reg(path, args.get(2).map(String::as_str)) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--export-bin" => {
            let (Some(src), Some(dst)) = (args.get(1), args.get(2)) else {
                eprintln!("Usage: pptool --export-bin <file.rom> <out.bin>");
                std::process::exit(2);
            };
            if let Err(e) = cmd_export_bin(src, dst) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        _ => false,
    }
}

fn print_help() {
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

fn load_app_from_path(path: &str) -> Result<(PowerPlayApp, Option<usize>), String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    let mut app = PowerPlayApp::default();
    if let Ok((ppt, off)) = vbios::extract_ppt_from_rom(&data) {
        app.vbios_ppt_offset = Some(off);
        app.vbios_buffer = Some(data);
        app.apply_loaded_buffer(ppt, path);
        Ok((app, Some(off)))
    } else {
        app.apply_loaded_buffer(data, path);
        Ok((app, None))
    }
}

fn cmd_inspect(path: &str) -> Result<(), String> {
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
    println!("Architecture : {} (rev 0x{:02X}.{:02X}.{:02X}, {} B)", format!("{:?}", arch), frev, crev, trev, app.raw_buffer.len());
    println!("Status       : {}", app.status_msg);
    println!("GFX Clocks   : {} .. {} MHz (OD Max: {} MHz)", app.freq_gfx[0], app.freq_gfx[1], app.oc_gfx_clock[1]);
    let uclk_peak = app.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
    println!("Memory/UCLK  : DPM0={} MHz .. Peak={} MHz (OD Max: {} MHz)", app.mem_dpm_freq[0][0], uclk_peak, app.oc_mem_clock[1]);
    println!("GFX Voltage  : {} .. {} mV", app.volt_gfx[0], app.volt_gfx[1]);
    println!("Power & TDC  : {} W | TDC GFX={} A, Soc={} A", app.power_limit_w, app.tdc_gfx, app.tdc_soc);
    println!("Fan Targets  : PWM Min={}%, Max={} RPM, Target={}°C, ZeroRPM={}", app.fan_pwm_min, app.fan_max_rpm, app.fan_target_temp, app.fan_zero_rpm_enable);
    Ok(())
}

fn cmd_export_oc(path: &str, out_path: Option<&str>) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let b64 = export_formats::encode_base64(&app.raw_buffer);
    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://apple.com\">\n\
         <plist version=\"1.0\">\n<dict>\n\
           <key>PP,PhmSoftPowerPlayTable</key>\n\
           <data>{}</data>\n\
         </dict>\n</plist>\n",
        b64
    );
    if let Some(dst) = out_path {
        std::fs::write(dst, plist.as_bytes()).map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
        println!("Exported OpenCore PP,PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    } else {
        print!("{}", plist);
    }
    Ok(())
}

fn cmd_export_reg(path: &str, out_path: Option<&str>) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let hex_bytes: Vec<String> = app.raw_buffer.iter().map(|b| format!("{:02x}", b)).collect();
    let reg = format!(
        "Windows Registry Editor Version 5.00\r\n\r\n\
         [HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Class\\{{4d36e968-e325-11ce-bfc1-08002be10318}}\\0000]\r\n\
         \"PP_PhmSoftPowerPlayTable\"=hex:{}\r\n",
        hex_bytes.join(",")
    );
    if let Some(dst) = out_path {
        std::fs::write(dst, reg.as_bytes()).map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
        println!("Exported Windows Registry PP_PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    } else {
        print!("{}", reg);
    }
    Ok(())
}

fn cmd_export_bin(src: &str, dst: &str) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(src)?;
    app.sync_fields_to_buffer();
    std::fs::write(dst, &app.raw_buffer).map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
    println!("Extracted PP_PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    Ok(())
}
