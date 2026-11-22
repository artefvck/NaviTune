# Polaris PowerPlayTable Layout (`SMU 8.0`, `rev 0x07`)

Verified directly against an **AMD Radeon RX 580 2048SP** VBIOS ROM (`target/rx580.rom`, `262,144 bytes` / `256 KiB`, AtomBIOS `PowerPlayInfo` at **`0x09C0E`**, **`833 bytes`**, `rev 0x07.01.00`).

## Main Table Header & OverDrive Fields (`0x000..0x04F`)

| Table Offset | VBIOS ROM Offset | Raw Bytes | Field Name | Decoded Value (RX 580 2048SP) |
| :--- | :--- | :--- | :--- | :--- |
| `0x000` | `0x09C0E` | `41 03 07 01 00` | `AtomCommonTableHeader` | `size = 833 B`, `rev = 0x07.01.00` |
| `0x013` | `0x09C21` | `00 00 06 01` | `ulPlatformCaps` | `0x01060000` (`17:DISABLE_VOLTAGE_ISLAND`, `18:ODN_AC_SUPPORT`, `24:FALCON_QUICK_TRANSITION`) |
| `0x017` | `0x09C25` | `40 0D 03 00` | `ulMaxODEngineClock` | `200000` (`2000 MHz` OverDrive GFX ceiling) |
| `0x01B` | `0x09C29` | `E8 6E 03 00` | `ulMaxODMemoryClock` | `225000` (`2250 MHz` OverDrive Mem ceiling) |
| `0x01F` | `0x09C2D` | `32 00` | `usPowerControlLimit` | `+50%` OverDrive Power slider limit |
| `0x025` | `0x09C33` | `A1 02` | `usFanTableOffset` | `0x02A1` (`ATOM_Tonga_Fan_Table`) |
| `0x027` | `0x09C35` | `98 02` | `usStateArrayOffset` | `0x0298` (`ATOM_Tonga_State_Array`) |
| `0x02B` | `0x09C39` | `B5 01` | `usMclkDependencyTableOffset` | `0x01B5` (`mclk_dep_table`) |
| `0x02D` | `0x09C3B` | `3B 01` | `usSclkDependencyTableOffset` | `0x013B` (`sclk_dep_table`) |
| `0x02F` | `0x09C3D` | `77 00` | `usVddcLookupTableOffset` | `0x0077` (`vddc_lookup_table`) |
| `0x031` | `0x09C3F` | `F9 00` | `usVddgfxLookupTableOffset` | `0x00F9` (`vddgfx_lookup_table`) |
| `0x033` | `0x09C41` | `DE 01` | `usMMDependencyTableOffset` | `0x01DE` (UVD/VCE multimedia DPM table) |
| `0x035` | `0x09C43` | `06 03` | `usVCEStateTableOffset` | `0x0306` (`ATOM_Tonga_VCE_State_Table`) |
| `0x039` | `0x09C47` | `D1 02` | `usPowerTuneTableOffset` | `0x02D1` (`ATOM_Polaris_PowerTune_Table`) |
| `0x03D` | `0x09C4B` | `20 03` | `usPPMTableOffset` | `0x0320` (`ATOM_Tonga_PPM_Table`) |
| `0x03F` | `0x09C4D` | `3A 03` | `usCacTdpTableOffset` | `0x033A` (`ATOM_PowerTune_Table` CAC/TDP) |
| `0x04B` | `0x09C59` | `00 00` | `usHardLimitTableOffset` | `0x0000` (absent on `rev 0x07.01.00`) |

## Sub-Table Offset Maps (RX 580 2048SP)

