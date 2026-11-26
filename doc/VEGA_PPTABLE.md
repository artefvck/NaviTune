# Vega 10 (`SMU 9.0`, `rev 0x08`) & Vega 20 (`SMU 11.0`, `rev 0x0B`)

## Vega 10 (`AtomVega10PowerPlayTable`, `rev 0x08`)
- Uses `ATOM_Vega10_Fan_Table` (`rev 10+`) with `Zero RPM` (`Stop Temp` / `Start Temp`), `PWM Min`, `Max RPM`, and `Target Temp`.
- Uses `ulPlatformCaps` (`0x013`, 32-bit `PHM_PlatformCaps_*` mask) rather than 64-bit `FeaturesToRun`. When `ulPlatformCaps == 0x00000000`, the 32-bit SMU Features checklist is locked **read-only** in the UI to prevent writing bitmasks into unused/reserved header space.
- Uses SCLK/MCLK DPM dependency tables, a VDDC Lookup Table (`0x03E`), a PowerTune sub-table (`0x042`), and a Hard Limit sub-table (`0x048`) rather than SMU 11/13 V-F curves.
- **Bidirectional Sync (`try_sync_vega10`)**: Writes `volt_gfx` (VDDC Max) back to the last entry of the VDDC Lookup Table (`vddc_off` @ `0x03E`, clamping lower LUT entries so `entry <= vmax`) and writes `temp_limits` (Hotspot Temp Limit) back to `hl_off + 0x15` (`0x048`), eliminating post-preset binary drift.

### Verified Vega 10 Radeon Pro (`target/Vega10RadeonPro.238574.rom`, `654 B`, `rev 0x08.01.00`)

Verified directly against `target/Vega10RadeonPro.238574.rom` (`262,144 B` VBIOS ROM, AtomBIOS `PowerPlayInfo` @ **`0x00592` / `1426`**, **`654 bytes`**, `rev 0x08.01.00`):

| Table / Sub-Table Pointer | VBIOS ROM Offset | Field Name | Decoded Value (Radeon Pro) |
| :--- | :--- | :--- | :--- |
| `0x000` | `0x000592` | `AtomCommonTableHeader` | `size = 654 B`, `rev = 0x08.01.00` |
| `0x013` | `0x0005A5` | `ulPlatformCaps` | `0x00000000` (locked read-only in UI) |
| `0x017` / `0x01B` | `0x0005A9` / `0x0005AD` | `max_od_engine_clock` / `max_od_memory_clock` | `1200 MHz` GFX / `900 MHz` HBM2 |
| `0x01F` | `0x0005B1` | `power_control_limit` | `0%` OverDrive power ceiling |
| `0x03A` / `0x038` | — | `gfxclk_dep_table` / `socclk_dep_table` | `852..1200 MHz` GFX / `167..600 MHz` SoC |
| `0x03C` | — | `mclk_dep_table` (`DPM 0..3`) | `[600, 900, 900, 900] MHz` HBM2 |
| `0x03E` (`vddc_off` LUT Max) | `0x004F94` (`0x4A02`) | `volt_gfx` (`Vmin / Vmax`) | `800 mV` .. `1000 mV` |
| `0x042` (`pt_off + 0x55/0x57`) | `0x0097E8` / `0x0097EA` | `tdc_gfx` / `power_limit_w` | `144 A` TDC / `150 W` Max Power Limit |
| `0x048` (`hl_off + 0x15`) | `0x0097A8` (`0x9216`) | `temp_limits` (Hotspot Limit) | `115 °C` (`fan_target_temp = 74 °C`, `20%` PWM, `3000 RPM`) |

## Vega 20 (`AtomVega20PowerPlayTable`, `rev 0x0B`)
- Verified against both compact workstation VBIOS PPTable slices (`1534 bytes`, e.g. Radeon Pro VII / Vega 20) and full `1924-byte` tables.
- Automatically slices buffers `< 1924 B` safely (`from_bytes_padded` / `write_back_to_slice`) while preserving trailing VBIOS bytes untouched.
- Key verified offsets:
  - `0x019`: `boost_power_limit` (W)
  - `0x055`: `od_settings_max` (GFX/Mem OverDrive ceilings)
  - `0x11D`: `socket_power_limit_ac` (W)
  - `0x13D`: `tdc_limit_gfx` / `tdc_limit_soc` (A)
  - `0x159`: `min_voltage_gfx` / `max_voltage_gfx` (mV)
  - `0x165`: `freq_table_gfx` (MHz)
  - `0x1B5`: `freq_table_uclk` (MHz)
  - `0x655`: `temp_limit[0..7]` (`°C`, present on full `1924 B` tables)
- Syncs `od_feature_flags[0..16]` directly with the **OverDrive Features** checklist.

## Verified Radeon Pro VII (`target/Vega20RadeonPro.251497.rom`, `1534 B`, `rev 0x0B.00`)

Verified directly against `target/Vega20RadeonPro.251497.rom` (`58,368 B` VBIOS ROM, AtomBIOS `PowerPlayInfo` @ **`0x0CD08`**, **`1,534 bytes`**, `rev 0x0B.00`):

| Table Offset | VBIOS ROM Offset | Field Name | Decoded Value |
| :--- | :--- | :--- | :--- |
| `0x000` | `0x0CD08` | `AtomCommonTableHeader` | `size = 1534 B`, `rev = 0x0B.00` |
| `0x016` | `0x0CD1E` | `BoostPowerLimit / MaxCurrent` | `160 W`, `160 A` GFX, `160 A` SoC |
| `0x027` | `0x0CD2F` | `od_settings_max` (`GFX / Mem`) | `1700 MHz` GFX, `1100 MHz` HBM2 |
| `0x03B` | `0x0CD43` | `od_settings_max` (`GFX Vmin / Vmax`) | `800 mV` .. `1100 mV` (`1200 mV` ceiling) |
| `0x04F` | `0x0CD57` | `od_settings_max` (`GFX Min`) | `810 MHz` |
| `0x33A` | `0x0D042` | `freq_table_gfx[0..7]` | `[300, 500, 800, 1100, 1300, 1500, 1600, 1700] MHz` |
| `0x38A` | `0x0D092` | `freq_table_uclk[0..3]` | `[300, 400, 500, 600] MHz` |
