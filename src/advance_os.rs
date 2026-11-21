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
        let Ok(out) = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["query", &key, "/v", "MatchingDeviceId"])
            .output()
        else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let txt = String::from_utf8_lossy(&out.stdout).to_ascii_uppercase();
        if txt.contains("VEN_1002") {
            if txt.contains("DEV_73") || txt.contains("DEV_74") {
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

    let out = Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["add", &full_key, "/v", "PP_PhmSoftPowerPlayTable", "/t", "REG_BINARY", "/d", &hex_data, "/f"])
        .output()
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

    let out = Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["delete", &full_key, "/v", "PP_PhmSoftPowerPlayTable", "/f"])
        .output()
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

#[cfg(target_os = "linux")]
pub(crate) fn detect_linux_card(selected_label: &str) -> String {
    selected_label
        .split('(')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .filter(|s| s.starts_with("card") && s[4..].chars().all(|c| c.is_ascii_digit()))
        .unwrap_or("card0")
        .to_string()
}

#[cfg(target_os = "linux")]
pub(crate) fn install_linux_ppt(ppt_bytes: &[u8], selected_label: &str) -> Result<String, String> {
    use std::process::Command;
    let card = detect_linux_card(selected_label);
    let sysfs_ppt = format!("/sys/class/drm/{}/device/pp_table", card);
    let tmp_bin = "/tmp/xppt_pp_table.bin";
    std::fs::write(tmp_bin, ppt_bytes).map_err(|e| format!("Failed writing temp file: {}", e))?;

    let unit = format!(
        "[Unit]\nDescription=Apply NaviTune Soft PowerPlay Table\nAfter=systemd-modules-load.service\n\n[Service]\nType=oneshot\nExecStart=/bin/sh -c 'cat /etc/xppt/pp_table.bin > {}'\nRemainAfterExit=yes\n\n[Install]\nWantedBy=multi-user.target\n",
        sysfs_ppt
    );
    let tmp_unit = "/tmp/xppt-pptable.service";
    std::fs::write(tmp_unit, unit).map_err(|e| format!("Failed writing temp unit: {}", e))?;

    let script = format!(
        "mkdir -p /etc/xppt && cp {} /etc/xppt/pp_table.bin && cp {} /etc/systemd/system/xppt-pptable.service && systemctl daemon-reload && systemctl enable xppt-pptable.service && cat /etc/xppt/pp_table.bin > {} && cmp -s /etc/xppt/pp_table.bin {}",
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

#[cfg(target_os = "macos")]
pub(crate) fn detect_live_acpi_gpu_path(selected_label: &str) -> String {
    use std::process::Command;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some((ref cached_lbl, ref cached_path)) = *guard {
            if cached_lbl == selected_label {
                return cached_path.clone();
            }
        }
    }

    let resolved = (|| -> String {
    let target_pci = selected_label
        .split('/')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let Ok(out) = Command::new("ioreg").args(["-r", "-c", "IOPCIDevice", "-l"]).output() else {
        return "_SB.PCI0.PEG0.PEGP.GFX0".to_string();
    };
    let txt = String::from_utf8_lossy(&out.stdout);
    let mut is_amd = false;
    let mut cur_pci = String::new();
    let mut cur_acpi = String::new();
    let mut first_amd_acpi: Option<String> = None;

    let normalize_acpi = |raw: &str| -> Option<String> {
        let stripped = raw.trim_matches('"').strip_prefix("IOACPIPlane:/")?;
        let parts: Vec<String> = stripped
            .split('/')
            .filter_map(|seg| {
                let name = seg.split('@').next()?.trim();
                if name.is_empty() { None } else { Some(name.to_string()) }
            })
            .collect();
        if parts.is_empty() { None } else { Some(parts.join(".")) }
    };

    for line in txt.lines() {
        let t = line.trim().trim_start_matches('|').trim();
        if t.starts_with("+-o ") {
            if is_amd && !cur_acpi.is_empty() {
                if !target_pci.is_empty() && cur_pci == target_pci {
                    return cur_acpi;
                }
                if first_amd_acpi.is_none() {
                    first_amd_acpi = Some(cur_acpi.clone());
                }
            }
            is_amd = false;
            cur_pci.clear();
            cur_acpi.clear();
        } else if t.starts_with(r#""vendor-id" = <02100000>"#) {
            is_amd = true;
        } else if t.starts_with(r#""pcidebug" = "#) {
            cur_pci = t.trim_start_matches(r#""pcidebug" = "#).trim_matches('"').to_string();
        } else if t.starts_with(r#""acpi-path" = "#) {
            let val = t.trim_start_matches(r#""acpi-path" = "#);
            if let Some(norm) = normalize_acpi(val) {
                cur_acpi = norm;
            }
        }
    }
    if is_amd && !cur_acpi.is_empty() {
        if !target_pci.is_empty() && cur_pci == target_pci {
            return cur_acpi;
        }
        if first_amd_acpi.is_none() {
            first_amd_acpi = Some(cur_acpi);
        }
    }
    first_amd_acpi.unwrap_or_else(|| "_SB.PCI0.PEG0.PEGP.GFX0".to_string())
    })();

    if let Ok(mut guard) = cache.lock() {
        *guard = Some((selected_label.to_string(), resolved.clone()));
    }
    resolved
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn detect_live_acpi_gpu_path(_selected_label: &str) -> String {
    "_SB.PCI0.PEG0.PEGP.GFX0".to_string()
}

pub(crate) fn export_macos_injector_kext(ppt_bytes: &[u8], kext_dir: &std::path::Path) -> Result<String, String> {
    let b64 = crate::export_formats::encode_base64(ppt_bytes);
    let mut wrapped_b64 = String::new();
    for chunk in b64.as_bytes().chunks(68) {
        wrapped_b64.push_str("					");
        wrapped_b64.push_str(&String::from_utf8_lossy(chunk));
        wrapped_b64.push('\n');
    }

    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://apple.com/">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>English</string>
	<key>CFBundleGetInfoString</key>
	<string>RadeonPowerPlayInjector 1.0.0 - NaviTune Soft PowerPlay Table Injector</string>
	<key>CFBundleIdentifier</key>
	<string>com.xppt.driver.RadeonPowerPlayInjector</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>RadeonPowerPlayInjector</string>
	<key>CFBundlePackageType</key>
	<string>KEXT</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0.0</string>
	<key>CFBundleSignature</key>
	<string>????</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
	<key>IOKitPersonalities</key>
	<dict>
		<key>AMDRadeonRDNA2PowerPlay</key>
		<dict>
			<key>Comment</key>
			<string>Merges PP_PhmSoftPowerPlayTable onto AMD IOPCIDevice before com.apple.kext.AMDRadeonX6000 starts</string>
			<key>CFBundleIdentifier</key>
			<string>com.apple.driver.AppleUSBMergeNub</string>
			<key>IOClass</key>
			<string>AppleUSBMergeNub</string>
			<key>IOMatchCategory</key>
			<string>RadeonPowerPlayInjector</string>
			<key>IOPCIMatch</key>
			<string>0x73ff1002 0x73ef1002 0x73df1002 0x73e01002 0x73e11002 0x73e31002 0x73bf1002 0x73af1002 0x743f1002 0x744c1002 0x74701002 0x747e1002 0x74801002 0x74831002</string>
			<key>IOPCITunnelCompatible</key>
			<true/>
			<key>IOProbeScore</key>
			<integer>2000</integer>
			<key>IOProviderClass</key>
			<string>IOPCIDevice</string>
			<key>IOProviderMergeProperties</key>
			<dict>
				<key>PP_DisablePowerContainment</key>
				<integer>1</integer>
				<key>PP_DisableULPS</key>
				<integer>1</integer>
				<key>PP_PhmSoftPowerPlayTable</key>
				<data>
{b64}				</data>
			</dict>
		</dict>
	</dict>
	<key>OSBundleLibraries</key>
	<dict>
		<key>com.apple.kpi.bsd</key>
		<string>15.3</string>
		<key>com.apple.kpi.iokit</key>
		<string>15.3</string>
		<key>com.apple.kpi.libkern</key>
		<string>15.3</string>
		<key>com.apple.kpi.mach</key>
		<string>15.3</string>
		<key>com.apple.kpi.unsupported</key>
		<string>15.3</string>
	</dict>
	<key>OSBundleRequired</key>
	<string>Root</string>
</dict>
</plist>
"#,
        b64 = wrapped_b64
    );

    let contents_dir = kext_dir.join("Contents");
    std::fs::create_dir_all(&contents_dir)
        .map_err(|e| format!("Failed creating {}: {}", contents_dir.display(), e))?;
    let plist_path = contents_dir.join("Info.plist");
    std::fs::write(&plist_path, info_plist)
        .map_err(|e| format!("Failed writing {}: {}", plist_path.display(), e))?;

    Ok(format!(
        "Exported codeless {} ({} B PPTable). Add to EFI/OC/Kexts & config.plist Kernel->Add",
        kext_dir.file_name().unwrap_or_default().to_string_lossy(),
        ppt_bytes.len()
    ))
}

pub(crate) fn compile_dsl_to_aml(dsl_text: &str, out_aml_path: &std::path::Path) -> Result<String, String> {
    use std::process::Command;

    let iasl_bin = ["/usr/local/bin/iasl", "/opt/homebrew/bin/iasl", "iasl"]
        .iter()
        .find(|bin| Command::new(bin).arg("-v").output().is_ok())
        .copied()
        .ok_or_else(|| {
            "iasl binary not found on your system.\n\nInstall ACPICA (e.g. 'brew install acpica' on macOS or place 'iasl' in PATH) to compile SSDT-sPPT.aml.".to_string()
        })?;

    let tmp_dsl = std::env::temp_dir().join("SSDT-sPPT.dsl");
    let tmp_aml = std::env::temp_dir().join("SSDT-sPPT.aml");
    let _ = std::fs::remove_file(&tmp_aml);
    std::fs::write(&tmp_dsl, dsl_text)
        .map_err(|e| format!("Failed writing temporary SSDT-sPPT.dsl: {}", e))?;

    let out = Command::new(iasl_bin)
        .args(["-oa", "-p"])
        .arg(&tmp_aml)
        .arg(&tmp_dsl)
        .output()
        .map_err(|e| format!("Failed executing {}: {}", iasl_bin, e))?;

    if !out.status.success() || !tmp_aml.exists() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(format!("iasl compilation failed:\n{}{}", stdout, stderr));
    }

    let aml_bytes = std::fs::read(&tmp_aml)
        .map_err(|e| format!("Failed reading compiled SSDT-sPPT.aml: {}", e))?;
    std::fs::write(out_aml_path, &aml_bytes)
        .map_err(|e| format!("Failed saving {}: {}", out_aml_path.display(), e))?;

    Ok(format!("Compiled & saved {} ({} bytes) via {}", out_aml_path.file_name().unwrap_or_default().to_string_lossy(), aml_bytes.len(), iasl_bin))
}

pub(crate) fn format_opencore_acpi_dsl(ppt_bytes: &[u8], acpi_path: &str, dis_pc: bool, dis_ulps: bool) -> String {
    let arch_lbl = if ppt_bytes.len() >= 5000 { "RDNA 3" } else { "RDNA 2" };
    let ext_path = if let Some(rest) = acpi_path.strip_prefix("_SB.") {
        format!("_SB_.{}", rest)
    } else {
        acpi_path.to_string()
    };
    let mut hex_rows = String::new();
    for (idx, chunk) in ppt_bytes.chunks(16).enumerate() {
        hex_rows.push_str("                    ");
        for (j, b) in chunk.iter().enumerate() {
            let is_last = idx * 16 + j + 1 == ppt_bytes.len();
            if is_last {
                hex_rows.push_str(&format!("0x{:02X}", b));
            } else {
                hex_rows.push_str(&format!("0x{:02X}, ", b));
            }
        }
        hex_rows.push('\n');
    }

    format!(
        r#"// OpenCore ACPI SSDT Patch for AMD {arch} Soft PowerPlay Table ({len} bytes)
// Compile with: iasl -tc SSDT-sPPT.dsl
// Detected Live ACPI GPU Path: {scope_path}

DefinitionBlock ("", "SSDT", 2, "XPPT", "AMDPPT", 0x00001000)
{{
    External ({ext_path}, DeviceObj)

    Scope ({scope_path})
    {{
        Method (_DSM, 4, NotSerialized)
        {{
            If ((Arg2 == Zero))
            {{
                Return (Buffer (One) {{ 0x03 }})
            }}

            If (_OSI ("Darwin"))
            {{
                Return (Package ()
                {{
                    "PP_DisablePowerContainment", Buffer (0x04) {{ 0x{:02X}, 0x00, 0x00, 0x00 }},
                    "PP_DisableULPS", Buffer (0x04) {{ 0x{:02X}, 0x00, 0x00, 0x00 }},
                    "PP_PhmSoftPowerPlayTable",
                    Buffer (0x{len:04X})
                    {{
{rows}                    }}
                }})
            }}

            Return (Package () {{}})
        }}
    }}
}}
"#,
        u8::from(dis_pc), u8::from(dis_ulps), len = ppt_bytes.len(),
        rows = hex_rows,
        ext_path = ext_path,
        scope_path = acpi_path,
        arch = arch_lbl
    )
}

