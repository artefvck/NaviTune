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
