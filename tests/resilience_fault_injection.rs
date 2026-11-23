use std::fs;
use std::process::Command;

#[test]
fn cli_inspect_handles_malformed_and_truncated_inputs_without_panic() {
    let tmp = std::env::temp_dir().join("navitune_resilience_test");
    let _ = fs::create_dir_all(&tmp);

    let empty = tmp.join("empty.bin");
    fs::write(&empty, b"").expect("write empty");

    let one_byte = tmp.join("one_byte.bin");
    fs::write(&one_byte, b"\x42").expect("write 1-byte");

    let mut corrupt_rom = vec![0u8; 4096];
    corrupt_rom[0] = 0x55;
    corrupt_rom[1] = 0xAA;
    corrupt_rom[0x48] = 0xFF;
    corrupt_rom[0x49] = 0xFF;
    let rom_path = tmp.join("corrupt_offsets.rom");
    fs::write(&rom_path, &corrupt_rom).expect("write corrupt rom");

    let bin = env!("CARGO_BIN_EXE_pptool");
    for path in [&empty, &one_byte, &rom_path] {
        let out = Command::new(bin)
            .arg("--inspect")
            .arg(path)
            .output()
            .expect("spawn pptool");
        assert_ne!(out.status.code(), Some(101), "pptool panicked on {:?}", path);
    }

    // Verify exporting into a non-existent nested directory fails cleanly without panic
    let bad_export = tmp.join("nonexistent_dir_12345").join("out.plist");
    let out = Command::new(bin)
        .arg("--inspect")
        .arg(&rom_path)
        .arg("--export-oc")
        .arg(&bad_export)
        .output()
        .expect("spawn pptool export");
    assert_ne!(out.status.code(), Some(101), "pptool panicked on invalid export target");

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_handles_corrupt_plist_and_reg_imports_without_panic() {
    let tmp = std::env::temp_dir().join("navitune_import_resilience_test");
    let _ = fs::create_dir_all(&tmp);

    let bad_plist = tmp.join("corrupt.plist");
    fs::write(
        &bad_plist,
        b"<?xml version=\"1.0\"?><plist><dict><key>PP_PhmSoftPowerPlayTable</key><data>!!!NOT_BASE64!!!</data></dict></plist>",
    )
    .expect("write bad plist");

    let bad_reg = tmp.join("corrupt.reg");
    fs::write(
        &bad_reg,
        b"Windows Registry Editor Version 5.00\r\n\"PP_PhmSoftPowerPlayTable\"=hex:ZZ,YY,XX,\\\r\n  00,11\r\n",
    )
    .expect("write bad reg");

    let bin = env!("CARGO_BIN_EXE_pptool");
    for path in [&bad_plist, &bad_reg] {
        let out = Command::new(bin)
            .arg("--inspect")
            .arg(path)
            .output()
            .expect("spawn pptool");
        assert_ne!(out.status.code(), Some(101), "pptool panicked on {:?}", path);
    }

    let _ = fs::remove_dir_all(&tmp);
}
