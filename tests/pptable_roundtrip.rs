#[allow(dead_code)]
#[path = "../src/pptable_polaris.rs"]
mod pptable_polaris;

#[allow(dead_code)]
#[path = "../src/ppt_arch.rs"]
mod ppt_arch;

#[allow(dead_code)]
#[path = "../src/pptable_vega.rs"]
mod pptable_vega;

#[allow(dead_code)]
#[path = "../src/pptable_rdna1.rs"]
mod pptable_rdna1;

#[allow(dead_code)]
#[path = "../src/pptable_rdna2.rs"]
mod pptable_rdna2;

#[allow(dead_code)]
#[path = "../src/pptable_rdna3.rs"]
mod pptable_rdna3;

#[allow(dead_code)]
#[path = "../src/vbios.rs"]
mod vbios;

#[allow(dead_code)]
#[path = "../src/export_formats.rs"]
mod export_formats;

#[allow(dead_code)]
#[path = "../src/import_formats.rs"]
mod import_formats;

use pptable_rdna2::AtomSmcDpmInfoV4_9;
use pptable_rdna3::AtomSmcDpmInfoV4_12;

use pptable_rdna1::AtomSmcDpmInfoV4_5;

fn make_rdna1_sample() -> Vec<u8> {
    let sz = core::mem::size_of::<AtomSmcDpmInfoV4_5>();
    let mut buf = vec![0u8; sz];
    buf[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
    buf[2] = 0x0C;
    buf[3] = 1;
    let t1 = AtomSmcDpmInfoV4_5::from_bytes_mut(&mut buf).expect("valid V4_5");
    t1.boost_power_limit = 180;
    t1.socket_power_limit_ac[0] = 180;
    t1.tdc_limit_gfx = 175;
    t1.min_voltage_gfx = 3000;
    t1.max_voltage_gfx = 4800;
    t1.freq_table_gfx[0] = 800;
    t1.freq_table_gfx[15] = 2100;
    t1.fan_target_temperature = 85;
    buf
}

#[test]
fn test_rdna1_v4_5_size_and_roundtrip() {
    let buf = make_rdna1_sample();
    assert_eq!(buf.len(), 1924, "AtomSmcDpmInfoV4_5 must be 1,924 bytes");
    let t1 = AtomSmcDpmInfoV4_5::from_bytes(&buf).expect("valid V4_5 read view");
    assert_eq!({ t1.socket_power_limit_ac[0] }, 180);
    assert_eq!({ t1.tdc_limit_gfx }, 175);
    assert_eq!({ t1.min_voltage_gfx } / 4, 750);
    assert_eq!({ t1.max_voltage_gfx } / 4, 1200);
    assert_eq!({ t1.freq_table_gfx[15] }, 2100);
    assert_eq!({ t1.fan_target_temperature }, 85);
}

fn make_rdna2_sample() -> Vec<u8> {
    let sz = core::mem::size_of::<AtomSmcDpmInfoV4_9>();
    let mut buf = vec![0u8; sz];
    buf[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
    buf[2] = 15;
    buf[3] = 1;
    let t = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut buf).expect("valid V4_9");
    t.socket_power_limit_ac[0] = 285;
    t.tdc_limit_gfx = 250;
    t.tdc_limit_soc = 38;
    t.fan_target_temp = 82;
    buf
}

fn make_rdna3_sample() -> Vec<u8> {
    let sz = core::mem::size_of::<AtomSmcDpmInfoV4_12>();
    let mut buf = vec![0u8; sz];
    buf[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
    buf[2] = 0x15;
    buf[3] = 2;
    let t3 = AtomSmcDpmInfoV4_12::from_bytes_mut(&mut buf).expect("valid V4_12");
    t3.od_gfx_max = 2900;
    t3.od_mem_max = 1200;
    t3.socket_power_limit_ac = 165;
    t3.fan_target_temp = 18;
    buf
}

#[test]
fn test_rdna2_v4_9_size_and_roundtrip() {
    let buf = make_rdna2_sample();
    assert_eq!(buf.len(), 2470, "AtomSmcDpmInfoV4_9 must be 2,470 bytes");
    let t = AtomSmcDpmInfoV4_9::from_bytes(&buf).expect("valid V4_9 read view");
    assert_eq!({ t.socket_power_limit_ac[0] }, 285);
    assert_eq!({ t.tdc_limit_gfx }, 250);
    assert_eq!({ t.tdc_limit_soc }, 38);
    assert_eq!({ t.fan_target_temp }, 82);
}

#[test]
fn test_rdna3_v4_12_size_and_navi31_vs_navi33_scaling() {
    let buf = make_rdna3_sample();
    assert_eq!(buf.len(), 5424, "AtomSmcDpmInfoV4_12 must be 5,424 bytes");
    let t3 = AtomSmcDpmInfoV4_12::from_bytes(&buf).expect("valid V4_12 read view");
    assert_eq!({ t3.od_gfx_max }, 2900);
    assert_eq!({ t3.od_mem_max }, 1200);
    assert_eq!({ t3.socket_power_limit_ac }, 165);

    let raw_tt = { t3.fan_target_temp } as i32;
    let ui_tt = if raw_tt > 200 { raw_tt / 10 } else { raw_tt };
    let saved_tt = if raw_tt > 200 { (ui_tt * 10) as u16 } else { ui_tt as u16 };
    assert_eq!(saved_tt, 18, "Navi33 fan_target_temp must preserve unscaled 18");
}

#[test]
fn test_reg_mpt_plist_format_roundtrip() {
    for sample in [make_rdna1_sample(), make_rdna2_sample(), make_rdna3_sample()] {
        let reg_txt = export_formats::format_reg_ppt(&sample);
        let parsed_reg = import_formats::parse_reg_ppt(reg_txt.as_bytes()).expect("parse .reg");
        assert_eq!(parsed_reg, sample, ".reg round-trip must be byte-exact");

        let mpt_bytes = export_formats::format_mpt_ppt(&sample);
        let parsed_mpt = import_formats::parse_mpt_ppt(&mpt_bytes).expect("parse .mpt");
        assert_eq!(parsed_mpt, sample, ".mpt round-trip must be byte-exact");

        let plist_txt = export_formats::format_opencore_plist(&sample, "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)");
        let parsed_plist = import_formats::parse_plist_ppt(plist_txt.as_bytes()).expect("parse .plist");
        assert_eq!(parsed_plist, sample, "OpenCore .plist round-trip must be byte-exact");
    }
}

#[test]
fn test_vbios_patch_ppt_and_checksum() {
    let mut rom = vec![0u8; 65536];
    rom[0] = 0x55;
    rom[1] = 0xAA;
    rom[2] = 128;

    let sample = make_rdna3_sample();
    let ppt_offset = 0x4000;
    let patched = vbios::patch_ppt_into_rom(&rom, ppt_offset, &sample).expect("patch ppt into rom");
    assert_eq!(&patched[ppt_offset..ppt_offset + sample.len()], sample.as_slice());

    let total_sum: u8 = patched.iter().fold(0u8, |acc, &v| acc.wrapping_add(v));
    assert_eq!(total_sum, 0, "Patched VBIOS ROM 8-bit checksum sum must equal 0x00");
}

#[test]
fn test_ppt_arch_detect_all_gens() {
    let mut r1 = make_rdna1_sample();
    assert_eq!(pptable_rdna1::PptArch::detect(&r1), pptable_rdna1::PptArch::Rdna1);
    r1[2] = 0x0D;
    assert_eq!(pptable_rdna1::PptArch::detect(&r1), pptable_rdna1::PptArch::Rdna1);

    let r2 = make_rdna2_sample();
    assert_eq!(pptable_rdna1::PptArch::detect(&r2), pptable_rdna1::PptArch::Rdna2);

    let r3 = make_rdna3_sample();
    assert_eq!(pptable_rdna1::PptArch::detect(&r3), pptable_rdna1::PptArch::Rdna3);
    assert_eq!(pptable_rdna1::PptArch::detect(&[0u8; 32]), pptable_rdna1::PptArch::Unknown);
}

#[test]
fn test_vega10_and_vega20_roundtrip() {
    let mut v10 = vec![0u8; 512];
    v10[0..2].copy_from_slice(&512u16.to_le_bytes());
    v10[2] = 0x08;
    v10[3] = 1;
    let t10 = pptable_vega::AtomVega10PowerPlayTable::from_bytes_mut(&mut v10).expect("valid Vega10");
    t10.max_od_engine_clock = 175000;
    t10.max_od_memory_clock = 110000;
    t10.power_control_limit = 50;
    assert_eq!(pptable_rdna1::PptArch::detect(&v10), pptable_rdna1::PptArch::Vega10);

    let mut v20 = vec![0u8; 1924];
    v20[0..2].copy_from_slice(&1924u16.to_le_bytes());
    v20[2] = 0x0B;
    v20[3] = 1;
    let t20 = pptable_vega::AtomVega20PowerPlayTable::from_bytes_mut(&mut v20).expect("valid Vega20");
    t20.socket_power_limit_ac[0] = 295;
    t20.tdc_limit_gfx = 260;
    t20.min_voltage_gfx = 3000;
    t20.max_voltage_gfx = 4600;
    assert_eq!(pptable_rdna1::PptArch::detect(&v20), pptable_rdna1::PptArch::Vega20);
    let reg_v20 = export_formats::format_reg_ppt(&v20);
    assert_eq!(import_formats::parse_reg_ppt(reg_v20.as_bytes()).expect("parse v20 reg"), v20);
    let plist_v20 = export_formats::format_opencore_plist(&v20, "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)");
    assert_eq!(import_formats::parse_plist_ppt(plist_v20.as_bytes()).expect("parse v20 plist"), v20);
}

#[test]
fn test_vega10_fan_subtable_roundtrip() {
    let mut v10 = vec![0u8; 512];
    v10[0..2].copy_from_slice(&512u16.to_le_bytes());
    v10[2] = 0x08;
    let fan_off: usize = 0x100;
    pptable_vega::AtomVega10PowerPlayTable::from_bytes_mut(&mut v10).unwrap().fan_table_offset = fan_off as u16;
    v10[fan_off] = 11;
    pptable_vega::AtomVega10PowerPlayTable::write_fan_subtable(&mut v10, 25, 2400, 75, 1650, true, 55, 62);
    let parsed = pptable_vega::AtomVega10PowerPlayTable::from_bytes(&v10).expect("valid v10");
    assert_eq!(parsed.read_fan_subtable(&v10), Some((25, 2400, 75, 1650, true, 55, 62)));
    let hard_off: usize = 0x160;
    pptable_vega::AtomVega10PowerPlayTable::from_bytes_mut(&mut v10).unwrap().hard_limit_table_offset = hard_off as u16;
    v10[hard_off] = 1;
    v10[hard_off + 1] = 1;
    pptable_vega::AtomVega10PowerPlayTable::write_hard_limits_full(&mut v10, 1200, 1750, 1100, 1150, 950, 1350);
    let parsed2 = pptable_vega::AtomVega10PowerPlayTable::from_bytes(&v10).expect("valid v10");
    assert_eq!(parsed2.read_hard_limits(&v10), Some((1200, 1750, 1100, 1150)));
    assert_eq!(parsed2.read_hard_limit_mem_volt(&v10), Some((950, 1350)));
}

#[test]
fn test_vega_chip_detection_all_ids() {
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x687F, 0x08), pptable_vega::VegaChip::Vega10);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x6861, 0x08), pptable_vega::VegaChip::Vega10);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x686C, 0x08), pptable_vega::VegaChip::Vega10);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x66AF, 0x0B), pptable_vega::VegaChip::Vega20);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x66A1, 0x0B), pptable_vega::VegaChip::Vega20);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x0000, 0x08), pptable_vega::VegaChip::Vega10);
    assert_eq!(pptable_vega::VegaChip::from_did_and_rev(0x0000, 0x0B), pptable_vega::VegaChip::Vega20);
    assert_eq!(ppt_arch::PptArch::Vega10.default_table_size(), 512);
    assert_eq!(ppt_arch::PptArch::Vega20.default_table_size(), 1924);
}

