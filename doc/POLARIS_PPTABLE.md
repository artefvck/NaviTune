# Polaris (`SMU 8.0`) PowerPlay Table Reference

NaviTune supports **AMD Polaris** (`Ellesmere` RX 470/480/570/580/590, `Baffin` RX 460/560, `Lexa` RX 550, `table_format_revision` `0x07`, `384` bytes).

## Supported Silicon & PCI Device IDs
* **Polaris 10/20/30 (`Ellesmere`)**: RX 470, RX 480, RX 570, RX 580, RX 590, Pro WX 5100 / WX 7100 (`0x67C0..=0x67DF`, `0x6FDF`)
* **Polaris 11/21 (`Baffin`)**: RX 460, RX 560, Pro WX 4100 (`0x67E0..=0x67FF`)
* **Polaris 12 (`Lexa`)**: RX 550, Pro WX 2100 / 3100 (`0x6980..=0x699F`)

## Key Byte Offsets (`AtomPolarisPowerPlayTable`, `0x07`)
* `0x017`: `max_od_engine_clock` (`10 kHz` units -> MHz * 100)
* `0x01B`: `max_od_memory_clock` (`10 kHz` units -> MHz * 100)
* `0x01F`: `power_control_limit` (+%)
* `0x025`: `fan_table_offset` -> `ATOM_Polaris_Fan_Table` (`PWM Min`, `Max RPM`, `Target Temp`, `Target GFX Clock`, V3 `Zero RPM`)
* `0x047`: `hard_limit_table_offset` -> `ATOM_Polaris_Hard_Limit_Table` (`GFXCLK`, `MCLK`, `VDDC`)
