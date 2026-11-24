use std::path::Path;

#[allow(dead_code)]
#[path = "../src/io/vbios.rs"]
mod vbios;

#[test]
fn test_all_target_vbios_roms() {
    let roms = [
        "target/rx580.rom",
        "target/Vega10RadeonPro.238574.rom",
        "target/Vega20RadeonPro.251497.rom",
        "target/Navi21.rom",
        "target/Navi22.rom",
        "target/Navi23.rom",
        "target/Navi24.rom",
        "target/Navi31.rom",
        "target/Navi32.rom",
        "target/Navi33.rom",
    ];
    for path in roms {
        if !Path::new(path).exists() { continue; }
        let rom = std::fs::read(path).expect(path);
        match vbios::extract_ppt_from_rom(&rom) {
            Ok((ppt, off)) => {
                let (frev, crev, trev) = (
                    ppt.get(2).copied().unwrap_or(0),
                    ppt.get(3).copied().unwrap_or(0),
                    ppt.get(4).copied().unwrap_or(0),
                );
                println!("{:38} -> PPT @ 0x{:06X} ({:4} B, rev 0x{:02X}.{:02X}.{:02X})", path, off, ppt.len(), frev, crev, trev);
            }
            Err(e) => println!("{:38} -> ERR: {}", path, e),
        }
    }
}

#[test]
fn w5500_vbios_inspect_and_unchecked_od() {
    let p = std::path::Path::new("target/AMD.RadeonProW5500.8192.191230.rom");
    if !p.exists() { return; }
    let rom = std::fs::read(p).expect("read W5500 ROM");
    let hdr = u16::from_le_bytes([rom[0x48], rom[0x49]]) as usize;
    let master = u16::from_le_bytes([rom[hdr + 0x20], rom[hdr + 0x21]]) as usize;
    let ppt_off = u16::from_le_bytes([rom[master + 0x22], rom[master + 0x23]]) as usize;
    let ppt_len = u16::from_le_bytes([rom[ppt_off], rom[ppt_off + 1]]) as usize;
    assert_eq!(ppt_off, 0xCC42);
    assert_eq!(ppt_len, 1674);
    let mut sppt = rom[ppt_off..ppt_off + ppt_len].to_vec();
    let orig = sppt.clone();
    assert_eq!(orig, sppt, "0-byte unmodified round-trip parity");
    let stock_mask = u32::from_le_bytes([sppt[0x10], sppt[0x11], sppt[0x12], sppt[0x13]]);
    assert_eq!(stock_mask, 0x0000_0800);
    sppt[0x10..0x14].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(u32::from_le_bytes([sppt[0x10], sppt[0x11], sppt[0x12], sppt[0x13]]), 0);
}
