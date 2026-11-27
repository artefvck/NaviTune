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

#[derive(Default)]
pub(super) struct PresetExports<'a> {
    pub rom: Option<&'a str>,
    pub bin: Option<&'a str>,
    pub mpt: Option<&'a str>,
    pub oc: Option<&'a str>,
    pub kext: Option<&'a str>,
    pub reg: Option<&'a str>,
}

pub(super) fn cmd_preset(
    preset_name: &str,
    path: &str,
    out: PresetExports<'_>,
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
        ("rom", out.rom, "Exported ROM"),
        ("bin", out.bin, "Exported BIN"),
        ("mpt", out.mpt, "Exported MPT"),
        ("plist", out.oc, "Exported OC "),
        ("reg", out.reg, "Exported REG"),
    ] {
        if let Some(dst) = dst_opt {
            let bytes = export_formats::serialize_export_bytes(
                ext,
                &app.raw_buffer,
                app.vbios_buffer.as_deref(),
                app.vbios_ppt_offset,
                DEFAULT_OC_PCI_PATH,
            )?;
            let mut rom_tag = String::new();
            if ext == "rom" {
                if let (Some(orig_rom), Some(voff)) = (app.vbios_buffer.as_deref(), app.vbios_ppt_offset) {
                    let (blk, csum, diff_cnt) = crate::ui_theme::verify_atombios_rom(orig_rom, &bytes, voff, app.raw_buffer.len())?;
                    rom_tag = format!(" [ROM OK | off 0x{:04X} | csum 0x{:02X} | {} B changed | blk {} B]", voff, csum, diff_cnt, blk);
                }
            }
            let dst_path = std::path::Path::new(dst);
            crate::io::write_atomic(dst_path, &bytes)
                .map_err(|e| format!("Failed to write '{}': {}", dst, e))?;
            if ext == "rom" {
                let disk_bytes = std::fs::read(dst_path)
                    .map_err(|e| format!("Failed to verify '{}' on disk: {}", dst, e))?;
                if disk_bytes != bytes {
                    return Err(format!("ROM read-back verification failed for '{}'", dst));
                }
            }
            println!("{} : {} ({} B){}", label, dst, bytes.len(), rom_tag);
        }
    }
    if let Some(kext_dst) = out.kext {
        let msg = crate::advance_os::export_macos_injector_kext(&app.raw_buffer, std::path::Path::new(kext_dst))?;
        println!("Exported KEXT: {}", msg);
    }
    Ok(())
}

pub(super) fn cmd_apply_cfg(cfg_path: &str, sysfs_target: &str) -> Result<(), String> {
    let cfg_txt = std::fs::read_to_string(cfg_path)
        .map_err(|e| format!("Failed reading config '{}': {}", cfg_path, e))?;
    let target_p = std::path::Path::new(sysfs_target);
    let cur_ppt = std::fs::read(target_p)
        .map_err(|e| format!("Failed reading sysfs '{}': {}", sysfs_target, e))?;
    let mut app = PowerPlayApp::default();
    app.load_bytes_for_path(std::path::Path::new("pp_table.bin"), cur_ppt.clone(), sysfs_target)?;
    let orig_mem_freq = app.mem_dpm_freq;
    let n_keys = crate::io_ppt::apply_tuning_cfg_to_app(&mut app, &cfg_txt)?;
    if app.mem_dpm_freq != orig_mem_freq {
        eprintln!("Warning: preserving native VBIOS mem_dpm_freq {:?} during live sysfs patch to prevent GDDR6 PHY hang", orig_mem_freq);
        app.mem_dpm_freq = orig_mem_freq;
        app.sync_fields_to_buffer();
    }
    if app.raw_buffer.len() != cur_ppt.len() {
        return Err(format!("Patched buffer size mismatch ({} vs {} B)", app.raw_buffer.len(), cur_ppt.len()));
    }
    std::fs::write(target_p, &app.raw_buffer)
        .map_err(|e| format!("Failed writing patched PPTable to '{}': {}", sysfs_target, e))?;
    if let Some(dev_dir) = target_p.parent() {
        let od_path = dev_dir.join("pp_od_clk_voltage");
        if od_path.exists() {
            if app.freq_gfx[0] > 0 && app.freq_gfx[1] >= app.freq_gfx[0] {
                let _ = std::fs::write(&od_path, format!("s 0 {}\n", app.freq_gfx[0]));
                let _ = std::fs::write(&od_path, format!("s 1 {}\n", app.freq_gfx[1]));
            }
            let uclk_peak = app.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
            if uclk_peak > 0 {
                let _ = std::fs::write(&od_path, format!("m 1 {}\n", uclk_peak));
            }
            let _ = std::fs::write(&od_path, "c\n");
        }
    }
    println!("Patched {} ({} B, {} .cfg keys) -> {}", sysfs_target, app.raw_buffer.len(), n_keys, cfg_path);
    Ok(())
}
