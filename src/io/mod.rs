pub mod export;
pub mod import;
pub mod ppt;
pub mod vbios;

pub(crate) fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let tmp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));
    if let Ok(mut file) = std::fs::File::create(&tmp_path) {
        if file.write_all(data).and_then(|_| file.sync_all()).is_ok() {
            drop(file);
            if std::fs::rename(&tmp_path, path).is_ok() {
                return Ok(());
            }
        }
        let _ = std::fs::remove_file(&tmp_path);
    }
    // Fallback for Windows when target file is open in another app (e.g., MorePowerTool)
    // or Controlled Folder Access blocks temp-file rename (os error 5).
    let mut direct = std::fs::File::create(path)?;
    direct.write_all(data)?;
    direct.sync_all()
}

pub(crate) fn read_bounded(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_INPUT_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File exceeds 64 MiB safety limit ({} bytes)", meta.len()),
        ));
    }
    std::fs::read(path)
}

#[cfg(target_os = "linux")]
fn linux_zenity_or_kdialog_pick(title: &str, filters: &[(&str, &[&str])]) -> Option<Option<std::path::PathBuf>> {
    use std::process::Command;
    if let Ok(out) = Command::new("zenity")
        .arg("--file-selection")
        .arg(format!("--title={}", title))
        .args(filters.iter().map(|(name, exts)| {
            let pat = exts.iter().map(|e| format!("*.{}", e)).collect::<Vec<_>>().join(" ");
            format!("--file-filter={} | {}", name, pat)
        }))
        .arg("--file-filter=All Files | *")
        .output()
    {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Some(if path.is_empty() { None } else { Some(std::path::PathBuf::from(path)) });
        }
        if out.status.code() == Some(1) {
            return Some(None);
        }
    }
    if let Ok(out) = Command::new("kdialog")
        .arg("--getopenfilename")
        .arg(".")
        .arg(format!("--title={}", title))
        .output()
    {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Some(if path.is_empty() { None } else { Some(std::path::PathBuf::from(path)) });
        }
        if out.status.code() == Some(1) {
            return Some(None);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn linux_zenity_or_kdialog_save(title: &str, default_name: Option<&str>, filters: &[(&str, &[&str])]) -> Option<Option<std::path::PathBuf>> {
    use std::process::Command;
    let mut cmd = Command::new("zenity");
    cmd.arg("--file-selection")
        .arg("--save")
        .arg("--confirm-overwrite")
        .arg(format!("--title={}", title));
    if let Some(name) = default_name {
        cmd.arg(format!("--filename={}", name));
    }
    for (name, exts) in filters {
        let pat = exts.iter().map(|e| format!("*.{}", e)).collect::<Vec<_>>().join(" ");
        cmd.arg(format!("--file-filter={} | {}", name, pat));
    }
    cmd.arg("--file-filter=All Files | *");
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Some(if path.is_empty() { None } else { Some(std::path::PathBuf::from(path)) });
        }
        if out.status.code() == Some(1) {
            return Some(None);
        }
    }
    let mut kcmd = Command::new("kdialog");
    kcmd.arg("--getsavefilename")
        .arg(default_name.unwrap_or("."))
        .arg(format!("--title={}", title));
    if let Ok(out) = kcmd.output() {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Some(if path.is_empty() { None } else { Some(std::path::PathBuf::from(path)) });
        }
        if out.status.code() == Some(1) {
            return Some(None);
        }
    }
    None
}

pub fn pick_file_dialog(title: &str, filters: &[(&str, &[&str])]) -> Option<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(res) = linux_zenity_or_kdialog_pick(title, filters) {
        return res;
    }
    let mut dlg = rfd::FileDialog::new().set_title(title);
    for (name, exts) in filters {
        dlg = dlg.add_filter(*name, exts);
    }
    dlg.pick_file()
}

pub fn save_file_dialog(
    title: &str,
    default_name: Option<&str>,
    filters: &[(&str, &[&str])],
) -> Option<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(res) = linux_zenity_or_kdialog_save(title, default_name, filters) {
        return res;
    }
    let mut dlg = rfd::FileDialog::new().set_title(title);
    if let Some(name) = default_name {
        dlg = dlg.set_file_name(name);
    }
    for (name, exts) in filters {
        dlg = dlg.add_filter(*name, exts);
    }
    dlg.save_file()
}
