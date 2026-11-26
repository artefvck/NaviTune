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