### 1. Voltage & Clock DPM Tables
- **`0x0077` (`VBIOS 0x09C85`) — `vddc_lookup_table`**: 16 entries (`8 B` stride); physical rails span **`750..1150 mV`** (entries `>= 0xFF00` such as `0xFF01..0xFF08` are AVFS/DPM virtual voltage indices).
- **`0x013B` (`VBIOS 0x09D49`) — `sclk_dep_table` (`15 B` stride, `ulSclk` at `+3..+7`)**: 8 DPM states = **`300, 588, 952, 1000, 1063, 1168, 1208, 1244 MHz`**.
- **`0x01B5` (`VBIOS 0x09DC3`) — `mclk_dep_table` (`13 B` stride, `VDDCI` `+1..+3`, `MVDD` `+5..+7`, `ulMclk` `+7..+11`)**:
  - `DPM 0`: **`300 MHz`** (`VDDCI 900 mV` / `MVDD 1000 mV`)
  - `DPM 1`: **`1000 MHz`** (`VDDCI 900 mV` / `MVDD 1000 mV`)
  - `DPM 2`: **`1750 MHz`** (`VDDCI 900 mV` / `MVDD 1000 mV`)

### 2. `ATOM_Tonga_Fan_Table` (`rev 9` @ `0x02A1` / `VBIOS 0x09EAF`)

Raw bytes (`0x2A1..0x2C4`):
`09 03 A0 0F 64 19 34 21 D0 07 A0 0F 70 17 94 2A 01 64 00 E4 12 98 08 78 63 01 00 4B 23 64 00 64 00 64 00 64`

| Sub-Offset | Table Offset | VBIOS Offset | Field | Raw Value | Decoded Value |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `+0x00` | `0x02A1` | `0x09EAF` | `ucRevId` | `0x09` | Revision `9` (`ATOM_Tonga_Fan_Table`) |
| `+0x01` | `0x02A2` | `0x09EB0` | `ucTHyst` | `3` | `3 °C` hysteresis |
| `+0x02` | `0x02A3` | `0x09EB1` | `usTMin` | `4000` | `40.00 °C` |
| `+0x04` | `0x02A5` | `0x09EB3` | `usTMed` | `6500` | `65.00 °C` |
| `+0x06` | `0x02A7` | `0x09EB5` | `usTHigh` | `8500` | `85.00 °C` |
| `+0x08` | `0x02A9` | `0x09EB7` | `usPWMMin` | `2000` | **`20.00%`** minimum fan PWM |
| `+0x0A` | `0x02AB` | `0x09EB9` | `usPWMMed` | `4000` | `40.00%` medium fan PWM |
| `+0x0C` | `0x02AD` | `0x09EBB` | `usPWMHigh` | `6000` | `60.00%` high fan PWM |
| `+0x0E` | `0x02AF` | `0x09EBD` | `usTMax` | `10900` | `109.00 °C` thermal shutdown |
| `+0x10` | `0x02B1` | `0x09EBF` | `ucFanControlMode` | `1` | PWM control mode `1` |
| `+0x11` | `0x02B2` | `0x09EC0` | `usFanPWMMax` | `100` | `100%` max PWM |
| `+0x13` | `0x02B4` | `0x09EC2` | `usFanOutputSensitivity` | `4836` | `4836` sensitivity |
| `+0x15` | `0x02B6` | `0x09EC4` | `usMaxFanRPM` | `2200` | **`2200 RPM`** max fan speed |
| `+0x17` | `0x02B8` | `0x09EC6` | `ulMinFanSCLKAcousticLimit` | `91000` | **`910 MHz`** acoustic SCLK target |
| `+0x1B` | `0x02BC` | `0x09ECA` | `ucTargetTemperature` | `75` | **`75 °C`** target GPU temperature |
| `+0x1C` | `0x02BD` | `0x09ECB` | `ucMinimumPWMLimit` | `35` | `35%` minimum PWM limit |

### 3. `ATOM_Polaris_PowerTune_Table` (`rev 4` @ `0x02D1` / `VBIOS 0x09EDF`)
- **`+0x05` (`0x02D6` / `VBIOS 0x09EE4`)**: `usTDC = 92` (**`92 A`**)
- **`+0x2D` (`0x02FE` / `VBIOS 0x09F0C`)**: `usMaximumPowerDeliveryLimit = 144` (**`144 W`**)
