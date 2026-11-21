# RDNA 1 (`SMU 11.0`) Soft PowerPlay Table Reference

NaviTune supports **AMD RDNA 1** (`Navi 10`, `Navi 12`, `Navi 14`) via `AtomSmcDpmInfoV4_5` (`1,924` bytes, `table_format_revision` `0x0C` / `0x0D`).

## Supported Silicon & PCI Device IDs
* **Navi 10**: RX 5600 / 5600 XT, RX 5700 / 5700 XT, Radeon Pro W5700 (`0x7310`, `0x7312`, `0x731F`)
* **Navi 12**: Radeon Pro V520, RX 5600M (`0x7360`, `0x7362`)
* **Navi 14**: RX 5300 / 5500 / 5500 XT, Radeon Pro W5500 (`0x7340`, `0x7341`, `0x7347`, `0x734F`)

## Key Byte Offsets (`AtomSmcDpmInfoV4_5`)
* `0x01D`: `boost_power_limit` (W)
* `0x02F` / `0x063`: `od_settings_max[0..13]` / `od_settings_min[0..13]` (GFXCLK, UCLK, Power %, Fan RPM, Acoustic RPM, Target Temp)
* `0x09B`: `od_feature_flags[0..16]`
* `0x0B7`: `socket_power_limit_ac[0..4]` (`PPT0..PPT3` in W)
* `0x0D7` / `0x0D9`: `tdc_limit_gfx` / `tdc_limit_soc` (A)
* `0x0F3..0x0F9`: `min_voltage_gfx`, `min_voltage_soc`, `max_voltage_gfx`, `max_voltage_soc` (`0.25 mV` units)
* `0x0FF` / `0x14F`: `freq_table_gfx[0..16]` / `freq_table_uclk[0..4]` (MHz)
* `0x66B`: `temp_limit[0..7]` (°C)
* `0x679` / `0x68B` / `0x697`: `fan_stop_temp`, `fan_start_temp`, `fan_pwm_min`, `fan_zero_rpm_enable`
