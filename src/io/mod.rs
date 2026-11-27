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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDialogAction {
    OpenPowerPlay,
    SavePowerPlay,
    SaveDiffModal,
    PatchVbiosRomModal,
    LinuxImportDonor,
    MacExportPlist,
    MacExportKext,
    MacExportSsdt,
    ExportInspectTxt,
    ExportDebugLog,
}

static PENDING_DIALOG_RESULT: std::sync::Mutex<Option<(FileDialogAction, std::path::PathBuf)>> =
    std::sync::Mutex::new(None);
static DIALOG_IN_FLIGHT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn spawn_pick_file_dialog(
    action: FileDialogAction,
    title: &str,
    filters: &[(&str, &[&str])],
) {
    use std::sync::atomic::Ordering;
    if DIALOG_IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return;
    }
    let title_s = title.to_string();
    let owned_filters: Vec<(String, Vec<String>)> = filters
        .iter()
        .map(|(n, exts)| (n.to_string(), exts.iter().map(|e| e.to_string()).collect()))
        .collect();
    std::thread::spawn(move || {
        let mut dlg = rfd::FileDialog::new().set_title(&title_s);
        for (name, exts) in &owned_filters {
            let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
            dlg = dlg.add_filter(name, &ext_refs);
        }
        if let Some(path) = dlg.pick_file() {
            if let Ok(mut guard) = PENDING_DIALOG_RESULT.lock() {
                *guard = Some((action, path));
            }
        }
        DIALOG_IN_FLIGHT.store(false, Ordering::SeqCst);
    });
}

pub fn spawn_save_file_dialog(
    action: FileDialogAction,
    title: &str,
    default_name: Option<&str>,
    filters: &[(&str, &[&str])],
) {
    use std::sync::atomic::Ordering;
    if DIALOG_IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return;
    }
    let title_s = title.to_string();
    let def_s = default_name.map(|s| s.to_string());
    let owned_filters: Vec<(String, Vec<String>)> = filters
        .iter()
        .map(|(n, exts)| (n.to_string(), exts.iter().map(|e| e.to_string()).collect()))
        .collect();
    std::thread::spawn(move || {
        let mut dlg = rfd::FileDialog::new().set_title(&title_s);
        if let Some(ref name) = def_s {
            dlg = dlg.set_file_name(name);
        }
        for (name, exts) in &owned_filters {
            let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
            dlg = dlg.add_filter(name, &ext_refs);
        }
        if let Some(path) = dlg.save_file() {
            if let Ok(mut guard) = PENDING_DIALOG_RESULT.lock() {
                *guard = Some((action, path));
            }
        }
        DIALOG_IN_FLIGHT.store(false, Ordering::SeqCst);
    });
}

pub fn take_pending_dialog_result() -> Option<(FileDialogAction, std::path::PathBuf)> {
    PENDING_DIALOG_RESULT.lock().ok().and_then(|mut g| g.take())
}

pub fn is_dialog_in_flight() -> bool {
    DIALOG_IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst)
}