#[test]
fn test_polaris_roundtrip() {
    let mut pol = vec![0u8; 384];
    pol[0..2].copy_from_slice(&384u16.to_le_bytes());
    pol[2] = 0x07;
    let hard_off: usize = 0x100;
    let t = pptable_polaris::AtomPolarisPowerPlayTable::from_bytes_mut(&mut pol).expect("valid Polaris");
    t.max_od_engine_clock = 145000;
    t.max_od_memory_clock = 225000;
    t.power_control_limit = 50;
    t.hard_limit_table_offset = hard_off as u16;
    pol[hard_off] = 1;
    pol[hard_off + 1] = 1;
    pptable_polaris::AtomPolarisPowerPlayTable::write_hard_limits_full(&mut pol, 1450, 2250, 1150, 950, 1000);
    let parsed = pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&pol).expect("valid Polaris");
    assert_eq!(parsed.read_hard_limits(&pol), Some((1450, 2250, 1150)));
    assert_eq!(parsed.read_hard_limit_mem_volt(&pol), Some((950, 1000)));
    let fan_off: usize = 0x130;
    pptable_polaris::AtomPolarisPowerPlayTable::from_bytes_mut(&mut pol).unwrap().fan_table_offset = fan_off as u16;
    pol[fan_off] = 9;
    pptable_polaris::AtomPolarisPowerPlayTable::write_fan_subtable(&mut pol, 20, 2200, 75, 910, false, 0, 0);
    assert_eq!(pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&pol).unwrap().read_fan_subtable(&pol), Some((20, 2200, 75, 910, false, 0, 0)));
    assert_eq!(ppt_arch::PptArch::detect(&pol), ppt_arch::PptArch::Polaris);
    let reg_pol = export_formats::format_reg_ppt(&pol);
    assert_eq!(import_formats::parse_reg_ppt(reg_pol.as_bytes()).expect("parse pol reg"), pol);
    let plist_pol = export_formats::format_opencore_plist(&pol, "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)");
    assert_eq!(import_formats::parse_plist_ppt(plist_pol.as_bytes()).expect("parse pol plist"), pol);
}

