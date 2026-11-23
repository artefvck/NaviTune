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
        "--dump-all" => {
            let Some(path) = args.get(1) else {
                eprintln!("Usage: pptool --dump-all <file.rom|file.bin>");
                std::process::exit(2);
            };
            if let Err(e) = cmd_dump_all(path) {
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


fn load_app_from_path(path: &str) -> Result<(PowerPlayApp, Option<usize>), String> {
    let data = crate::io::read_bounded(std::path::Path::new(&path)).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    let mut app = PowerPlayApp::default();
    if let Ok((ppt, off)) = vbios::extract_ppt_from_rom(&data) {
        app.vbios_ppt_offset = Some(off);
        app.vbios_buffer = Some(data);
        app.apply_ppt_bytes(ppt, path);
        Ok((app, Some(off)))
    } else {
        app.apply_ppt_bytes(data, path);
        Ok((app, None))
    }
}

