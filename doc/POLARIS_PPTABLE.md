# Polaris PowerPlayTable Layout (`SMU 8.0`, `rev 0x07`)

Verified against real Ellesmere RX 580 VBIOS ROMs (`833 bytes`, `rev 0x07.01.00`, VBIOS offset `0x09C0E`).

## Header & Capability Offsets

| Offset | Field | Type | Description |
| :--- | :--- | :--- | :--- |
| `0x000` | `AtomCommonTableHeader` | 5 B | `size = 833`, `frev = 0x07`, `crev = 0x01`, `trev = 0x00` |
| `0x013` | `ulPlatformCaps` | `u32` | `PHM_PlatformCaps_*` mask (RX 580 default `0x01060000`: bit 17 `DISABLE_VOLTAGE_ISLAND`, bit 18 `ODN_AC_SUPPORT`, bit 24 `FALCON_QUICK_TRANSITION`) |
| `0x017` | `ulMaxODEngineClock` | `u32` | OverDrive GFX clock ceiling in 10 kHz (`200000` = `2000 MHz`) |
| `0x01B` | `ulMaxODMemoryClock` | `u32` | OverDrive Memory clock ceiling in 10 kHz (`225000` = `2250 MHz`) |
| `0x01F` | `usPowerControlLimit` | `u16` | OverDrive Power slider limit (`+50%`) |

## Verified RX 580 Sub-Tables

| Sub-Table | Table Offset | VBIOS Offset | Entry Stride & Decoded Fields |
| :--- | :--- | :--- | :--- |
| `vddc_lookup_table` | `0x077` | `0x09C85` | `VDDC = 750..1150 mV` |
| `sclk_dep_table` | `0x13B` | `0x09D49` | `15 B` stride (`ulSclk` at `+3..+7`): DPM 0 = `300 MHz`, DPM 7 (`0x1A6`) = `1244 MHz` |
| `mclk_dep_table` | `0x1B5` | `0x09DC3` | `13 B` stride (`VDDCI` `+1..+3`, `MVDD` `+5..+7`, `ulMclk` `+7..+11`): `300 / 1000 / 1750 MHz` @ `900 / 1000 mV` |
| `ATOM_Tonga_Fan_Table` | `0x2A1` | `0x09EAF` | `rev 9`: `Min PWM = 20%`, `Max RPM = 2200 RPM`, `Target Temp = 75°C`, `Target GFX = 910 MHz` |
| `ATOM_Polaris_PowerTune` | `0x2D1` | `0x09EDF` | `rev 4`: `Max Power = 144 W` (`+0x2D`), `TDC = 92 A` (`+0x05`) |
| `ATOM_Polaris_Hard_Limit_Table` | `0x000` | — | Absent (`0x000`) on `rev 0x07.01`; read/write falls back to DPM & PowerTune sub-tables |
