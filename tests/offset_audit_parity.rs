
#[test]
fn verify_all_target_roms_cli_roundtrip() {
    let bin = env!("CARGO_BIN_EXE_pptool");
    let mut roms: Vec<std::path::PathBuf> = std::fs::read_dir("target")
        .expect("read target/")
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("rom"))
        .collect();
    roms.sort();
    assert!(!roms.is_empty(), "expected at least one .rom in target/");

    for rom_path in &roms {
        let inspect = std::process::Command::new(bin)
            .args(["--inspect", rom_path.to_str().unwrap()])
            .output()
            .expect("run --inspect");
        assert!(inspect.status.success(), "--inspect failed on {rom_path:?}");
        let stdout = String::from_utf8_lossy(&inspect.stdout);
        assert!(
            !stdout.contains("Architecture : Unknown"),
            "Unknown architecture on {rom_path:?}:
{stdout}"
        );

        let out_bin = std::env::temp_dir().join(format!(
            "navitune_parity_{}.bin",
            rom_path.file_stem().unwrap().to_string_lossy()
        ));
        let export = std::process::Command::new(bin)
            .args(["--export-bin", rom_path.to_str().unwrap(), out_bin.to_str().unwrap()])
            .output()
            .expect("run --export-bin");
        assert!(export.status.success(), "--export-bin failed on {rom_path:?}");

        let rom_bytes = std::fs::read(rom_path).expect("read rom");
        let exported = std::fs::read(&out_bin).expect("read exported bin");
        let _ = std::fs::remove_file(&out_bin);
        assert!(exported.len() >= 512, "exported PPTable too small ({} B) for {rom_path:?}", exported.len());

        let idx = rom_bytes
            .windows(8)
            .position(|w| w == &exported[..8])
            .unwrap_or_else(|| panic!("PPTable header not found in {rom_path:?}"));
        let factory = &rom_bytes[idx..idx + exported.len()];
        assert_eq!(
            factory,
            exported.as_slice(),
            "untouched load->save round-trip modified bytes on {rom_path:?}"
        );
    }
}
