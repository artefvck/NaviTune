use std::path::Path;

#[allow(dead_code)]
#[path = "../src/vbios.rs"]
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
