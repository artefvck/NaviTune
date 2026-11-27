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
fn run_privileged_in_terminal_or_pkexec(title: &str, raw_cmd: &str) -> Result<bool, String> {
    use std::process::Command;

    let is_root = std::env::var("USER").unwrap_or_default() == "root";
    if is_root {
        let st = Command::new("sh")
            .args(["-c", raw_cmd])
            .status()
            .map_err(|e| format!("Failed to execute root shell: {}", e))?;
        return Ok(st.success());
    }

    let script_path = "/tmp/navitune_priv_task.sh";
    let status_path = "/tmp/navitune_priv_task.rc";
    let _ = std::fs::remove_file(status_path);

    let clean_title = title.replace('\'', "");
    let quoted_cmd = format!("'{}'", raw_cmd.replace('\'', "'\\''"));
    let wrapper = format!(
        "#!/usr/bin/env bash\n\
         echo '============================================================'\n\
         echo '  {title}'\n\
         echo '============================================================'\n\
         echo ''\n\
         sudo bash -c {quoted_cmd}\n\
         RC=$?\n\
         echo \"$RC\" > {status_path}\n\
         echo ''\n\
         if [ \"$RC\" -eq 0 ]; then\n\
           echo '[OK] Operation completed successfully.'\n\
         else\n\
           echo \"[ERROR] Operation failed with exit code $RC.\"\n\
         fi\n\
         read -r -p 'Press Enter to close this window...' _\n\
         exit $RC\n",
        title = clean_title,
        quoted_cmd = quoted_cmd,
        status_path = status_path,
    );
    std::fs::write(script_path, wrapper).map_err(|e| format!("Failed writing helper script: {}", e))?;
    let _ = Command::new("chmod").args(["+x", script_path]).status();

    let candidates: &[(&str, &[&str])] = &[
        ("gnome-terminal", &["--wait", "--title=NaviTune Privilege Task", "--", "bash", script_path]),
        ("konsole", &["--nofork", "-e", "bash", script_path]),
        ("xfce4-terminal", &["--disable-server", "-e", script_path]),
        ("mate-terminal", &["--disable-factory", "-e", script_path]),
        ("lxterminal", &["-e", script_path]),
        ("alacritty", &["-e", "bash", script_path]),
        ("kitty", &["bash", script_path]),
        ("x-terminal-emulator", &["-e", script_path]),
    ];

    for (bin, args) in candidates {
        let found = Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if found {
            let st = Command::new(bin)
                .args(*args)
                .status()
                .map_err(|e| format!("Failed launching {}: {}", bin, e))?;
            if let Ok(rc_txt) = std::fs::read_to_string(status_path) {
                return Ok(rc_txt.trim() == "0");
            }
            return Ok(st.success());
        }
    }

    let st = Command::new("pkexec")
        .args(["bash", "-c", raw_cmd])
        .status()
        .map_err(|e| format!("Failed to execute pkexec: {}", e))?;
    Ok(st.success())
}

