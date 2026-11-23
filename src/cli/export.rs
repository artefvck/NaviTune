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
