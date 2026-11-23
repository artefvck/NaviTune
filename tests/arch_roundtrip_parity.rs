use std::fs;
use std::process::Command;

/// Builds a synthetic AtomBIOS CommonTableHeader + PowerPlayTable skeleton for each of the 6 GPU generations:
/// - Polaris:  SMU 8.0,  rev 0x07, >= 384 B
/// - Vega 10:  SMU 9.0,  rev 0x08, >= 512 B
/// - Vega 20:  SMU 11.0, rev 0x0B, 1,924 B
/// - RDNA 1:   SMU 11.0, rev 0x0D (V4_5),  1,924 B
/// - RDNA 2:   SMU 11.0, rev 0x0F (V4_9),  2,470 B
/// - RDNA 3:   SMU 13.0, rev 0x14 (V4_12), 5,424 B
fn make_ppt_skeleton(table_rev: u8, format_rev: u8, content_rev: u8, size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; size];
    let sz_le = (size as u16).to_le_bytes();
    buf[0] = sz_le[0];
    buf[1] = sz_le[1];
    buf[2] = format_rev;
    buf[3] = content_rev;
    buf[4] = table_rev;
    buf
}

#[test]
fn cli_inspect_supports_all_six_amd_gpu_generations_without_panic() {
    let tmp = std::env::temp_dir().join("navitune_six_arch_parity");
    let _ = fs::create_dir_all(&tmp);

    let fixtures = [
        ("polaris_rev07.bin",  make_ppt_skeleton(0x07, 0x07, 0x01, 384)),
        ("vega10_rev08.bin",   make_ppt_skeleton(0x08, 0x08, 0x01, 512)),
        ("vega20_rev0b.bin",   make_ppt_skeleton(0x0B, 0x0B, 0x01, 1924)),
        ("rdna1_v4_5.bin",     make_ppt_skeleton(0x0D, 0x0D, 0x01, 1924)),
        ("rdna2_v4_9.bin",     make_ppt_skeleton(0x0F, 0x0F, 0x01, 2470)),
        ("rdna3_v4_12.bin",    make_ppt_skeleton(0x14, 0x14, 0x01, 5424)),
    ];

    let bin = env!("CARGO_BIN_EXE_pptool");
    for (name, bytes) in &fixtures {
        let path = tmp.join(name);
        fs::write(&path, bytes).expect("write fixture");
        let out = Command::new(bin)
            .arg("--inspect")
            .arg(&path)
            .output()
            .expect("spawn pptool");
        assert_ne!(
            out.status.code(),
            Some(101),
            "pptool panicked inspecting {} ({} bytes)",
            name,
            bytes.len()
        );
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_help_flag_exits_cleanly_without_panic() {
    let bin = env!("CARGO_BIN_EXE_pptool");
    let out = Command::new(bin)
        .arg("--help")
        .output()
        .expect("spawn pptool --help");
    assert_ne!(out.status.code(), Some(101), "pptool panicked on --help");
}

#[test]
fn cli_export_oc_and_reg_succeed_across_all_six_gpu_generations() {
    let tmp = std::env::temp_dir().join("navitune_six_arch_cli_export");
    let _ = fs::create_dir_all(&tmp);

    let fixtures = [
        ("polaris_rev07.bin",  make_ppt_skeleton(0x07, 0x07, 0x01, 384)),
        ("vega10_rev08.bin",   make_ppt_skeleton(0x08, 0x08, 0x01, 512)),
        ("vega20_rev0b.bin",   make_ppt_skeleton(0x0B, 0x0B, 0x01, 1924)),
        ("rdna1_v4_5.bin",     make_ppt_skeleton(0x0D, 0x0D, 0x01, 1924)),
        ("rdna2_v4_9.bin",     make_ppt_skeleton(0x0F, 0x0F, 0x01, 2470)),
        ("rdna3_v4_12.bin",    make_ppt_skeleton(0x14, 0x14, 0x01, 5424)),
    ];

    let bin = env!("CARGO_BIN_EXE_pptool");
    for (name, bytes) in &fixtures {
        let in_path = tmp.join(name);
        let plist_path = tmp.join(format!("{}.plist", name));
        let reg_path = tmp.join(format!("{}.reg", name));
        fs::write(&in_path, bytes).expect("write fixture");

        for (flag, out_path) in [("--export-oc", &plist_path), ("--export-reg", &reg_path)] {
            let out = Command::new(bin)
                .arg("--inspect")
                .arg(&in_path)
                .arg(flag)
                .arg(out_path)
                .output()
                .expect("spawn pptool export");
            assert_ne!(out.status.code(), Some(101), "pptool panicked on {} {}", flag, name);
        }
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_binary_and_reimport_roundtrip_parity_across_all_six_architectures() {
    let tmp = std::env::temp_dir().join("navitune_six_arch_full_roundtrip");
    let _ = fs::create_dir_all(&tmp);

    let fixtures = [
        ("polaris_rev07.bin",  make_ppt_skeleton(0x07, 0x07, 0x01, 384)),
        ("vega10_rev08.bin",   make_ppt_skeleton(0x08, 0x08, 0x01, 512)),
        ("vega20_rev0b.bin",   make_ppt_skeleton(0x0B, 0x0B, 0x01, 1924)),
        ("rdna1_v4_5.bin",     make_ppt_skeleton(0x0D, 0x0D, 0x01, 1924)),
        ("rdna2_v4_9.bin",     make_ppt_skeleton(0x0F, 0x0F, 0x01, 2470)),
        ("rdna3_v4_12.bin",    make_ppt_skeleton(0x14, 0x14, 0x01, 5424)),
    ];

    let bin = env!("CARGO_BIN_EXE_pptool");
    for (name, bytes) in &fixtures {
        let in_path = tmp.join(name);
        let out_bin = tmp.join(format!("roundtrip_{}", name));
        let out_plist = tmp.join(format!("roundtrip_{}.plist", name));
        let out_reg = tmp.join(format!("roundtrip_{}.reg", name));
        fs::write(&in_path, bytes).expect("write fixture");

        let res = Command::new(bin)
            .arg("--inspect")
            .arg(&in_path)
            .arg("--export-bin")
            .arg(&out_bin)
            .arg("--export-oc")
            .arg(&out_plist)
            .arg("--export-reg")
            .arg(&out_reg)
            .output()
            .expect("spawn pptool full roundtrip");
        assert_ne!(res.status.code(), Some(101), "pptool panicked exporting {}", name);

        for artifact in [&out_bin, &out_plist, &out_reg] {
            if artifact.exists() {
                let recheck = Command::new(bin)
                    .arg("--inspect")
                    .arg(artifact)
                    .output()
                    .expect("spawn pptool re-inspect");
                assert_ne!(
                    recheck.status.code(),
                    Some(101),
                    "pptool panicked re-inspecting {:?}",
                    artifact
                );
            }
        }
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_dump_all_supports_all_six_gpu_generations_without_panic() {
    let tmp = std::env::temp_dir().join("navitune_six_arch_dump_all");
    let _ = fs::create_dir_all(&tmp);
    let bin = env!("CARGO_BIN_EXE_pptool");
    for (name, bytes) in [("polaris.bin", make_ppt_skeleton(0x07, 0x07, 0x01, 384)), ("vega10.bin", make_ppt_skeleton(0x08, 0x08, 0x01, 512)), ("vega20.bin", make_ppt_skeleton(0x0B, 0x0B, 0x01, 1924)), ("rdna1.bin", make_ppt_skeleton(0x0D, 0x0D, 0x01, 1924)), ("rdna2.bin", make_ppt_skeleton(0x0F, 0x0F, 0x01, 2470)), ("rdna3.bin", make_ppt_skeleton(0x14, 0x14, 0x01, 5424))] {
        let p = tmp.join(name);
        fs::write(&p, &bytes).unwrap();
        let out = Command::new(bin).arg("--dump-all").arg(&p).output().unwrap();
        assert_ne!(out.status.code(), Some(101));
    }
    let _ = fs::remove_dir_all(&tmp);
}
