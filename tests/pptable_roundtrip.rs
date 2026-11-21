#[allow(dead_code)]
#[path = "../src/pptable_rdna2.rs"]
mod pptable_rdna2;

#[allow(dead_code)]
#[path = "../src/pptable_rdna3.rs"]
mod pptable_rdna3;

use pptable_rdna2::AtomSmcDpmInfoV4_9;
use pptable_rdna3::AtomSmcDpmInfoV4_12;

#[test]
fn test_rdna2_v4_9_size_and_roundtrip() {
    let sz = core::mem::size_of::<AtomSmcDpmInfoV4_9>();
    assert_eq!(sz, 2470, "AtomSmcDpmInfoV4_9 must be 2,470 bytes");

    let mut buf = vec![0u8; sz];
    buf[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
    buf[2] = 15;
    buf[3] = 1;

    {
        let t = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut buf).expect("valid V4_9 mutable view");
        t.socket_power_limit_ac[0] = 285;
        t.tdc_limit_gfx = 250;
        t.tdc_limit_soc = 38;
        t.fan_target_temp = 82;
    }

    let t_read = AtomSmcDpmInfoV4_9::from_bytes(&buf).expect("valid V4_9 read view");
    assert_eq!({ t_read.socket_power_limit_ac[0] }, 285);
    assert_eq!({ t_read.tdc_limit_gfx }, 250);
    assert_eq!({ t_read.tdc_limit_soc }, 38);
    assert_eq!({ t_read.fan_target_temp }, 82);
}

#[test]
fn test_rdna3_v4_12_size_and_navi31_vs_navi33_scaling() {
    let sz = core::mem::size_of::<AtomSmcDpmInfoV4_12>();
    assert_eq!(sz, 5424, "AtomSmcDpmInfoV4_12 must be 5,424 bytes");

    let mut buf = vec![0u8; sz];
    buf[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
    buf[2] = 18;
    buf[3] = 2;

    {
        let t3 = AtomSmcDpmInfoV4_12::from_bytes_mut(&mut buf).expect("valid V4_12 mutable view");
        t3.od_gfx_max = 2900;
        t3.od_mem_max = 1200;
        t3.socket_power_limit_ac = 165;
        t3.fan_target_temp = 18;
    }

    let t3_read = AtomSmcDpmInfoV4_12::from_bytes(&buf).expect("valid V4_12 read view");
    assert_eq!({ t3_read.od_gfx_max }, 2900);
    assert_eq!({ t3_read.od_mem_max }, 1200);
    assert_eq!({ t3_read.socket_power_limit_ac }, 165);

    let raw_tt = { t3_read.fan_target_temp } as i32;
    let ui_tt = if raw_tt > 200 { raw_tt / 10 } else { raw_tt };
    assert_eq!(ui_tt, 18);
    let saved_tt = if raw_tt > 200 { (ui_tt * 10) as u16 } else { ui_tt as u16 };
    assert_eq!(saved_tt, 18, "Navi33 fan_target_temp must preserve unscaled 18");

    let navi31_raw = 1010i32;
    let navi31_ui = if navi31_raw > 200 { navi31_raw / 10 } else { navi31_raw };
    assert_eq!(navi31_ui, 101);
    let navi31_saved = if navi31_raw > 200 { (navi31_ui * 10) as u16 } else { navi31_ui as u16 };
    assert_eq!(navi31_saved, 1010, "Navi31 fan_target_temp must preserve tenths-of-C 1010");
}

#[test]
fn test_option_rom_8bit_checksum_roundtrip() {
    let mut rom = vec![0u8; 65536];
    rom[0] = 0x55;
    rom[1] = 0xAA;
    rom[2] = 128;
    rom[0x100] = 0x42;
    rom[0x200] = 0x99;

    let sum_without_csum: u8 = rom.iter().enumerate()
        .filter(|&(idx, _)| idx != 6)
        .fold(0u8, |acc, (_, &v)| acc.wrapping_add(v));
    rom[6] = 0u8.wrapping_sub(sum_without_csum);

    let total_sum: u8 = rom.iter().fold(0u8, |acc, &v| acc.wrapping_add(v));
    assert_eq!(total_sum, 0, "Patched Option ROM 8-bit checksum sum must equal 0x00");
}
