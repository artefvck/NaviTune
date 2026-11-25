# RDNA 2 PowerPlayTable Layout (`SMU 11.0`, `V4_9`, Navi 21 / 22 / 23 / 24)

Verified directly against `src/arch/rdna2.rs` (`AtomSmcDpmInfoV4_9`, `2470 bytes`, RX 6600 / 6700 / 6800 / 6900 XT).

- **Struct**: `AtomSmcDpmInfoV4_9` (`2470 bytes`)
- **Verified Offset Assertions**: `23` fields checked at compile time

## Architecture Notes

- Full `2470-byte` (`0x9A6`) `AtomSmcDpmInfoV4_9` layout with 64-bit `features_to_run` and `od_feature_flags[0..16]`.
- Supports all 18 `PptCard` panels including RDNA 2-exclusive `Thermal Vmin` (`0x3B2..0x3C2`) and `Power Mode Acoustic` (`0x2F2..0x302`).
- Supports 4-preset `Power Modes` (`Quiet`, `Balanced`, `Turbo`, `Rage` @ `0x2DA..0x2F2`).

## Verified Binary Offset Map

| Offset | Struct Field | Rust Type |
| :--- | :--- | :--- |
| `0x2DA` | `pm_quiet_limit` | `u32` |
| `0x2EA` | `pm_quiet_temp` | `u16` |
| `0x2F2` | `ac_quiet` | `[u16; 2]` |
| `0x322` | `smc_pptable_if_version` | `u32` |
| `0x326` | `features_to_run` | `u64` |
| `0x32E` | `socket_power_limit_ac` | `[u16; 4]` |
| `0x52E` | `freq_gfx_min` | `u16` |
| `0x54E` | `freq_vclk` | `[u16; 2]` |
| `0x55E` | `freq_dclk` | `[u16; 2]` |
| `0x56E` | `freq_soc` | `[u16; 2]` |
| `0x57E` | `freq_uclk` | `[u16; 4]` |
| `0x586` | `freq_dcef` | `[u16; 2]` |
| `0x596` | `freq_disp` | `[u16; 2]` |
| `0x5A6` | `freq_pix` | `[u16; 2]` |
| `0x5B6` | `freq_phy` | `[u16; 2]` |
| `0x5C6` | `freq_dtb` | `[u16; 2]` |
| `0x5D6` | `freq_fclk` | `[u16; 2]` |
| `0x65A` | `mem_clk_div` | `[u8; 4]` |
| `0x6F6` | `pcie_gen_speed` | `[u8; 2]` |
| `0x716` | `fan_pwm_min` | `u16` |
| `0x74E` | `static_volt_gfx` | `QuadraticCoeff` |
| `0x7AA` | `static_volt_offset_gfx` | `QuadraticCoeff` |
| `0x7C2` | `dctol_gfx` | `i16` |

## Verified ULV & Thermal Offsets (V4_9)
- **ULV Voltage (`0x380..0x388`, `u16 / 4` mV)**: `ulv_voltage_vmin_gfx` (`0x380`), `ulv_voltage_vmin_soc` (`0x382`), `ulv_voltage_vmax_gfx` (`0x384`), `ulv_voltage_vmax_soc` (`0x386`).
- **Thermal Vmin (`0x3B2..0x3C2`, `u16` °C)**: `tvmin_gfx` (`0x3B2`), `tvmin_soc` (`0x3B4`), `tvmin_hyst_gfx` (`0x3BE`), `tvmin_hyst_soc` (`0x3C0`).
- **Table Sizes**: `2470 B` (Navi 21 / 22 / 23) and `2690 B` (Navi 24).
