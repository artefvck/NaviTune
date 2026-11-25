# RDNA 1 PowerPlayTable Layout (`SMU 11.0`, `V4_5`, Navi 10 / 12 / 14)

Verified directly against `src/arch/rdna1.rs` (`AtomSmcDpmInfoV4_5`, RX 5500 / 5600 / 5700 XT).

- **Struct**: `AtomSmcDpmInfoV4_5` (`1924 bytes`)
- **Verified Offset Assertions**: `9` fields checked at compile time

## Architecture Notes

- Uses `AtomSmcDpmInfoV4_5` (`rev 0x0C` / `0x0D`) with 64-bit `features_to_run` SMU mask and 16-byte `od_feature_flags`.
- Supports `Core Voltage`, `Memory Voltage`, `ULV Voltage`, `Power & Current`, `Clock Limits`, `Memory DPM`, `PCIe & Link Clocks`, `V/F Curve`, `Voltage Offset`, `Fan Settings & Limits`, `Fan Curve`, and `Thermal Limits`.
- Hides RDNA 2/3-only cards (`Power Modes`, `Power Mode Acoustic`, `Thermal Vmin`) via `PptArch::Rdna1`.

## Verified Binary Offset Map

| Offset | Struct Field | Rust Type |
| :--- | :--- | :--- |
| `0x0AB` | `smc_version` | `u32` |
| `0x0B7` | `socket_power_limit_ac` | `[u16; 4]` |
| `0x0D7` | `tdc_limit_gfx` | `u16` |
| `0x0F3` | `min_voltage_gfx` | `u16` |
| `0x0FF` | `freq_table_gfx` | `[u16; 16]` |
| `0x14F` | `freq_table_uclk` | `[u16; 4]` |
| `0x66B` | `temp_limit` | `[u16; 7]` |
| `0x68B` | `fan_pwm_min` | `u16` |
| `0x697` | `fan_zero_rpm_enable` | `u8` |

## Compact `1674 B` RDNA 1 VBIOS Layout (`outer_sz = 0x01E2`, e.g., Radeon Pro W5500 / RX 5500–5700 XT)

| Offset | Field | Rust Type |
| :--- | :--- | :--- |
| `0x005` | `outer_sz` (`0x01E2`) | `u16` |
| `0x0C2` | `od_feature_flags` | `[u8; 16]` |
| `0x0E2` / `0x162` | `od_gfx_max` / `od_gfx_min` | `u32` |
| `0x0EE` / `0x16E` | `od_mem_max` / `od_mem_min` | `u32` |
| `0x106` / `0x186` | `od_pwr_max` / `od_pwr_min` | `u32` |
| `0x10A` / `0x18A` | `od_fan_max` / `od_fan_min` | `u32` |
| `0x10E` / `0x18E` | `od_acoustic_max` / `od_acoustic_min` | `u32` |
| `0x1E2 + 0x004` | `features_to_run` | `u64` |
| `0x1E2 + 0x00C` | `socket_power_limit_ac` | `u16` |
| `0x1E2 + 0x068` | `min_voltage_gfx` / `max_voltage_gfx` (`mV * 4`) | `u16` |
| `0x1E2 + 0x14C` | `freq_gfx_min` / `freq_gfx_max` | `u16` |
| `0x1E2 + 0x19C` | `freq_table_uclk` (`DPM 0..3`) | `[u16; 4]` |
| `0x11A` / `0x11E` / `0x122` | `od_mem_timing_ctrl` / `od_pwr_mode_ctrl` / `od_zero_rpm_ctrl` | `u32` |

| `0x1E2 + 0x034..0x046` | `temp_limit[0..9]` (`Edge` -> `Liquid1`) | `[u16; 9]` |
| `0x1E2 + 0x294` | `fan_pwm_min` / `fan_acoustic_rpm` / `fan_throttling_rpm` / `fan_max_rpm` | `u16` |
