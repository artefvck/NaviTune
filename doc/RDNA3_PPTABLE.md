# RDNA 3 (`SMU 13.0`) Soft PowerPlay Table Reference

NaviTune supports **AMD RDNA 3** (`Navi 31`, `Navi 32`, `Navi 33`) via `AtomSmcDpmInfoV4_12` (`5,424` bytes, `table_format_revision` `0x14`–`0x16`).

## Supported Silicon & PCI Device IDs
* **Navi 31**: RX 7900 GRE / 7900 XT / 7900 XTX, Radeon Pro W7900 / W7800 (`0x744C`)
* **Navi 32**: RX 7700 XT / 7800 XT (`0x7470`, `0x747E`)
* **Navi 33**: RX 7600 / 7600 XT, Radeon Pro W7600 / W7500 (`0x7480`, `0x7483`)

## Key Byte Offsets (`AtomSmcDpmInfoV4_12`)
* `0x036`: `od_feature_flags`
* `0x056` / `0x058`: `od_gfx_max` / `od_mem_max` (MHz)
* `0x064`: `od_pwr_max` (+%)
* `0x0FA`: `socket_power_limit_ac` (`PPT0` in W)
* `0x126` / `0x128`: `tdc_limit_gfx` / `tdc_limit_soc` (A)
* `0x14E..0x154`: `min_voltage_gfx`, `min_voltage_soc`, `max_voltage_gfx`, `max_voltage_soc` (`0.25 mV` units)
* `0x63C`: `freq_uclk` (MHz)
* `0x810` / `0x813` / `0x816`: `pcie_gen_speed` (0-indexed), `pcie_lane_count`, `lclk_freq`
* `0x9E0`..`0xA14`: Fan PWM/RPM, `fan_target_temp` (tenths of °C on Navi 31, °C on Navi 33), and Zero RPM
