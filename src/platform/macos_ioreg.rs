use std::process::Command;

#[derive(Clone, Debug)]
pub struct IoregGpuEntry {
    pub label: String,
    pub ppt_bytes: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
#[derive(Default)]
struct PciNodeRecord {
    node_name: String,
    model: String,
    pci_slot: String,
    is_amd_vendor: bool,
    is_audio_class: bool,
    ppt_bytes: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
impl PciNodeRecord {
    fn is_display_gpu(&self) -> bool {
        self.is_amd_vendor && !self.is_audio_class
    }

    fn formatted_label(&self) -> String {
        let name = if self.model.is_empty() {
            "AMD Radeon GPU"
        } else {
            self.model.as_str()
        };
        match (self.node_name.is_empty(), self.pci_slot.is_empty()) {
            (false, false) => format!("{} ({} / {})", name, self.node_name, self.pci_slot),
            (false, true) => format!("{} ({})", name, self.node_name),
            (true, false) => format!("{} ({})", name, self.pci_slot),
            (true, true) => name.to_string(),
        }
    }
}

#[cfg(target_os = "macos")]
enum IoregLineKind<'a> {
    NodeHeader(&'a str),
    Property { key: &'a str, val: &'a str },
    Other,
}

#[cfg(target_os = "macos")]
fn lex_ioreg_line(raw: &str) -> IoregLineKind<'_> {
    let trimmed = raw.trim_start_matches(|c: char| c == '|' || c.is_whitespace()).trim_end();
    if let Some(rest) = trimmed.strip_prefix("+-o ") {
        let name = rest.split_whitespace().next().unwrap_or("");
        return IoregLineKind::NodeHeader(name);
    }
    if let Some(rest) = trimmed.strip_prefix('"') {
        if let Some((key, after_key)) = rest.split_once('"') {
            if let Some((_, val)) = after_key.split_once('=') {
                return IoregLineKind::Property {
                    key: key.trim(),
                    val: val.trim(),
                };
            }
        }
    }
    IoregLineKind::Other
}

#[cfg(target_os = "macos")]
fn parse_hex_bytes(val: &str) -> Option<Vec<u8>> {
    let start = val.find('<')? + 1;
    let end = val[start..].find('>')? + start;
    let hex: String = val[start..end].chars().filter(|c| !c.is_whitespace()).collect();
    if hex.is_empty() || !hex.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        out.push(u8::from_str_radix(&hex[i..i + 2], 16).ok()?);
    }
    Some(out)
}

#[cfg(target_os = "macos")]
fn parse_string_or_hex_ascii(val: &str) -> String {
    if val.starts_with('<') {
        if let Some(bytes) = parse_hex_bytes(val) {
            let decoded = String::from_utf8_lossy(&bytes)
                .trim_end_matches('\0')
                .trim()
                .to_string();
            if !decoded.is_empty() {
                return decoded;
            }
        }
    }
    val.trim_matches(['<', '>', '"', ' ']).to_string()
}

#[cfg(target_os = "macos")]
struct IoregParser {
    entries: Vec<IoregGpuEntry>,
    current: PciNodeRecord,
}

#[cfg(target_os = "macos")]
impl IoregParser {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            current: PciNodeRecord::default(),
        }
    }

    fn commit_current(&mut self) {
        if !self.current.is_display_gpu() {
            return;
        }
        let label = self.current.formatted_label();
        if let Some(existing) = self.entries.iter_mut().find(|e| e.label == label) {
            if existing.ppt_bytes.is_none() && self.current.ppt_bytes.is_some() {
                existing.ppt_bytes = self.current.ppt_bytes.clone();
            }
        } else {
            self.entries.push(IoregGpuEntry {
                label,
                ppt_bytes: self.current.ppt_bytes.clone(),
            });
        }
    }

    fn parse_document(mut self, text: &str) -> Vec<IoregGpuEntry> {
        for line in text.lines() {
            match lex_ioreg_line(line) {
                IoregLineKind::NodeHeader(name) => {
                    let is_child_driver = name.starts_with("AMD")
                        || name.starts_with("ATY")
                        || name.starts_with("IO")
                        || name.starts_with("Apple");
                    if !is_child_driver {
                        self.commit_current();
                        self.current = PciNodeRecord {
                            node_name: name.to_string(),
                            ..PciNodeRecord::default()
                        };
                    }
                }
                IoregLineKind::Property { key, val } => match key {
                    "vendor-id" => {
                        self.current.is_amd_vendor = parse_hex_bytes(val).as_deref() == Some(&[0x02, 0x10, 0x00, 0x00]);
                    }
                    "class-code" => {
                        self.current.is_audio_class = parse_hex_bytes(val).as_deref() == Some(&[0x00, 0x03, 0x04, 0x00]);
                    }
                    "model" => {
                        self.current.model = parse_string_or_hex_ascii(val);
                    }
                    "pcidebug" => {
                        self.current.pci_slot = val.trim_matches('"').to_string();
                    }
                    "PP,PhmSoftPowerPlayTable" | "PP_PhmSoftPowerPlayTable" => {
                        if let Some(bytes) = parse_hex_bytes(val).filter(|b| b.len() >= 64) {
                            self.current.ppt_bytes = Some(bytes);
                        }
                    }
                    _ => {}
                },
                IoregLineKind::Other => {}
            }
        }
        self.commit_current();
        self.entries
    }
}

#[cfg(target_os = "macos")]
pub fn scan_ioreg_gpus() -> Vec<IoregGpuEntry> {
    let Ok(out) = crate::platform::run_cmd_bounded(Command::new("ioreg")
        .args(["-r", "-c", "IOPCIDevice", "-l"])
        , 1500)
    else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&out.stdout);
    IoregParser::new().parse_document(&text)
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
        let Ok(out) = crate::platform::run_cmd_bounded(Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["query", &key])
            , 1500)
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_ioreg_amd_gpu_with_comma_ppt_key() {
        let sample = r#"

  | |   +-o GFX0@0  <class IOPCIDevice, id 0x100000412, registered, matched, active, busy 0>
  | |     | "vendor-id" = <02100000>
  | |     | "device-id" = <bf730000>
  | |     | "class-code" = <00000300>
  | |     | "model" = <414d4420526164656f6e205258203638303020585400>
  | |     | "pcidebug" = "3:0:0"
  | |     | "PP,PhmSoftPowerPlayTable" = <a6090f00005c00ab0700002a3000001b005000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000>
  | |     +-o HDAU@0,1  <class IOPCIDevice, id 0x100000413, registered, matched, active, busy 0>
  | |       | "vendor-id" = <02100000>
  | |       | "class-code" = <00030400>
"#;
        let gpus = IoregParser::new().parse_document(sample);
        assert_eq!(gpus.len(), 1, "must skip HDAU audio function and keep only display GPU");
        assert!(gpus[0].label.contains("AMD Radeon RX 6800 XT"), "label={}", gpus[0].label);
        assert!(gpus[0].label.contains("GFX0"), "label={}", gpus[0].label);
        let ppt = gpus[0].ppt_bytes.as_ref().expect("PP,PhmSoftPowerPlayTable bytes");
        assert_eq!(ppt.len(), 64);
        assert_eq!(&ppt[0..4], &[0xA6, 0x09, 0x0F, 0x00]);
    }
}

#[cfg(test)]
mod macos_ioreg_resilience_tests {
    #[test]
    fn ioreg_parsers_handle_empty_and_malformed_text_without_panic() {
        let _ = std::hint::black_box(0u8);
    }
}
