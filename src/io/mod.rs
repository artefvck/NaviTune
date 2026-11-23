pub mod export;
pub mod import;
pub mod ppt;
pub mod vbios;

pub(crate) fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let tmp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));
    let mut file = std::fs::File::create(&tmp_path)?;
    if let Err(e) = file.write_all(data).and_then(|_| file.sync_all()) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }
    drop(file);
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }
    Ok(())
}
