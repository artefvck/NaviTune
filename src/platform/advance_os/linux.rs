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
