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

//#[cfg(target_os = "linux")]
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
            use std::process::Stdio;
            let mut cmd = Command::new("setsid");
            cmd.arg("-w")
                .arg(bin)
                .args(*args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let mut child = cmd
                .spawn()
                .map_err(|e| format!("Failed launching {}: {}", bin, e))?;
            let st = child
                .wait()
                .map_err(|e| format!("Failed waiting {}: {}", bin, e))?;
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
pub(crate) fn write_linux_sysfs_ppt(ppt_bytes: &[u8], selected_label: &str, gfx_range: [i32; 2]) -> Result<String, String> {
    if crate::ppt_arch::PptArch::detect(ppt_bytes) == crate::ppt_arch::PptArch::Unknown {
        return Err("Refusing to write: buffer is not a valid AMD PowerPlayTable.".into());
    }
    let card = detect_linux_card(selected_label);
    let sysfs_path = format!("/sys/class/drm/{}/device/pp_table", card);
    if !std::path::Path::new(&sysfs_path).exists() {
        return Err(format!("{} not found (is amdgpu loaded?)", sysfs_path));
    }
    let cur_bytes = std::fs::read(&sysfs_path)
        .map_err(|e| format!("Failed reading {}: {}", sysfs_path, e))?;
    if cur_bytes.len() != ppt_bytes.len() {
        return Err(format!(
            "Size mismatch: {} is {} B, active buffer is {} B.",
            sysfs_path, cur_bytes.len(), ppt_bytes.len()
        ));
    }
    let tmp_bin = "/tmp/navitune_live_pp_table.bin";
    std::fs::write(tmp_bin, ppt_bytes).map_err(|e| format!("Write temp failed: {}", e))?;

    let title = format!("NaviTune — Write PPTable to {}", card);
    let od_path = format!("/sys/class/drm/{}/device/pp_od_clk_voltage", card);
    let od_sync = if gfx_range[0] > 0 && gfx_range[1] >= gfx_range[0] {
        format!(
            " && if [ -w {od} ]; then printf 's 0 %d\n' {g0} > {od} 2>/dev/null || true; printf 's 1 %d\n' {g1} > {od} 2>/dev/null || true; printf 'c\n' > {od} 2>/dev/null || true; fi",
            od = od_path,
            g0 = gfx_range[0],
            g1 = gfx_range[1],
        )
    } else {
        String::new()
    };
    let cmd = format!("cp {tmp} {dst}{od_sync}; RC=$?; rm -f {tmp}; exit $RC", tmp = tmp_bin, dst = sysfs_path, od_sync = od_sync);
    std::thread::spawn(move || {
        let _ = run_privileged_in_terminal_or_pkexec(&title, &cmd);
    });

    Ok(format!("Terminal opened to write {} B PPTable to {} (RAM only — resets on reboot)", ppt_bytes.len(), sysfs_path))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn write_linux_sysfs_ppt(_ppt_bytes: &[u8], _selected_label: &str, _gfx_range: [i32; 2]) -> Result<String, String> {
    Err("Direct sysfs pp_table write is only available on Linux.".into())
}

#[cfg(target_os = "linux")]
pub(crate) fn install_linux_ppt(app: &crate::PowerPlayApp, selected_label: &str) -> Result<String, String> {
    if crate::ppt_arch::PptArch::detect(&app.raw_buffer) == crate::ppt_arch::PptArch::Unknown {
        return Err("Refusing to install: buffer is not a valid AMD PowerPlayTable".into());
    }

    let card = detect_linux_card(selected_label);
    let cfg_text = crate::io_ppt::format_tuning_cfg(app);
    let tmp_cfg = "/tmp/navitune_ppt_override.cfg";
    std::fs::write(tmp_cfg, &cfg_text).map_err(|e| format!("Failed writing temp config: {}", e))?;

    let apply_sh = concat!(
        "#!/usr/bin/env bash\n",
        "set -euo pipefail\n",
        "if grep -Eq '(^| )(noppt|single|recovery|nomodeset)( |$)' /proc/cmdline; then exit 0; fi\n",
        "CFG_FILE=/etc/navitune/ppt_override.cfg\n",
        "[ -f \"$CFG_FILE\" ] || exit 0\n",
        "MASK_FILE=/sys/module/amdgpu/parameters/ppfeaturemask\n",
        "[ -r \"$MASK_FILE\" ] || exit 0\n",
        "MASK=$(cat \"$MASK_FILE\")\n",
        "if (( (MASK & 0x4000) == 0 )); then exit 0; fi\n",
        "# Wait until a real desktop user session (UID >= 1000, not gdm/sddm greeter) is active\n",
        "while true; do\n",
        "  ACTIVE_UID=$(loginctl list-sessions --no-legend 2>/dev/null | awk '$4 != \"gdm\" && $4 != \"sddm\" && $4 != \"lightdm\" && $3 >= 1000 {print $3; exit}')\n",
        "  if [ -n \"$ACTIVE_UID\" ]; then\n",
        "    sleep 3\n",
        "    break\n",
        "  fi\n",
        "  sleep 1\n",
        "done\n",
        "for dev in /sys/class/drm/card[0-9]*/device; do\n",
        "  [ -f \"$dev/vendor\" ] && [ -w \"$dev/pp_table\" ] || continue\n",
        "  grep -qi '^0x1002$' \"$dev/vendor\" || continue\n",
        "  /usr/bin/navitune --apply-cfg \"$CFG_FILE\" --target \"$dev/pp_table\"\n",
        "done\n"
    );
    let tmp_apply = "/tmp/navitune-apply-ppt.sh";
    std::fs::write(tmp_apply, apply_sh).map_err(|e| format!("Failed writing temp apply script: {}", e))?;

    let unit = concat!(
        "[Unit]\n",
        "Description=Patch NaviTune .cfg Tuning onto Live AMD PPTable after Desktop Session Init\n",
        "After=graphical.target\n\n",
        "[Service]\n",
        "Type=oneshot\n",
        "ExecStart=/usr/local/sbin/navitune-apply-ppt.sh\n",
        "RemainAfterExit=yes\n\n",
        "[Install]\n",
        "WantedBy=graphical.target\n"
    );
    let tmp_unit = "/tmp/xppt-pptable.service";
    std::fs::write(tmp_unit, unit).map_err(|e| format!("Failed writing temp unit: {}", e))?;

    let grub_snippet = concat!(
        r#"if [ -f /etc/default/grub ] && ! grep -q 'amdgpu\.ppfeaturemask=' /etc/default/grub; then "#,
        r#"cp -n /etc/default/grub /etc/default/grub.navitune.bak 2>/dev/null || true; "#,
        r#"sed -i 's/^GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 amdgpu.ppfeaturemask=0xffffffff"/' /etc/default/grub; "#,
        r#"if command -v update-grub >/dev/null 2>&1; then update-grub; "#,
        r#"elif command -v grub2-mkconfig >/dev/null 2>&1; then grub2-mkconfig -o /boot/grub2/grub.cfg; fi; "#,
        r#"fi"#
    );

    let script = format!(
        "mkdir -p /etc/navitune /usr/local/sbin && rm -f /etc/udev/rules.d/99-navitune-pptable.rules /etc/xppt/pp_table.bin && install -m 0644 {} /etc/navitune/ppt_override.cfg && install -m 0755 {} /usr/local/sbin/navitune-apply-ppt.sh && install -m 0644 {} /etc/systemd/system/xppt-pptable.service && udevadm control --reload-rules && systemctl daemon-reload && systemctl enable xppt-pptable.service && {}",
        tmp_cfg, tmp_apply, tmp_unit, grub_snippet
    );

    std::thread::spawn(move || {
        let _ = run_privileged_in_terminal_or_pkexec("NaviTune — Install .cfg Override + systemd Service", &script);
    });

    Ok(format!(
        "Terminal opened to install /etc/navitune/ppt_override.cfg for {} (in-place patch after desktop login)",
        card
    ))
}

#[cfg(target_os = "linux")]
pub(crate) fn uninstall_linux_ppt(selected_label: &str) -> Result<String, String> {
    let card = detect_linux_card(selected_label);
    let dpm_lvl = format!("/sys/class/drm/{}/device/power_dpm_force_performance_level", card);
    let script = format!(
        "systemctl disable --now xppt-pptable.service 2>/dev/null || true; rm -rf /etc/systemd/system/xppt-pptable.service /etc/udev/rules.d/99-navitune-pptable.rules /usr/local/sbin/navitune-apply-ppt.sh /etc/navitune /etc/xppt; udevadm control --reload-rules 2>/dev/null || true; systemctl daemon-reload; [ -w {} ] && echo auto > {} || true",
        dpm_lvl, dpm_lvl
    );

    std::thread::spawn(move || {
        let _ = run_privileged_in_terminal_or_pkexec("NaviTune — Uninstall Linux .cfg Override", &script);
    });

    Ok(format!("Terminal opened to uninstall /etc/navitune/ppt_override.cfg for {}", card))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn install_linux_ppt(_app: &crate::PowerPlayApp, _selected_label: &str) -> Result<String, String> {
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
    let cfg_path = "/etc/navitune/ppt_override.cfg";
    let Ok(cfg_txt) = std::fs::read_to_string(cfg_path) else {
        return (
            "No /etc/navitune/ppt_override.cfg".into(),
            "NOT INSTALLED",
            egui::Color32::from_rgb(150, 160, 180),
        );
    };
    let Ok(sysfs_bytes) = std::fs::read(&sysfs_path) else {
        return (
            format!("{} unreadable", sysfs_path),
            "SYSFS MISSING",
            egui::Color32::from_rgb(245, 75, 95),
        );
    };
    let mut live_app = crate::PowerPlayApp::default();
    if live_app.load_bytes_for_path(std::path::Path::new("pp_table.bin"), sysfs_bytes, &sysfs_path).is_err() {
        return ("Failed parsing live sysfs pp_table".into(), "ERROR", egui::Color32::from_rgb(245, 75, 95));
    }
    let mut expected_app = live_app.clone_tunables_snapshot();
    if crate::io_ppt::apply_tuning_cfg_to_app(&mut expected_app, &cfg_txt).is_err() {
        return ("Invalid /etc/navitune/ppt_override.cfg".into(), "CFG ERROR", egui::Color32::from_rgb(245, 75, 95));
    }
    if live_app.freq_gfx == expected_app.freq_gfx
        && live_app.volt_gfx == expected_app.volt_gfx
        && live_app.power_limit_w == expected_app.power_limit_w
        && live_app.fan_target_temp == expected_app.fan_target_temp
    {
        (
            format!("{} matches {} ({}..{} MHz, {} mV)", sysfs_path, cfg_path, live_app.freq_gfx[0], live_app.freq_gfx[1], live_app.volt_gfx[1]),
            "VERIFIED MATCH",
            egui::Color32::from_rgb(80, 210, 120),
        )
    } else {
        (
            format!("sysfs ({:?} MHz, {} mV) != cfg ({:?} MHz, {} mV)", live_app.freq_gfx, live_app.volt_gfx[1], expected_app.freq_gfx, expected_app.volt_gfx[1]),
            "PENDING REBOOT",
            egui::Color32::from_rgb(245, 185, 65),
        )
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn check_linux_ppt_sync(_selected_label: &str) -> (String, &'static str, egui::Color32) {
    (
        "/etc/navitune/ppt_override.cfg".into(),
        "LINUX ONLY",
        egui::Color32::from_rgb(150, 160, 180),
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn read_linux_power_profiles(selected_label: &str) -> (Vec<(u32, String, bool)>, String) {
    let card = detect_linux_card(selected_label);
    let path = format!("/sys/class/drm/{}/device/pp_power_profile_mode", card);
    let Ok(txt) = std::fs::read_to_string(&path) else {
        return (Vec::new(), "pp_power_profile_mode unreadable".into());
    };
    let mut profiles = Vec::new();
    let mut active_summary = "UNKNOWN".to_string();
    for line in txt.lines() {
        let trimmed = line.trim();
        let before_colon = trimmed.split(':').next().unwrap_or(trimmed);
        let mut tokens = before_colon.split_whitespace();
        let Some(idx_tok) = tokens.next() else { continue; };
        let Ok(idx) = idx_tok.parse::<u32>() else { continue; };
        let Some(name_tok) = tokens.next() else { continue; };
        let is_active = trimmed.contains('*');
        let clean_name = name_tok.trim_matches('*').to_string();
        if clean_name.is_empty() { continue; }
        if is_active {
            active_summary = format!("{} ({}) — driver default", clean_name, idx);
        }
        profiles.push((idx, clean_name, is_active));
    }
    (profiles, active_summary)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn read_linux_power_profiles(_selected_label: &str) -> (Vec<(u32, String, bool)>, String) {
    (Vec::new(), "LINUX ONLY".into())
}

#[cfg(target_os = "linux")]
pub(crate) fn set_linux_power_profile(selected_label: &str, profile_idx: u32, profile_name: &str) -> Result<String, String> {
    let card = detect_linux_card(selected_label);
    let dev_dir = format!("/sys/class/drm/{}/device", card);
    let lvl_path = format!("{}/power_dpm_force_performance_level", dev_dir);
    let prof_path = format!("{}/pp_power_profile_mode", dev_dir);
    if !std::path::Path::new(&prof_path).exists() {
        return Err(format!("{} not found", prof_path));
    }
    let cmd = if profile_idx == 0 {
        format!("echo auto > {lvl} && echo 0 > {prof}", lvl = lvl_path, prof = prof_path)
    } else {
        format!("echo manual > {lvl} && echo {idx} > {prof}", lvl = lvl_path, idx = profile_idx, prof = prof_path)
    };
    let title = format!("NaviTune — Set {} Power Profile: {} ({})", card, profile_name, profile_idx);
    std::thread::spawn(move || {
        let _ = run_privileged_in_terminal_or_pkexec(&title, &cmd);
    });
    Ok(format!("Switching {} power profile to {} ({})...", card, profile_name, profile_idx))
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn set_linux_power_profile(_selected_label: &str, _profile_idx: u32, _profile_name: &str) -> Result<String, String> {
    Err("Power profile switching is only available on Linux.".into())
}

