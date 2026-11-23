#![allow(unused_imports)]
use super::*;
use crate::export_formats;
use crate::ppt_arch::PptArch;
use crate::vbios;
use crate::PowerPlayApp;


pub(super) fn cmd_export_oc(path: &str, out_path: Option<&str>) -> Result<(), String> {
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


pub(super) fn cmd_export_reg(path: &str, out_path: Option<&str>) -> Result<(), String> {
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


pub(super) fn cmd_export_bin(src: &str, dst: &str) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(src)?;
    app.sync_fields_to_buffer();
    std::fs::write(dst, &app.raw_buffer).map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
    println!("Extracted PP_PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    Ok(())
}

pub(super) fn cmd_preset(
    preset_name: &str,
    path: &str,
    out_bin: Option<&str>,
    out_oc: Option<&str>,
    out_reg: Option<&str>,
) -> Result<(), String> {
    use crate::ui::safe_presets::SafePresetKind;
    let kind = match preset_name.to_ascii_lowercase().as_str() {
        "quiet" | "undervolt" | "quiet-undervolt" => SafePresetKind::QuietUndervolt,
        "oc" | "balanced" | "balanced-oc" => SafePresetKind::BalancedOc,
        "fix" | "autofix" | "auto-fix" => SafePresetKind::AutoFixRisks,
        other => {
            return Err(format!(
                "Unknown preset '{}'. Expected: quiet-undervolt | balanced-oc | auto-fix",
                other
            ));
        }
    };

    let (mut app, _) = load_app_from_path(path)?;
    app.apply_safe_preset(kind);
    let mod_bytes = app.modified_byte_count();
    println!("Preset       : {:?}", kind);
    println!("Status       : {}", app.status_msg);
    println!("Modified     : {} byte(s)", mod_bytes);
    println!("GFX Clocks   : {:?} MHz | GFX mV: {:?} mV | Power: {} W | Fan Target: {} C", app.freq_gfx, app.volt_gfx, app.power_limit_w, app.fan_target_temp);

    let tmp_bin = std::env::temp_dir().join(format!("navitune_preset_{}.bin", std::process::id()));
    let bin_target = out_bin.unwrap_or_else(|| tmp_bin.to_str().unwrap_or("preset_out.bin"));
    crate::io::write_atomic(std::path::Path::new(bin_target), &app.raw_buffer)
        .map_err(|e| format!("Failed to write '{}': {}", bin_target, e))?;
    if out_bin.is_some() {
        println!("Exported BIN : {} ({} B)", bin_target, app.raw_buffer.len());
    }
    if let Some(dst) = out_oc {
        cmd_export_oc(bin_target, Some(dst))?;
    }
    if let Some(dst) = out_reg {
        cmd_export_reg(bin_target, Some(dst))?;
    }
    if out_bin.is_none() {
        let _ = std::fs::remove_file(&tmp_bin);
    }
    Ok(())
}
