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
                    let diffs: Vec<_> = ppt.iter().zip(roundtrip.iter()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, (a, b))| format!("0x{:03X}: 0x{:02X}->0x{:02X}", i, a, b)).collect(); assert!(diffs.is_empty(), "diffs in {:?}: {:?}", path, diffs);
                }
            }
            PptArch::Polaris => {
                let pol = pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&ppt).expect("polaris parse");
                assert!(pol.max_od_engine_clock > 0, "non-zero Polaris OD GFX clock for {:?}", path);
                let (sclk_min, sclk_max, _vddc_min, vddc_max, pwr_w, tdc_a, mclk_rows) = pol.read_polaris_dpm_and_powertune(&ppt);
                let mut mem_f = [[0; 2]; 4];
                let mut mem_v = [[0; 2]; 4];
                for (i, &(f, v, _)) in mclk_rows.iter().take(4).enumerate() {
                    mem_f[i][0] = f;
                    mem_v[i][0] = v;
                }
                let mut roundtrip = ppt.clone();
                pptable_polaris::AtomPolarisPowerPlayTable::write_polaris_dpm_and_powertune(&mut roundtrip, sclk_min, sclk_max, vddc_max, pwr_w, tdc_a, &mem_f, &mem_v);
                if let Some((pwm_min, max_rpm, tgt_t, tgt_g, zrpm, stop_t, start_t)) = pol.read_fan_subtable(&ppt) {
                    pptable_polaris::AtomPolarisPowerPlayTable::write_fan_subtable(&mut roundtrip, pwm_min, max_rpm, tgt_t, tgt_g, zrpm, stop_t, start_t);
                }
                let diffs: Vec<_> = ppt.iter().zip(roundtrip.iter()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, (a, b))| format!("0x{:03X}: 0x{:02X}->0x{:02X}", i, a, b)).collect();
                assert!(diffs.is_empty(), "Polaris diffs in {:?}: {:?}", path, diffs);
            }
            PptArch::Rdna1 => {
                if let Some(t) = pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&ppt) {
                    assert!(t.socket_power_limit_ac[0] > 0, "non-zero RDNA1 power limit for {:?}", path);
                    let mut roundtrip = ppt.clone();
                    let tm = pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes_mut(&mut roundtrip).expect("rdna1 mut");
                    let qmv = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
                    tm.min_voltage_gfx = qmv((t.min_voltage_gfx / 4) as i32, tm.min_voltage_gfx);
                    tm.max_voltage_gfx = qmv((t.max_voltage_gfx / 4) as i32, tm.max_voltage_gfx);
                    let diffs: Vec<_> = ppt.iter().zip(roundtrip.iter()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, (a, b))| format!("0x{:03X}: 0x{:02X}->0x{:02X}", i, a, b)).collect();
                    assert!(diffs.is_empty(), "RDNA1 diffs in {:?}: {:?}", path, diffs);
                }
            }
            PptArch::Rdna2 => {
                if let Some(t) = pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&ppt) {
                    assert!(t.socket_power_limit_ac[0] > 0, "non-zero RDNA2 power limit for {:?}", path);
                    assert!(t.freq_gfx_max > 0, "non-zero RDNA2 GFX max for {:?}", path);
                    let mut roundtrip = ppt.clone();
                    let tm = pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes_mut(&mut roundtrip).expect("rdna2 mut");
                    let qmv = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
                    tm.min_voltage_gfx = qmv((t.min_voltage_gfx / 4) as i32, tm.min_voltage_gfx);
                    tm.min_voltage_soc = qmv((t.min_voltage_soc / 4) as i32, tm.min_voltage_soc);
                    tm.vmin_low_gfx = qmv((t.vmin_low_gfx / 4) as i32, tm.vmin_low_gfx);
                    tm.vmin_high_gfx = qmv((t.vmin_high_gfx / 4) as i32, tm.vmin_high_gfx);
                    tm.vmin_low_soc = qmv((t.vmin_low_soc / 4) as i32, tm.vmin_low_soc);
                    tm.vmin_high_soc = qmv((t.vmin_high_soc / 4) as i32, tm.vmin_high_soc);
                    let diffs: Vec<_> = ppt.iter().zip(roundtrip.iter()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, (a, b))| format!("0x{:03X}: 0x{:02X}->0x{:02X}", i, a, b)).collect();
                    assert!(diffs.is_empty(), "RDNA2 diffs in {:?}: {:?}", path, diffs);
                }
            }
            PptArch::Rdna3 => {
                if let Some(t) = pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&ppt) {
                    assert!(t.socket_power_limit_ac > 0, "non-zero RDNA3 power limit for {:?}", path);
                    assert!(t.freq_gfx_max > 0, "non-zero RDNA3 GFX max for {:?}", path);
                    let mut roundtrip = ppt.clone();
                    let tm = pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes_mut(&mut roundtrip).expect("rdna3 mut");
                    let qmv = |ui_mv: i32, orig: u16| if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 };
                    tm.min_voltage_gfx = qmv((t.min_voltage_gfx / 4) as i32, tm.min_voltage_gfx);
                    tm.max_voltage_gfx = qmv((t.max_voltage_gfx / 4) as i32, tm.max_voltage_gfx);
                    let diffs: Vec<_> = ppt.iter().zip(roundtrip.iter()).enumerate().filter(|(_, (a, b))| a != b).map(|(i, (a, b))| format!("0x{:03X}: 0x{:02X}->0x{:02X}", i, a, b)).collect();
                    assert!(diffs.is_empty(), "RDNA3 diffs in {:?}: {:?}", path, diffs);
                }
            }
            _ => {}
        }
        checked += 1;
    }
    println!("Verified Struct vs VBIOS parity across {} real ROM(s)", checked);
}
