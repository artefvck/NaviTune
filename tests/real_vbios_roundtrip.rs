use std::path::Path;

#[allow(dead_code)]
#[path = "../src/vbios.rs"]
mod vbios;

#[allow(dead_code)]
#[path = "../src/pptable_vega.rs"]
mod pptable_vega;

#[allow(dead_code)]
#[path = "../src/pptable_polaris.rs"]
mod pptable_polaris;

#[test]
fn test_real_rx580_2048sp_and_vega20_pro_vbios_roundtrip() {
    // 1. Verify RX 580 2048SP (262,144 B VBIOS @ 0x09C0E, 833 B, rev 0x07.01.00)
    if Path::new("target/rx580.rom").exists() {
        let rom = std::fs::read("target/rx580.rom").expect("read rx580.rom");
        let (mut ppt, ppt_off) = vbios::extract_ppt_from_rom(&rom)
            .expect("extract Polaris PowerPlayInfo from target/rx580.rom");
        assert_eq!(ppt_off, 0x09C0E);
        assert_eq!(ppt.len(), 833);

        let pol = pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&ppt)
            .expect("parse Polaris PPTable");
        let (s_min, s_max, v_min, v_max, pwr_w, tdc_a, m_states) = pol.read_polaris_dpm_and_powertune(&ppt);
        assert_eq!((s_min, s_max), (300, 1244));
        assert_eq!((v_min, v_max), (750, 1150));
        assert_eq!((pwr_w, tdc_a), (144, 92));
        assert_eq!(m_states.len(), 3);
        assert_eq!(m_states[0], (300, 900, 1000));
        assert_eq!(m_states[2], (1750, 900, 1000));

        let (pwm_min, max_rpm, tgt_temp, tgt_gfx, _, _, _) = pol.read_fan_subtable(&ppt)
            .expect("parse Polaris Fan subtable");
        assert_eq!((pwm_min, max_rpm, tgt_temp, tgt_gfx), (20, 2200, 75, 910));

        pptable_polaris::AtomPolarisPowerPlayTable::write_fan_subtable(
            &mut ppt, pwm_min, max_rpm, tgt_temp, tgt_gfx, false, 0, 0,
        );
        assert_eq!(ppt.len(), 833);
    }

    // 2. Verify Radeon Pro VII (58,368 B VBIOS @ 0x0CD08, 1534 B, rev 0x0B.00)
    if Path::new("target/Vega20RadeonPro.251497.rom").exists() {
        let rom = std::fs::read("target/Vega20RadeonPro.251497.rom").expect("read Vega20RadeonPro.251497.rom");
        let (ppt, ppt_off) = vbios::extract_ppt_from_rom(&rom)
            .expect("extract Vega 20 PowerPlayInfo from target/Vega20RadeonPro.251497.rom");
        assert_eq!(ppt_off, 0x0CD08);
        assert_eq!(ppt.len(), 1534);
        assert_eq!(u16::from_le_bytes([ppt[0x016], ppt[0x017]]), 160);
        assert_eq!(u32::from_le_bytes([ppt[0x027], ppt[0x028], ppt[0x029], ppt[0x02A]]), 1700);
        assert_eq!(u32::from_le_bytes([ppt[0x02B], ppt[0x02C], ppt[0x02D], ppt[0x02E]]), 1100);
        assert_eq!(u32::from_le_bytes([ppt[0x03B], ppt[0x03C], ppt[0x03D], ppt[0x03E]]), 800);
        assert_eq!(u32::from_le_bytes([ppt[0x043], ppt[0x044], ppt[0x045], ppt[0x046]]), 1100);
        assert_eq!(u16::from_le_bytes([ppt[0x33A], ppt[0x33B]]), 300);
        assert_eq!(u16::from_le_bytes([ppt[0x348], ppt[0x349]]), 1700);
        assert_eq!(u16::from_le_bytes([ppt[0x38A], ppt[0x38B]]), 300);
        assert_eq!(u16::from_le_bytes([ppt[0x390], ppt[0x391]]), 600);
    }
}
