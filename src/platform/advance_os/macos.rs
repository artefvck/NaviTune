#![allow(unused_imports)]
use super::*;
use eframe::egui;

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

    let Ok(out) = crate::platform::run_cmd_bounded(Command::new("ioreg").args(["-r", "-c", "IOPCIDevice", "-l"]), 1500) else {
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
			<string>0x67df1002 0x67ef1002 0x699f1002 0x6fdf1002 0x68601002 0x68611002 0x68621002 0x68631002 0x68641002 0x68671002 0x68681002 0x686c1002 0x687f1002 0x66a01002 0x66a11002 0x66a21002 0x66a31002 0x66a71002 0x66af1002 0x73101002 0x73121002 0x731f1002 0x73401002 0x73411002 0x73471002 0x734f1002 0x73601002 0x73621002 0x73ff1002 0x73ef1002 0x73df1002 0x73e01002 0x73e11002 0x73e31002 0x73bf1002 0x73af1002 0x743f1002 0x744c1002 0x74701002 0x747e1002 0x74801002 0x74831002</string>
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
        .find(|bin| crate::platform::run_cmd_bounded(Command::new(bin).arg("-v"), 1500).is_ok())
        .copied()
        .ok_or_else(|| {
            "iasl binary not found on your system.\n\nInstall ACPICA (e.g. 'brew install acpica' on macOS or place 'iasl' in PATH) to compile SSDT-sPPT.aml.".to_string()
        })?;

    let tmp_dsl = std::env::temp_dir().join("SSDT-sPPT.dsl");
    let tmp_aml = std::env::temp_dir().join("SSDT-sPPT.aml");
    let _ = std::fs::remove_file(&tmp_aml);
    std::fs::write(&tmp_dsl, dsl_text)
        .map_err(|e| format!("Failed writing temporary SSDT-sPPT.dsl: {}", e))?;

    let out = crate::platform::run_cmd_bounded(Command::new(iasl_bin)
        .args(["-oa", "-p"])
        .arg(&tmp_aml)
        .arg(&tmp_dsl)
        , 1500)
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
    let arch_lbl = crate::ppt_arch::PptArch::detect(ppt_bytes).label();
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


#[cfg(test)]
mod advance_os_export_tests {
    use super::{export_macos_injector_kext, format_opencore_acpi_dsl};

    #[test]
    fn macos_kext_and_acpi_dsl_support_rdna1_1674b() {
        let mut ppt = vec![0u8; 1674];
        ppt[0..2].copy_from_slice(&1674u16.to_le_bytes());
        ppt[2] = 0x0C;
        ppt[5..7].copy_from_slice(&0x01E2u16.to_le_bytes());

        let dsl = format_opencore_acpi_dsl(&ppt, "_SB.PCI0.PEG0.PEGP.GFX0", true, true);
        assert!(dsl.contains("RDNA 1"));
        assert!(dsl.contains("Buffer (0x068A)"));
        assert!(dsl.contains("External (_SB_.PCI0.PEG0.PEGP.GFX0, DeviceObj)"));

        let kext_dir = std::env::temp_dir().join(format!("NaviTuneTest_{}.kext", std::process::id()));
        let _ = std::fs::remove_dir_all(&kext_dir);
        let res = export_macos_injector_kext(&ppt, &kext_dir);
        assert!(res.is_ok(), "kext export failed: {:?}", res);
        let plist = std::fs::read_to_string(kext_dir.join("Contents/Info.plist")).expect("read Info.plist");
        let _ = std::fs::remove_dir_all(&kext_dir);
        assert!(plist.contains("0x73411002"));
        assert!(plist.contains("0x73401002"));
    }
}