#[test]
fn test_cross_arch_gcn_and_rdna_detection() {
    assert!(ppt_arch::PptArch::Polaris.is_gcn_subtable());
    assert!(ppt_arch::PptArch::Vega10.is_gcn_subtable());
    assert!(!ppt_arch::PptArch::Vega20.is_gcn_subtable());
    assert!(!ppt_arch::PptArch::Rdna1.is_gcn_subtable());
    assert!(ppt_arch::PptArch::Polaris.gcn_subtable_hint("power").is_some());
    assert!(ppt_arch::PptArch::Vega10.gcn_subtable_hint("freq").is_some());
    assert!(ppt_arch::PptArch::Rdna2.gcn_subtable_hint("fan").is_none());
    assert_eq!(ppt_arch::PptArch::Polaris.default_table_size(), 384);
}

#[test]
fn test_all_six_architectures_parity() {
    let cases: &[(u16, u8, ppt_arch::PptArch)] = &[
        (384, 0x07, ppt_arch::PptArch::Polaris),
        (512, 0x08, ppt_arch::PptArch::Vega10),
        (1924, 0x0B, ppt_arch::PptArch::Vega20),
        (1924, 0x0D, ppt_arch::PptArch::Rdna1),
        (2470, 0x0F, ppt_arch::PptArch::Rdna2),
        (5424, 0x14, ppt_arch::PptArch::Rdna3),
    ];
    for &(sz, rev, expected) in cases {
        let mut buf = vec![0u8; sz as usize];
        buf[0..2].copy_from_slice(&sz.to_le_bytes());
        buf[2] = rev;
        assert_eq!(ppt_arch::PptArch::detect(&buf), expected);
        assert_eq!(expected.default_table_size(), sz as usize);
        assert_ne!(expected.smu_generation(), "Unknown");
        let reg = export_formats::format_reg_ppt(&buf);
        assert_eq!(import_formats::parse_reg_ppt(reg.as_bytes()).expect("reg roundtrip"), buf);
        let plist = export_formats::format_opencore_plist(&buf, "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)");
        assert_eq!(import_formats::parse_plist_ppt(plist.as_bytes()).expect("plist roundtrip"), buf);
    }
}

