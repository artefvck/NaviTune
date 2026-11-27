#![allow(unused_imports)]
mod audit;
mod export;
mod inspect;
use audit::*;
use export::*;
use inspect::*;

use crate::export_formats;
use crate::ppt_arch::PptArch;
use crate::vbios;
use crate::PowerPlayApp;


#[cfg(target_os = "windows")]
fn attach_parent_console_if_needed() {
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
    }
    unsafe {
        let _ = AttachConsole(0xFFFF_FFFF);
    }
}

pub fn try_run_cli() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return false;
    }
    #[cfg(target_os = "windows")]
    attach_parent_console_if_needed();
    match args[0].as_str() {
        "-h" | "--help" => {
            print_help();
            true
        }
        "--inspect" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: navitune --inspect <file.rom|file.bin>");
                std::process::exit(2);
            };
            if let Err(e) = cmd_inspect(path) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--dump-all" => {
            let is_json = args.iter().skip(1).any(|a| a == "--json");
            let Some(path) = args.iter().skip(1).find(|a| a.as_str() != "--json") else {
                eprintln!("Usage: navitune --dump-all <file.rom|file.bin> [--json]");
                std::process::exit(2);
            };
            let res = if is_json { cmd_dump_all_json(path) } else { cmd_dump_all(path) };
            if let Err(e) = res {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--export-oc" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: navitune --export-oc <file.rom|file.bin> [out.plist]");
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
                eprintln!("Usage: navitune --export-reg <file.rom|file.bin> [out.reg]");
                std::process::exit(2);
            };
            if let Err(e) = cmd_export_reg(path, args.get(2).map(String::as_str)) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--preset" => {
            let (Some(preset_name), Some(path)) = (args.get(1), args.get(2)) else {
                eprintln!("Usage: navitune --preset <quiet-undervolt|balanced-oc|auto-fix> <file> [--export-rom out.rom] [--export-bin out.bin] [--export-mpt out.mpt] [--export-oc out.plist] [--export-kext out.kext] [--export-reg out.reg]");
                std::process::exit(2);
            };
            let mut out_rom: Option<&str> = None;
            let mut out_bin: Option<&str> = None;
            let mut out_mpt: Option<&str> = None;
            let mut out_oc: Option<&str> = None;
            let mut out_kext: Option<&str> = None;
            let mut out_reg: Option<&str> = None;
            let mut i = 3;
            while i < args.len() {
                match args.get(i).map(String::as_str) {
                    Some("--export-rom")  => { out_rom  = args.get(i + 1).map(String::as_str); i += 2; }
                    Some("--export-bin")  => { out_bin  = args.get(i + 1).map(String::as_str); i += 2; }
                    Some("--export-mpt")  => { out_mpt  = args.get(i + 1).map(String::as_str); i += 2; }
                    Some("--export-oc")   => { out_oc   = args.get(i + 1).map(String::as_str); i += 2; }
                    Some("--export-kext") => { out_kext = args.get(i + 1).map(String::as_str); i += 2; }
                    Some("--export-reg")  => { out_reg  = args.get(i + 1).map(String::as_str); i += 2; }
                    _ => { i += 1; }
                }
            }
            if let Err(e) = cmd_preset(preset_name, path, PresetExports { rom: out_rom, bin: out_bin, mpt: out_mpt, oc: out_oc, kext: out_kext, reg: out_reg }) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--diff" => {
            let is_json = args.iter().skip(1).any(|a| a == "--json");
            let files: Vec<&str> = args.iter().skip(1).map(String::as_str).filter(|&a| a != "--json").collect();
            if files.len() < 2 {
                eprintln!("Usage: navitune --diff <file_a> <file_b> [--json]");
                std::process::exit(2);
            }
            if let Err(e) = cmd_diff(files.first().copied().unwrap_or_default(), files.get(1).copied().unwrap_or_default(), is_json) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--export-bin" => {
            let (Some(src), Some(dst)) = (args.get(1), args.get(2)) else {
                eprintln!("Usage: navitune --export-bin <file.rom> <out.bin>");
                std::process::exit(2);
            };
            if let Err(e) = cmd_export_bin(src, dst) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
                "--apply-cfg" => {
            let Some(cfg_path) = args.get(1) else {
                eprintln!("Usage: navitune --apply-cfg <override.cfg> [--target /sys/class/drm/card1/device/pp_table]");
                std::process::exit(2);
            };
            let target = args.windows(2)
                .find(|w| w[0] == "--target")
                .map(|w| w[1].as_str())
                .unwrap_or("/sys/class/drm/card1/device/pp_table");
            if let Err(e) = cmd_apply_cfg(cfg_path, target) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        "--audit" => {
            let is_json = args.iter().skip(1).any(|a| a == "--json");
            let Some(path) = args.iter().skip(1).find(|a| a.as_str() != "--json") else {
                eprintln!("Usage: navitune --audit <file.rom|file.bin> [--json]");
                std::process::exit(2);
            };
            if let Err(e) = cmd_audit(path, is_json) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            true
        }
        _ => false,
    }
}


fn load_app_from_path(path: &str) -> Result<(PowerPlayApp, Option<usize>), String> {
    let p = std::path::Path::new(&path);
    let data = crate::io::read_bounded(p).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    let mut app = PowerPlayApp::default();
    let off = app.load_bytes_for_path(p, data, path)?;
    Ok((app, off))
}

