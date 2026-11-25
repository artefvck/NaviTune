# RDNA 3 PowerPlayTable Layout (`SMU 13.0`, `V4_12`, Navi 31 / 32 / 33)

Verified directly against `src/arch/rdna3.rs` (`AtomSmcDpmInfoV4_12`, `5424 bytes`, RX 7600 / 7700 / 7800 / 7900 XTX).

- **Struct**: `AtomSmcDpmInfoV4_12` (`5424 bytes`)
- **Verified Offset Assertions**: `8` fields checked at compile time

## Architecture Notes

- Full `5424-byte` (`0x1530`) `AtomSmcDpmInfoV4_12` layout with 64-bit `features_to_run` (`0x344`) and `od_feature_flags[0..20]` (`0x0DF`).
- Uses AVFS per-die fused V/F curves (`max_voltage_gfx = 0`, `min_voltage_gfx > 0`), so `Power Check` only flags `GFX Vmin > Vmax` when `Vmax > 0`.
- Supports dynamic SMU 13.0 `Power Modes` while hiding RDNA 2-only `Thermal Vmin` and `Power Mode Acoustic`.
- **Navi 33 (`format_revision >= 0x16`) Alternate Offsets**: Resolves `freq_gfx_min` from `0x694` (when `0x5EC < 500`) and `fan_target_temp` from `0x886` (when `0x878 < 40`).

## Verified Binary Offset Map

| Offset | Struct Field | Rust Type |
| :--- | :--- | :--- |
| `0x366` | `tdc_limit_gfx` | `u16` |
| `0x7D4` | `mem_vddci` | `[u16; 4]` |
| `0x810` | `pcie_gen_speed` | `[u8; 3]` |
| `0x81E` | `fan_stop_temp` | `u16` |
| `0x838` | `fan_start_temp` | `u16` |
| `0x86A` | `fan_throttling_rpm` | `u16` |
| `0x87C` | `fan_zero_rpm_enable` | `u8` |
