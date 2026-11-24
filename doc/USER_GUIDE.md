# NaviTune User Guide

## Loading a PPTable or VBIOS ROM
- **Open File**: Load either a standalone binary PowerPlayTable (`.bin` / `.ppt`) or a full GPU VBIOS ROM (`.rom`, e.g., `rx580.rom` or `Vega20RadeonPro.251497.rom`).
- **Automatic VBIOS Extraction**: When opening a `.rom` image, NaviTune locates the AtomBIOS `PowerPlayInfo` offset automatically (e.g., `0x09C0E` on RX 580) and preserves all surrounding VBIOS bytes when saving back to `.rom`.

## Editing & Factory Baseline Tracking
- **Unmodified Fields**: Rendered with a neutral slate border (`#878E9A`).
- **Modified Fields & Checkboxes**: Highlighted in crimson (`#F0465A`) with a `●` marker on modified checkboxes; hover over any field box or checkbox to inspect its **struct field tag**, **byte offset**, and **factory default value**.
- **Adaptive Architecture Cards**: Cards that do not exist on the loaded GPU generation are automatically hidden, expanding active cards to full width or showing a framed architectural summary card (`draw_card`) explaining where those controls live on that silicon.

## Verification & Export
- **Safe Tune**: Validates GFX/Memory clocks, voltage ordering (`Vmin <= Vmax` when `Vmax > 0`), power/TDC limits, thermal targets, and SMU safety flags for the detected architecture.
- **Log / Audit Tabs**: Inspect the full system dump (`Dump Audit`), live VBIOS byte offsets (`Native Offset`), and struct layout (`Struct Window`).
- **Advance (macOS / Linux / Windows)**: Export `PP,PhmSoftPowerPlayTable` base64/hex properties for OpenCore `DeviceProperties`, `sysfs` `pp_table` overrides on Linux, or Windows registry `PP_PhmSoftPowerPlayTable` blobs.

## Pre-Save Binary Diff & VBIOS Checksum Audit
- Clicking **Diff & Save (N B)** in the bottom action bar opens a live byte-level diff table showing every modified PPTable offset, VBIOS ROM offset, old/new byte values, and automatic AtomBIOS `0x21` checksum recalculation before writing to disk.
- **RDNA 2 ULV Voltage (`0x380..0x388`)**: Displays `UlvVoltageVminGfx` (`0x380`), `UlvVoltageVminSoc` (`0x382`), `UlvVoltageVmaxGfx` (`0x384`), and `UlvVoltageVmaxSoc` (`0x386`) in mV (`raw / 4`).
