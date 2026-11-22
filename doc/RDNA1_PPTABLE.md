# RDNA 1 PowerPlayTable Layout (`SMU 11.0`, `V4_5`, Navi 10 / 12 / 14)

Verified directly against `src/pptable_rdna1.rs` (`AtomSmcDpmInfoV4_5`, RX 5500 / 5600 / 5700 XT).

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
