#![allow(unused_imports)]
use super::*;
use eframe::egui;


#[cfg(target_os = "windows")]
fn resolve_windows_amd_subkey(selected_label: &str) -> String {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    if let Some(sub) = selected_label
        .split("Class\\")
        .nth(1)
        .and_then(|x| x.split(")").next())
        .filter(|x| x.len() == 4 && x.chars().all(|c| c.is_ascii_digit()))
    {
        return sub.to_string();
    }

    let base = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let mut first_amd: Option<String> = None;
    for idx in 0..16 {
        let sub = format!("{:04}", idx);
        let key = format!(r"{}\{}", base, sub);
        let Ok(out) = crate::platform::run_cmd_bounded(Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["query", &key, "/v", "MatchingDeviceId"])
            , 1500)
        else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let txt = String::from_utf8_lossy(&out.stdout).to_ascii_uppercase();
        if txt.contains("VEN_1002") {
            if txt.contains("DEV_73") || txt.contains("DEV_74") || txt.contains("DEV_67") || txt.contains("DEV_68") || txt.contains("DEV_66") {
                return sub;
            }
            if first_amd.is_none() {
                first_amd = Some(sub);
            }
        }
    }
    first_amd.unwrap_or_else(|| "0000".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn write_ppt_to_windows_registry(ppt_bytes: &[u8], selected_label: &str) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let base = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let sub = resolve_windows_amd_subkey(selected_label);
    let full_key = format!(r"{}\{}", base, sub);
    let hex_data: String = ppt_bytes.iter().map(|b| format!("{:02X}", b)).collect();

    let out = crate::platform::run_cmd_bounded(Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["add", &full_key, "/v", "PP_PhmSoftPowerPlayTable", "/t", "REG_BINARY", "/d", &hex_data, "/f"])
        , 1500)
        .map_err(|e| format!("Failed to launch reg.exe: {}", e))?;

    if out.status.success() {
        Ok(format!("Wrote {} bytes to {}\\PP_PhmSoftPowerPlayTable (reboot or restart GPU driver to apply)", ppt_bytes.len(), full_key))
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Registry write failed (run NaviTune as Administrator to write to HKLM)".into()
        } else {
            format!("{} (run NaviTune as Administrator to write to HKLM)", err)
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn write_ppt_to_windows_registry(_ppt_bytes: &[u8], _selected_label: &str) -> Result<String, String> {
    Err("Direct regedit write is only available when running on Windows.".into())
}

#[cfg(target_os = "windows")]
pub(crate) fn delete_ppt_from_windows_registry(selected_label: &str) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let base = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let sub = resolve_windows_amd_subkey(selected_label);
    let full_key = format!(r"{}\{}", base, sub);

    let out = crate::platform::run_cmd_bounded(Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["delete", &full_key, "/v", "PP_PhmSoftPowerPlayTable", "/f"])
        , 1500)
        .map_err(|e| format!("Failed to launch reg.exe: {}", e))?;

    if out.status.success() {
        Ok(format!("Deleted {}\\PP_PhmSoftPowerPlayTable (reboot or restart GPU driver to restore stock VBIOS)", full_key))
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Registry delete failed (key may not exist, or run NaviTune as Administrator)".into()
        } else {
            format!("{} (run NaviTune as Administrator)", err)
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn delete_ppt_from_windows_registry(_selected_label: &str) -> Result<String, String> {
    Err("Direct regedit delete is only available when running on Windows.".into())
}

#[cfg(target_os = "windows")]
pub(crate) fn restart_windows_gpu_driver(selected_label: &str) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let base = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let sub = resolve_windows_amd_subkey(selected_label);
    let full_key = format!(r"{}\{}", base, sub);

    let target_match_id = crate::platform::run_cmd_bounded(
        Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["query", &full_key, "/v", "MatchingDeviceId"]),
        1500,
    )
    .ok()
    .and_then(|out| {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find(|l| l.contains("MatchingDeviceId"))
            .and_then(|l| l.split_whitespace().last())
            .map(|s| s.to_ascii_uppercase())
    })
    .unwrap_or_default();

    let enum_out = crate::platform::run_cmd_bounded(
        Command::new("pnputil")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/enum-devices", "/class", "Display", "/connected"]),
        3000,
    )
    .map_err(|e| format!("Failed to launch pnputil.exe: {}", e))?;

    let enum_txt = String::from_utf8_lossy(&enum_out.stdout);
    let mut chosen_instance: Option<String> = None;
    let mut fallback_amd_instance: Option<String> = None;

    for line in enum_txt.lines() {
        let trimmed = line.trim();
        if let Some((_, val)) = trimmed.split_once(':') {
            let id = val.trim();
            let upper = id.to_ascii_uppercase();
            if upper.starts_with(r"PCI\VEN_1002") {
                if !target_match_id.is_empty() && upper.starts_with(&target_match_id) {
                    chosen_instance = Some(id.to_string());
                    break;
                }
                if fallback_amd_instance.is_none() {
                    fallback_amd_instance = Some(id.to_string());
                }
            }
        }
    }

    let instance_id = chosen_instance
        .or(fallback_amd_instance)
        .ok_or_else(|| "No active AMD Display adapter found via pnputil".to_string())?;

    let restart_out = crate::platform::run_cmd_bounded(
        Command::new("pnputil")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/restart-device", &instance_id]),
        8000,
    )
    .map_err(|e| format!("Failed to execute pnputil /restart-device: {}", e))?;

    if restart_out.status.success() {
        Ok(format!("Restarted AMD GPU driver ({})", instance_id))
    } else {
        let stderr = String::from_utf8_lossy(&restart_out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&restart_out.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(if detail.is_empty() {
            "pnputil /restart-device failed (run NaviTune as Administrator)".into()
        } else {
            format!("pnputil /restart-device failed: {}", detail)
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn restart_windows_gpu_driver(_selected_label: &str) -> Result<String, String> {
    Err("GPU driver restart is only available when running on Windows.".into())
}
