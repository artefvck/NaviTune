#![allow(unused_imports)]
use super::*;
use eframe::egui;

#[cfg(target_os = "linux")]
pub(crate) fn check_linux_ppfeaturemask() -> (String, &'static str, egui::Color32) {
    let raw = match std::fs::read_to_string("/sys/module/amdgpu/parameters/ppfeaturemask") {
        Ok(v) => v.trim().to_string(),
        Err(_) => {
            return (
                "amdgpu module param unreadable".into(),
                "UNKNOWN",
                egui::Color32::from_rgb(150, 160, 180),
            );
        }
    };
    let parsed = if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        raw.parse::<u64>().ok()
    };
    match parsed {
        Some(mask) if (mask & 0x4000) != 0 => (
            format!("0x{:08X} (OverDrive bit 0x4000 ON)", mask),
            "UNLOCKED",
            egui::Color32::from_rgb(80, 210, 120),
        ),
        Some(mask) => (
            format!("0x{:08X} (need amdgpu.ppfeaturemask=0xffffffff)", mask),
            "LOCKED",
            egui::Color32::from_rgb(245, 75, 95),
        ),
        None => (
            raw,
            "UNKNOWN",
            egui::Color32::from_rgb(150, 160, 180),
        ),
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn check_linux_ppfeaturemask() -> (String, &'static str, egui::Color32) {
    (
        "/sys/module/amdgpu/parameters/ppfeaturemask".into(),
        "LINUX ONLY",
        egui::Color32::from_rgb(150, 160, 180),
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn check_linux_systemd_service() -> (String, &'static str, egui::Color32) {
    let unit_exists = std::path::Path::new("/etc/systemd/system/xppt-pptable.service").exists();
    if !unit_exists {
        return (
            "xppt-pptable.service (not installed)".into(),
            "NOT INSTALLED",
            egui::Color32::from_rgb(150, 160, 180),
        );
    }
    let enabled = std::process::Command::new("systemctl")
        .args(["is-enabled", "xppt-pptable.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if enabled {
        (
            "xppt-pptable.service (boot persistence ON)".into(),
            "ENABLED",
            egui::Color32::from_rgb(80, 210, 120),
        )
    } else {
        (
            "xppt-pptable.service (disabled)".into(),
            "DISABLED",
            egui::Color32::from_rgb(245, 185, 65),
        )
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn check_linux_systemd_service() -> (String, &'static str, egui::Color32) {
    (
        "xppt-pptable.service".into(),
        "LINUX ONLY",
        egui::Color32::from_rgb(150, 160, 180),
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn detect_linux_card(selected_label: &str) -> String {
    for tok in selected_label.split(|c: char| !c.is_ascii_alphanumeric()) {
        if tok.starts_with("card") && tok.len() > 4 && tok[4..].chars().all(|c| c.is_ascii_digit()) {
            return tok.to_string();
        }
    }
    if let Ok(rd) = std::fs::read_dir("/sys/class/drm") {
        let mut cards: Vec<String> = rd
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.starts_with("card") && n.len() > 4 && n[4..].chars().all(|c| c.is_ascii_digit()))
            .filter(|n| std::path::Path::new(&format!("/sys/class/drm/{}/device/pp_table", n)).exists())
            .collect();
        cards.sort();
        if let Some(first_amd) = cards.into_iter().find(|n| {
            std::fs::read_to_string(format!("/sys/class/drm/{}/device/vendor", n))
                .map(|v| v.trim().eq_ignore_ascii_case("0x1002"))
                .unwrap_or(false)
        }) {
            return first_amd;
        }
    }
    "card0".to_string()
}

#[cfg(target_os = "linux")]
pub(crate) fn install_linux_ppt(ppt_bytes: &[u8], selected_label: &str) -> Result<String, String> {
    use std::process::Command;
    let card = detect_linux_card(selected_label);
    let sysfs_ppt = format!("/sys/class/drm/{}/device/pp_table", card);
    let tmp_bin = "/tmp/xppt_pp_table.bin";
    std::fs::write(tmp_bin, ppt_bytes).map_err(|e| format!("Failed writing temp file: {}", e))?;

    if crate::ppt_arch::PptArch::detect(ppt_bytes) == crate::ppt_arch::PptArch::Unknown {
        return Err("Refusing to install: buffer is not a valid AMD PowerPlayTable".into());
    }

    let unit = format!(
        "[Unit]\nDescription=Apply NaviTune Soft PowerPlay Table\nAfter=systemd-udev-settle.service\nWants=systemd-udev-settle.service\nBefore=display-manager.service\n\n[Service]\nType=oneshot\nExecStart=/usr/bin/dd if=/etc/xppt/pp_table.bin of={} bs=65536\nRemainAfterExit=yes\n\n[Install]\nWantedBy=multi-user.target\n",
        sysfs_ppt
    );
    let tmp_unit = "/tmp/xppt-pptable.service";
    std::fs::write(tmp_unit, unit).map_err(|e| format!("Failed writing temp unit: {}", e))?;

    let script = format!(
        "mkdir -p /etc/xppt && install -m 0644 {} /etc/xppt/pp_table.bin && install -m 0644 {} /etc/systemd/system/xppt-pptable.service && systemctl daemon-reload && systemctl enable xppt-pptable.service && /usr/bin/dd if=/etc/xppt/pp_table.bin of={} bs=65536 && cmp -s /etc/xppt/pp_table.bin {}",
        tmp_bin, tmp_unit, sysfs_ppt, sysfs_ppt
    );

    let is_root = std::fs::metadata("/root").map(|_| true).unwrap_or(false) && std::env::var("USER").unwrap_or_default() == "root";
    let status = if is_root {
        Command::new("sh").args(["-c", &script]).status()
    } else {
        Command::new("pkexec").args(["sh", "-c", &script]).status()
    }.map_err(|e| format!("Failed to execute privilege helper: {}", e))?;

    if status.success() {
        Ok(format!("Installed & applied PP_PhmSoftPowerPlayTable ({} B) to {} + systemd service", ppt_bytes.len(), sysfs_ppt))
    } else {
        Err("Install failed (ensure amdgpu.ppfeaturemask=0xffffffff is set and authentication succeeded)".into())
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn uninstall_linux_ppt(selected_label: &str) -> Result<String, String> {
    use std::process::Command;
    let card = detect_linux_card(selected_label);
    let dpm_lvl = format!("/sys/class/drm/{}/device/power_dpm_force_performance_level", card);
    let script = format!(
        "systemctl disable --now xppt-pptable.service 2>/dev/null || true; rm -f /etc/systemd/system/xppt-pptable.service /etc/xppt/pp_table.bin; systemctl daemon-reload; [ -w {} ] && echo auto > {} || true",
        dpm_lvl, dpm_lvl
    );

    let is_root = std::env::var("USER").unwrap_or_default() == "root";
    let status = if is_root {
        Command::new("sh").args(["-c", &script]).status()
    } else {
        Command::new("pkexec").args(["sh", "-c", &script]).status()
    }.map_err(|e| format!("Failed to execute privilege helper: {}", e))?;

    if status.success() {
        Ok(format!("Uninstalled xppt-pptable.service & reverted {} (reboot to fully reload stock VBIOS table)", card))
    } else {
        Err("Uninstall failed or authentication cancelled".into())
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn install_linux_ppt(_ppt_bytes: &[u8], _selected_label: &str) -> Result<String, String> {
    Err("Linux sysfs/systemd install is only available when running on Linux.".into())
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn uninstall_linux_ppt(_selected_label: &str) -> Result<String, String> {
    Err("Linux sysfs/systemd uninstall is only available when running on Linux.".into())
}

#[cfg(target_os = "linux")]
pub(crate) fn check_linux_ppt_sync(selected_label: &str) -> (String, &'static str, egui::Color32) {
    let card = detect_linux_card(selected_label);
    let sysfs_path = format!("/sys/class/drm/{}/device/pp_table", card);
    let etc_path = "/etc/xppt/pp_table.bin";
    let etc_bytes = std::fs::read(etc_path).ok();
    let sysfs_bytes = std::fs::read(&sysfs_path).ok();

    match (etc_bytes, sysfs_bytes) {
        (None, _) => (
            "No /etc/xppt/pp_table.bin".into(),
            "NOT INSTALLED",
            egui::Color32::from_rgb(150, 160, 180),
        ),
        (Some(_), None) => (
            format!("{} unreadable", sysfs_path),
            "SYSFS MISSING",
            egui::Color32::from_rgb(245, 75, 95),
        ),
        (Some(eb), Some(sb)) if eb == sb => (
            format!("{} == {} ({} B)", sysfs_path, etc_path, sb.len()),
            "VERIFIED MATCH",
            egui::Color32::from_rgb(80, 210, 120),
        ),
        (Some(eb), Some(sb)) => (
            format!("sysfs={} B vs etc={} B", sb.len(), eb.len()),
            "MISMATCH (NOT APPLIED)",
            egui::Color32::from_rgb(245, 75, 95),
        ),
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn check_linux_ppt_sync(_selected_label: &str) -> (String, &'static str, egui::Color32) {
    (
        "/etc/xppt/pp_table.bin".into(),
        "LINUX ONLY",
        egui::Color32::from_rgb(150, 160, 180),
    )
}
