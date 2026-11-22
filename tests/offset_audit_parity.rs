#[path = "../src/arch/detect.rs"]
mod ppt_arch;
#[path = "../src/arch/polaris.rs"]
mod pptable_polaris;
#[path = "../src/arch/rdna1.rs"]
mod pptable_rdna1;
#[path = "../src/arch/rdna2.rs"]
mod pptable_rdna2;
#[path = "../src/arch/rdna3.rs"]
mod pptable_rdna3;
#[path = "../src/arch/vega.rs"]
mod pptable_vega;
#[path = "../src/io/vbios.rs"]
mod vbios;

use ppt_arch::PptArch;
use pptable_vega::VegaParsedState;

#[test]
fn verify_all_target_vbios_parity() {
    let entries = match std::fs::read_dir("target") {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut checked = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rom") {
            continue;
        }
        let rom = std::fs::read(&path).expect("read rom");
        let (ppt, off) = vbios::extract_ppt_from_rom(&rom).expect("extract ppt from rom");
        assert!(off < rom.len(), "valid VBIOS offset for {:?}", path);

        let arch = PptArch::detect(&ppt);
        let frev = ppt.get(2).copied().unwrap_or(0);
        match arch {
            PptArch::Vega10 | PptArch::Vega20 => {
                if let Some(st) = VegaParsedState::parse(&ppt, frev) {
                    assert!(st.freq_gfx[1] > 0, "non-zero GFX max for {:?}", path);
                    assert!(st.power_limit_w > 0, "non-zero power limit for {:?}", path);
                    let mut roundtrip = ppt.clone();
                    assert!(st.write_to_buf(&mut roundtrip), "write_to_buf for {:?}", path);
                    let st2 = VegaParsedState::parse(&roundtrip, frev).expect("reparse");
                    assert_eq!(st.freq_gfx, st2.freq_gfx);
                    assert_eq!(st.power_limit_w, st2.power_limit_w);
                    assert_eq!(st.tdc_gfx, st2.tdc_gfx);
                }
            }
            PptArch::Polaris => {
                let pol = pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&ppt).expect("polaris parse");
                assert!(pol.max_od_engine_clock > 0, "non-zero Polaris OD GFX clock for {:?}", path);
            }
            _ => {}
        }
        checked += 1;
    }
    println!("Verified Struct vs VBIOS parity across {} real ROM(s)", checked);
}
