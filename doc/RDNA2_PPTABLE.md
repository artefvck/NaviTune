# RDNA 2 (`SMU 11.0`) Soft PowerPlay Table Reference

NaviTune supports **AMD RDNA 2** (`Navi 21`, `Navi 22`, `Navi 23`, `Navi 24`) via `AtomSmcDpmInfoV4_9` (`2,470` bytes, `table_format_revision` `0x0F`–`0x13`).

## Supported Silicon & PCI Device IDs
* **Navi 21**: RX 6800 / 6800 XT / 6900 XT / 6950 XT, Radeon Pro W6800 (`0x73BF`, `0x73AF`)
* **Navi 22**: RX 6700 / 6700 XT / 6750 XT (`0x73DF`)
* **Navi 23**: RX 6600 / 6600 XT / 6650 XT, Radeon Pro W6600 (`0x73FF`, `0x73EF`, `0x73E3`)
* **Navi 24**: RX 6400 / 6500 XT, Radeon Pro W6400 (`0x743F`)

## Key Byte Offsets (`AtomSmcDpmInfoV4_9`)
* `0x0C2`: `od_feature_flags`
* `0x0FA`: `socket_power_limit_ac` (`PPT0..PPT3` in W)
* `0x11A` / `0x11C`: `tdc_limit_gfx` / `tdc_limit_soc` (A)
* `0x13A..0x140`: `vmin_gfx`, `vmin_soc`, `vmax_gfx`, `vmax_soc` (`0.25 mV` units)
* `0x57E`: `freq_uclk` (MHz + Div)
* `0x6F6` / `0x6F8` / `0x6FA`: `pcie_gen_speed`, `pcie_lane_count`, `lclk_freq`
* `0x8A2`..`0x8BE`: Fan PWM, RPM, Zero RPM stop/start temperatures, and target temp