#[cfg(target_os = "linux")]
fn is_ppfeaturemask_unlocked() -> bool {
    let Ok(raw) = std::fs::read_to_string("/sys/module/amdgpu/parameters/ppfeaturemask") else {
        return false;
    };
    let raw = raw.trim();
    let parsed = if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        raw.parse::<u64>().ok()
    };
    parsed.map(|m| (m & 0x4000) != 0).unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub(crate) fn install_linux_ppt(ppt_bytes: &[u8], selected_label: &str) -> Result<String, String> {
    use std::process::Command;
    if crate::ppt_arch::PptArch::detect(ppt_bytes) == crate::ppt_arch::PptArch::Unknown {
        return Err("Refusing to install: buffer is not a valid AMD PowerPlayTable".into());
    }

    let card = detect_linux_card(selected_label);
    let sysfs_ppt = format!("/sys/class/drm/{}/device/pp_table", card);
    let tmp_bin = "/tmp/xppt_pp_table.bin";
    std::fs::write(tmp_bin, ppt_bytes).map_err(|e| format!("Failed writing temp file: {}", e))?;

    let unit = format!(
        "[Unit]
Description=Apply NaviTune Soft PowerPlay Table
After=systemd-udev-settle.service
Wants=systemd-udev-settle.service
Before=display-manager.service

[Service]
Type=oneshot
ExecStart=/usr/bin/dd if=/etc/xppt/pp_table.bin of={} bs=65536
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
",
        sysfs_ppt
    );
    let tmp_unit = "/tmp/xppt-pptable.service";
    std::fs::write(tmp_unit, unit).map_err(|e| format!("Failed writing temp unit: {}", e))?;

    let already_unlocked = is_ppfeaturemask_unlocked();
    let grub_snippet = concat!(
        r#"if [ -f /etc/default/grub ] && ! grep -q 'amdgpu\.ppfeaturemask=' /etc/default/grub; then "#,
        r#"cp -n /etc/default/grub /etc/default/grub.navitune.bak 2>/dev/null || true; "#,
        r#"sed -i 's/^GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 amdgpu.ppfeaturemask=0xffffffff"/' /etc/default/grub; "#,
        r#"if command -v update-grub >/dev/null 2>&1; then update-grub; "#,
        r#"elif command -v grub2-mkconfig >/dev/null 2>&1; then grub2-mkconfig -o /boot/grub2/grub.cfg; fi; "#,
        r#"fi"#
    );

    let _ = (&sysfs_ppt, already_unlocked);
    let script = format!(
        "mkdir -p /etc/xppt && install -m 0644 {} /etc/xppt/pp_table.bin && install -m 0644 {} /etc/systemd/system/xppt-pptable.service && systemctl daemon-reload && systemctl enable xppt-pptable.service && {}",
        tmp_bin, tmp_unit, grub_snippet
    );

    let ok = run_privileged_in_terminal_or_pkexec("NaviTune — Install Linux PowerPlayTable & GRUB Mask", &script)?;

    if ok {
        Ok(format!(
            "Installed /etc/xppt/pp_table.bin ({} B) & enabled xppt-pptable.service for {} — REBOOT to apply safely before display manager starts",
            ppt_bytes.len(), card
        ))
    } else {
        Err("Install failed or authentication cancelled".into())
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn uninstall_linux_ppt(selected_label: &str) -> Result<String, String> {
    use std::process::Command;
    let card = detect_linux_card(selected_label);
    let dpm_lvl = format!("/sys/class/drm/{}/device/power_dpm_force_performance_level", card);
    let grub_clean = concat!(
        r#"if [ -f /etc/default/grub ] && grep -q 'amdgpu\.ppfeaturemask=0xffffffff' /etc/default/grub; then "#,
        r#"sed -i 's/ *amdgpu\.ppfeaturemask=0xffffffff//g' /etc/default/grub; "#,
        r#"if command -v update-grub >/dev/null 2>&1; then update-grub; "#,
        r#"elif command -v grub2-mkconfig >/dev/null 2>&1; then grub2-mkconfig -o /boot/grub2/grub.cfg; fi; "#,
        r#"fi"#
    );
    let script = format!(
        "systemctl disable --now xppt-pptable.service 2>/dev/null || true; rm -f /etc/systemd/system/xppt-pptable.service /etc/xppt/pp_table.bin; systemctl daemon-reload; {}; [ -w {} ] && echo auto > {} || true",
        grub_clean, dpm_lvl, dpm_lvl
    );

    let ok = run_privileged_in_terminal_or_pkexec("NaviTune — Uninstall Linux PowerPlayTable & Restore GRUB", &script)?;

    if ok {
        Ok(format!("Uninstalled xppt-pptable.service & cleaned GRUB mask for {} (reboot to fully reload stock VBIOS table)", card))
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
