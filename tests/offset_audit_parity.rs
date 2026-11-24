
#[test]
fn verify_all_target_roms_cli_roundtrip() {
    let bin = env!("CARGO_BIN_EXE_navitune");
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

        let audit = std::process::Command::new(bin)
            .args(["--audit", rom_path.to_str().unwrap()])
            .output()
            .expect("run --audit");
        assert!(
            audit.status.success(),
            "--audit failed on {rom_path:?}: {}",
            String::from_utf8_lossy(&audit.stderr)
        );
    }
}

#[test]
fn verify_target_stable_mpt_roundtrip_and_audit() {
    let mpt_path = std::path::Path::new("target/stable.mpt");
    if !mpt_path.exists() {
        return;
    }
    let bin = env!("CARGO_BIN_EXE_navitune");
    let inspect = std::process::Command::new(bin)
        .args(["--inspect", mpt_path.to_str().unwrap()])
        .output()
        .expect("run --inspect on stable.mpt");
    assert!(inspect.status.success(), "--inspect failed on target/stable.mpt: {}", String::from_utf8_lossy(&inspect.stderr));
    let stdout = String::from_utf8_lossy(&inspect.stdout);
    assert!(!stdout.contains("Architecture : Unknown"), "Unknown architecture in target/stable.mpt: {stdout}");

    let audit = std::process::Command::new(bin)
        .args(["--audit", mpt_path.to_str().unwrap()])
        .output()
        .expect("run --audit on stable.mpt");
    assert!(audit.status.success(), "--audit failed on target/stable.mpt: {}", String::from_utf8_lossy(&audit.stderr));

    let out_bin = std::env::temp_dir().join(format!("navitune_stable_mpt_{}.bin", std::process::id()));
    let export = std::process::Command::new(bin)
        .args(["--export-bin", mpt_path.to_str().unwrap(), out_bin.to_str().unwrap()])
        .output()
        .expect("run --export-bin on stable.mpt");
    assert!(export.status.success(), "--export-bin failed on target/stable.mpt: {}", String::from_utf8_lossy(&export.stderr));
    let bin_bytes = std::fs::read(&out_bin).expect("read exported bin");
    let _ = std::fs::remove_file(&out_bin);
    assert!(bin_bytes.len() >= 384, "exported PPTable from stable.mpt too small: {} B", bin_bytes.len());
}

#[test]
fn verify_all_import_export_formats_byte_exact_parity() {
    let bin = env!("CARGO_BIN_EXE_navitune");
    let tmp = std::env::temp_dir().join(format!("navitune_io_parity_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp dir");

    let samples: Vec<(&str, usize, u8, Option<u16>)> = vec![
        ("polaris", 833, 0x07, None),
        ("vega10", 512, 0x08, None),
        ("vega20_1534", 1534, 0x0B, None),
        ("vega20_1924", 1924, 0x0B, None),
        ("rdna1_1674", 1674, 0x0C, Some(0x01E2)),
        ("rdna1_1924", 1924, 0x0C, None),
        ("rdna2_2470", 2470, 0x0F, None),
        ("rdna3_5424", 5424, 0x15, None),
    ];

    for (tag, sz, frev, outer) in samples {
        let mut raw = vec![0u8; sz];
        raw[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
        raw[2] = frev;
        raw[4] = if sz == 1924 { 0x01 } else { 0x02 };
        if let Some(osz) = outer {
            raw[5..7].copy_from_slice(&osz.to_le_bytes());
        }
        let src_bin = tmp.join(format!("{tag}.bin"));
        std::fs::write(&src_bin, &raw).expect("write src bin");

        let out_mpt = tmp.join(format!("{tag}.mpt"));
        let out_oc = tmp.join(format!("{tag}.plist"));
        let out_reg = tmp.join(format!("{tag}.reg"));
        let out_kext = tmp.join(format!("{tag}.kext"));

        let st = std::process::Command::new(bin)
            .args([
                "--preset", "auto-fix", src_bin.to_str().unwrap(),
                "--export-bin", src_bin.to_str().unwrap(),
                "--export-mpt", out_mpt.to_str().unwrap(),
                "--export-oc", out_oc.to_str().unwrap(),
                "--export-kext", out_kext.to_str().unwrap(),
                "--export-reg", out_reg.to_str().unwrap(),
            ])
            .status()
            .expect("run --preset export all");
        assert!(st.success(), "export failed for {tag}");
        assert!(out_kext.join("Contents/Info.plist").exists(), "missing Info.plist in .kext for {tag}");

        let expected = std::fs::read(&src_bin).expect("read expected bin");
        for (fmt_name, exported_path) in [("mpt", &out_mpt), ("plist", &out_oc), ("reg", &out_reg)] {
            let re_bin = tmp.join(format!("{tag}_from_{fmt_name}.bin"));
            let st2 = std::process::Command::new(bin)
                .args(["--export-bin", exported_path.to_str().unwrap(), re_bin.to_str().unwrap()])
                .status()
                .expect("re-import format");
            assert!(st2.success(), "re-import failed for {tag} ({fmt_name})");
            let actual = std::fs::read(&re_bin).expect("read re-imported bin");
            assert_eq!(expected, actual, "import/export byte mismatch for {tag} via .{fmt_name}");
        }
    }

    let _ = std::fs::remove_dir_all(&tmp);
}
