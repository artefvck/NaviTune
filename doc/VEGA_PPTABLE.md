# Vega 10 (`SMU 9.0`, `rev 0x08`) & Vega 20 (`SMU 11.0`, `rev 0x0B`)

## Vega 10 (`AtomVega10PowerPlayTable`, `rev 0x08`)
- Uses `ATOM_Vega10_Fan_Table` (`rev 10+`) with `Zero RPM` (`Stop Temp` / `Start Temp`), `PWM Min`, `Max RPM`, and `Target Temp`.
- Uses `ulPlatformCaps` (`0x013`, 32-bit `PHM_PlatformCaps_*` mask) rather than 64-bit `FeaturesToRun`.
- Uses SCLK/MCLK DPM dependency tables rather than SMU 11/13 V-F curves.

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
