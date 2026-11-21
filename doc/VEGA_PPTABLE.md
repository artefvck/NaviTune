# Vega 10 (`SMU 9.0`) & Vega 20 (`SMU 11.0`) PowerPlay Table Reference

NaviTune supports **AMD Vega 10** (`AtomVega10PowerPlayTable`, `table_format_revision` `0x08`) and **AMD Vega 20** (`AtomVega20PowerPlayTable`, `1,924` bytes, `table_format_revision` `0x0B`).

## Supported Silicon & PCI Device IDs
* **Vega 10 (`0x08`)**: RX Vega 56, RX Vega 64, Radeon Vega Frontier Edition, Radeon Pro WX 8200 / WX 9100 (`0x6860`, `0x6863`, `0x687F`)
* **Vega 20 (`0x0B`)**: Radeon VII, Radeon Pro Vega II / Vega II Duo (`0x66A0`, `0x66A1`, `0x66AF`)

## Key Byte Offsets
### Vega 10 (`AtomVega10PowerPlayTable`, `0x08`)
* `0x017`: `max_od_engine_clock` (`10 kHz` units -> MHz * 100)
* `0x01B`: `max_od_memory_clock` (`10 kHz` units -> MHz * 100)
* `0x01F`: `power_control_limit` (+%)

### Vega 20 (`AtomVega20PowerPlayTable`, `0x0B`, `1,924` bytes)
* `0x055` / `0x0A9`: `od_settings_max[0..21]` / `od_settings_min[0..21]`
* `0x11D`: `socket_power_limit_ac[0..4]` (`PPT0..PPT3` in W)
* `0x13D` / `0x13F`: `tdc_limit_gfx` / `tdc_limit_soc` (A)
* `0x159..0x15F`: `min_voltage_gfx`, `min_voltage_soc`, `max_voltage_gfx`, `max_voltage_soc` (`0.25 mV` units)
* `0x165` / `0x1B5`: `freq_table_gfx[0..16]` / `freq_table_uclk[0..4]` (MHz)
* `0x655`: `temp_limit[0..7]` (°C)
* `0x663` / `0x675` / `0x681`: `fan_stop_temp`, `fan_start_temp`, `fan_pwm_min`, `fan_zero_rpm_enable`