#[test]
fn test_new_blank_table_all_six_archs() {
    for arch in [
        ppt_arch::PptArch::Polaris,
        ppt_arch::PptArch::Vega10,
        ppt_arch::PptArch::Vega20,
        ppt_arch::PptArch::Rdna1,
        ppt_arch::PptArch::Rdna2,
        ppt_arch::PptArch::Rdna3,
    ] {
        let buf = arch.new_blank_table();
        assert_eq!(buf.len(), arch.default_table_size());
        assert_eq!(ppt_arch::PptArch::detect(&buf), arch);
    }
}

#[test]
fn test_ppt_arch_capability_matrix_all_six() {
    use ppt_arch::PptArch::*;
    for arch in [Polaris, Vega10] {
        assert!(arch.has_hard_limit_subtable());
        assert!(!arch.has_od8_and_smu_masks());
        assert!(!arch.has_multi_power_modes());
        assert!(!arch.has_vf_curve_points());
    }
    for arch in [Vega20, Rdna1] {
        assert!(!arch.has_hard_limit_subtable());
        assert!(arch.has_od8_and_smu_masks());
        assert!(!arch.has_multi_power_modes());
        assert!(arch.has_vf_curve_points());
    }
    for arch in [Rdna2, Rdna3] {
        assert!(!arch.has_hard_limit_subtable());
        assert!(arch.has_od8_and_smu_masks());
        assert!(arch.has_multi_power_modes());
        assert!(arch.has_vf_curve_points());
    }
}

