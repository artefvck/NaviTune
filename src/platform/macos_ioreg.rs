use std::process::Command;

#[derive(Clone, Debug)]
pub struct IoregGpuEntry {
    pub label: String,
    pub ppt_bytes: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
fn parse_hex_angle(line: &str) -> Option<Vec<u8>> {
    let s = line.find('<')? + 1;
    let e = line[s..].find('>')? + s;
    let hex_str = &line[s..e];
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .filter_map(|i| hex_str.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
        .collect();
    if bytes.len() >= 64 { Some(bytes) } else { None }
}

#[cfg(target_os = "macos")]
pub fn scan_ioreg_gpus() -> Vec<IoregGpuEntry> {
    let mut results: Vec<IoregGpuEntry> = Vec::new();
    let out = match Command::new("ioreg")
        .args(["-r", "-k", "PP_PhmSoftPowerPlayTable", "-l"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).into_owned(),
        Err(_) => return results,
    };

    let mut cur_node = String::new();
    let mut cur_model = String::new();
    let mut cur_pci = String::new();

    for line in out.lines() {
        let trimmed = line.trim().trim_start_matches('|').trim();
        if trimmed.starts_with("+-o ") {
            let node_name = trimmed
                .trim_start_matches("+-o ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            if !node_name.starts_with("AMD") && !node_name.starts_with("ATY") {
                cur_node = node_name;
                cur_model.clear();
                cur_pci.clear();
            }
        } else if trimmed.starts_with(r#""model" = "#) {
            cur_model = trimmed
                .trim_start_matches(r#""model" = "#)
                .trim_matches(&['<', '>', '"', ' '])
                .to_string();
        } else if trimmed.starts_with(r#""pcidebug" = "#) {
            cur_pci = trimmed
                .trim_start_matches(r#""pcidebug" = "#)
                .trim_matches('"')
                .to_string();
        } else if let Some(pos) = trimmed.find(r#""PP_PhmSoftPowerPlayTable""#) {
            if let Some(bytes) = parse_hex_angle(&trimmed[pos..]) {
                if !results.iter().any(|r| r.ppt_bytes.as_deref() == Some(bytes.as_slice())) {
                    let gpu_name = if cur_model.is_empty() {
                        "AMD Radeon GPU".to_string()
                    } else {
                        cur_model.clone()
                    };
                    let loc = match (cur_node.is_empty(), cur_pci.is_empty()) {
                        (false, false) => format!(" ({} / {})", cur_node, cur_pci),
                        (false, true) => format!(" ({})", cur_node),
                        (true, false) => format!(" ({})", cur_pci),
                        (true, true) => String::new(),
                    };
                    results.push(IoregGpuEntry {
                        label: format!("{}{}", gpu_name, loc),
                        ppt_bytes: Some(bytes),
                    });
                }
            }
        }
    }
    results
}

#[cfg(target_os = "linux")]
pub fn scan_ioreg_gpus() -> Vec<IoregGpuEntry> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        let mut cards: Vec<_> = entries.flatten().collect();
        cards.sort_by_key(|e| e.file_name());
        for entry in cards {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("card") && !name.contains('-') {
                let dev_dir = entry.path().join("device");
                let vendor = std::fs::read_to_string(dev_dir.join("vendor")).unwrap_or_default();
                if vendor.trim().eq_ignore_ascii_case("0x1002") {
                    let ppt_path = dev_dir.join("pp_table");
                    let bytes = std::fs::read(&ppt_path).ok().filter(|b| b.len() >= 64);
                    let uevent = std::fs::read_to_string(dev_dir.join("uevent")).unwrap_or_default();
                    let slot = uevent
                        .lines()
                        .find(|l| l.starts_with("PCI_SLOT_NAME="))
                        .map(|l| l.trim_start_matches("PCI_SLOT_NAME="))
                        .unwrap_or(&name);
                    results.push(IoregGpuEntry {
                        label: format!("AMD Radeon GPU ({} / {})", name, slot),
                        ppt_bytes: bytes,
                    });
                }
            }
        }
    }
    results
}

#[cfg(target_os = "windows")]
pub fn scan_ioreg_gpus() -> Vec<IoregGpuEntry> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut results = Vec::new();
    let base = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    for idx in 0..16 {
        let sub = format!("{:04}", idx);
        let key = format!(r"{}\{}", base, sub);
        let Ok(out) = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["query", &key])
            .output()
        else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let txt = String::from_utf8_lossy(&out.stdout);
        let mut desc = String::new();
        let mut is_amd = false;
        let mut ppt: Option<Vec<u8>> = None;

        for line in txt.lines() {
            let t = line.trim();
            if t.starts_with("DriverDesc") && t.contains("REG_SZ") {
                if let Some((_, val)) = t.split_once("REG_SZ") {
                    desc = val.trim().to_string();
                }
            } else if t.starts_with("MatchingDeviceId") && t.contains("REG_SZ") {
                if t.to_ascii_uppercase().contains("VEN_1002") {
                    is_amd = true;
                }
            } else if t.starts_with("PP_PhmSoftPowerPlayTable") && t.contains("REG_BINARY") {
                if let Some(hex_str) = t.split_whitespace().last() {
                    let bytes: Vec<u8> = (0..hex_str.len())
                        .step_by(2)
                        .filter_map(|i| hex_str.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
                        .collect();
                    if bytes.len() >= 64 {
                        ppt = Some(bytes);
                    }
                }
            }
        }

        let up = desc.to_ascii_uppercase();
        if is_amd || up.contains("AMD") || up.contains("RADEON") {
            let gpu_name = if desc.is_empty() { "AMD Radeon GPU" } else { desc.as_str() };
            results.push(IoregGpuEntry {
                label: format!("{} (Class\\{})", gpu_name, sub),
                ppt_bytes: ppt,
            });
        }
    }
    results
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn scan_ioreg_gpus() -> Vec<IoregGpuEntry> {
    Vec::new()
}
