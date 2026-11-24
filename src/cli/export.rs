#![allow(unused_imports)]
use super::*;
use crate::export_formats;
use crate::ppt_arch::PptArch;
use crate::vbios;
use crate::PowerPlayApp;

const DEFAULT_OC_PCI_PATH: &str = "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)";

pub(super) fn cmd_export_oc(path: &str, out_path: Option<&str>) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let bytes = export_formats::serialize_export_bytes("plist", &app.raw_buffer, None, None, DEFAULT_OC_PCI_PATH)?;
    if let Some(dst) = out_path {
        crate::io::write_atomic(std::path::Path::new(dst), &bytes)
            .map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
        println!("Exported OpenCore PP,PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    } else {
        print!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
}

pub(super) fn cmd_export_reg(path: &str, out_path: Option<&str>) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(path)?;
    app.sync_fields_to_buffer();
    let bytes = export_formats::serialize_export_bytes("reg", &app.raw_buffer, None, None, DEFAULT_OC_PCI_PATH)?;
    if let Some(dst) = out_path {
        crate::io::write_atomic(std::path::Path::new(dst), &bytes)
            .map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
        println!("Exported Windows Registry PP_PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    } else {
        print!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
}

pub(super) fn cmd_export_bin(src: &str, dst: &str) -> Result<(), String> {
    let (mut app, _) = load_app_from_path(src)?;
    app.sync_fields_to_buffer();
    let bytes = export_formats::serialize_export_bytes("bin", &app.raw_buffer, None, None, DEFAULT_OC_PCI_PATH)?;
    crate::io::write_atomic(std::path::Path::new(dst), &bytes)
        .map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
    println!("Extracted PP_PhmSoftPowerPlayTable ({} B) -> {}", app.raw_buffer.len(), dst);
    Ok(())
}

pub(super) fn cmd_preset(
    preset_name: &str,
    path: &str,
    out_rom: Option<&str>,
    out_bin: Option<&str>,
    out_mpt: Option<&str>,
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
    app.sync_fields_to_buffer();
    let mod_bytes = app.modified_byte_count();
    println!("Preset       : {:?}", kind);
    println!("Status       : {}", app.status_msg);
    println!("Modified     : {} byte(s)", mod_bytes);
    println!("GFX Clocks   : {:?} MHz | GFX mV: {:?} mV | Power: {} W | Fan Target: {} C", app.freq_gfx, app.volt_gfx, app.power_limit_w, app.fan_target_temp);

    for (ext, dst_opt, label) in [
        ("rom", out_rom, "Exported ROM"),
        ("bin", out_bin, "Exported BIN"),
        ("mpt", out_mpt, "Exported MPT"),
        ("plist", out_oc, "Exported OC "),
        ("reg", out_reg, "Exported REG"),
    ] {
        if let Some(dst) = dst_opt {
            let bytes = export_formats::serialize_export_bytes(
                ext,
                &app.raw_buffer,
                app.vbios_buffer.as_deref(),
                app.vbios_ppt_offset,
                DEFAULT_OC_PCI_PATH,
            )?;
            crate::io::write_atomic(std::path::Path::new(dst), &bytes)
                .map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
            println!("{} : {} ({} B)", label, dst, bytes.len());
        }
    }
    Ok(())
}