#[test]
fn test_features_to_run_all_six_architectures() {
    use ppt_arch::PptArch::*;
    assert_eq!(Polaris.feature_bit_name(0), "POWER_CONTAINMENT");
    assert_eq!(Vega10.feature_bit_name(11), "FAN_SPEED_IN_RPM");
    assert_eq!(Vega20.feature_bit_name(17), "GFXOFF");
    assert_eq!(Rdna1.feature_bit_name(30), "FAN_CONTROL");
    assert_eq!(Rdna2.feature_bit_name(36), "OUT_OF_BAND_MONITOR");
    assert_eq!(Rdna3.feature_bit_name(19), "GFXOFF");
    assert_eq!(Rdna3.feature_bit_name(30), "FAN_CONTROL");
    let decoded = Rdna3.decode_feature_bits((1u64 << 1) | (1u64 << 19));
    assert_eq!(decoded, vec![(1, "DPM_GFXCLK"), (19, "GFXOFF")]);
}

#[test]
fn test_power_modes_and_acoustic_capabilities_all_six() {
    use ppt_arch::PptArch::*;
    for arch in [Polaris, Vega10, Vega20, Rdna1] {
        assert!(!arch.has_multi_power_modes());
        assert!(!arch.has_power_mode_acoustic());
    }
    assert!(Rdna2.has_multi_power_modes());
    assert!(Rdna2.has_power_mode_acoustic());
    assert!(Rdna3.has_multi_power_modes());
    assert!(!Rdna3.has_power_mode_acoustic());
}
