# NaviTune User Guide

## Loading a PPTable or VBIOS ROM
- **Open File**: Load a standalone binary PowerPlayTable (`.bin` / `.ppt`), a MorePowerTool profile (`.mpt`), a Windows Registry export (`.reg`), an OpenCore DeviceProperties `.plist`, or a full GPU VBIOS ROM (`.rom`, e.g., `rx580.rom`, `W5500.rom`, `Navi23.rom`, or `Navi33.rom`).
- **Automatic VBIOS Extraction**: When opening a `.rom` image, NaviTune locates the AtomBIOS `PowerPlayInfo` offset automatically (e.g., `0x09C0E` on RX 580) and preserves all surrounding VBIOS bytes when saving back to `.rom`.

## Editing & Factory Baseline Tracking
- **Unmodified Fields**: Rendered with a neutral slate border (`#878E9A`).
- **Modified Fields & Checkboxes**: Highlighted in crimson (`#F0465A`) with a `●` marker on modified checkboxes; hover over any field box or checkbox to inspect its **struct field tag**, **byte offset**, **factory default value**, and live **`+Δ` / `-Δ` delta** (also mirrored in the bottom status footer).
- **Adaptive Architecture Cards**: Cards that do not exist on the loaded GPU generation are automatically hidden, expanding active cards to full width or showing a framed architectural summary card (`draw_card`) explaining where those controls live on that silicon.

## Verification & Export
- **Safe Tune**: Validates GFX/Memory clocks, voltage ordering (`Vmin <= Vmax` when `Vmax > 0`), power/TDC limits, thermal targets, and SMU safety flags for the detected architecture.
- **Log / Audit Tabs**: Inspect the full system dump (`Dump Audit`), live VBIOS byte offsets (`Native Offset`), and struct layout (`Struct Window`).
- **Quick Export ▾ (Bottom Action Bar)**: One-click dropdown next to **Diff & Save** for exporting OpenCore `DeviceProperties.plist` (`PP,PhmSoftPowerPlayTable`), `RadeonPowerPlayInjector.kext`, or copying Base64 / continuous no-space Hex from any tab.
- **Advance (macOS / Linux / Windows)**:
  - **macOS RX 6600 (`Navi 23`) `500 MHz` Unlock**: Click **Apply RX 6600 (Navi 23) 500 MHz Fix** in **Advance -> macOS** to disable `VCN_PG` (SMU Bit `22`) and align GFX clocks (`300..2350 MHz` DPM / `2450 MHz` OD), voltage ceiling (`1050 mV`), power/TDC (`100 W` / `100 A`), and fan hysteresis (`45°C -> 52°C -> 75°C`) with macOS `AMDRadeonX6000` Navi 23 expectations so the card boosts normally instead of locking at `500 MHz`.
  - **Software Injection vs. Hardware Flashing**: Direct SPI hardware VBIOS flashing (`amdvbflash` execution) is intentionally omitted to avoid bricking RSA-signed (`$PSP`) RDNA 2/3 cards; use OpenCore `.plist` / `.kext` / `.aml` injection on macOS, `.reg` / `.mpt` on Windows, or `sysfs` `pp_table` on Linux.

## Headless CLI Reference (`navitune`)
- **Inspect & Dump**: `navitune --inspect <file.rom|file.bin>` or `navitune --dump-all <file> [--json]`
- **Parity & Binary Diff**: `navitune --audit <file.rom|file.bin> [--json]` and `navitune --diff <file_a> <file_b> [--json]`
- **One-Click Safe Presets**: `navitune --preset <quiet-undervolt|balanced-oc|auto-fix> <file> [--export-rom out.rom] [--export-bin out.bin] [--export-mpt out.mpt] [--export-oc out.plist] [--export-kext out.kext] [--export-reg out.reg]`
- **Direct Export**: `navitune --export-oc <file> [out.plist]`, `navitune --export-reg <file> [out.reg]`, or `navitune --export-bin <file.rom> <out.bin>`

## UI Performance & Low GPU Mode
- Click **Theme** in the top toolbar to toggle **Low GPU Mode** (enabled by default and saved in `theme.cfg`).
- **Low GPU Mode** disables blurred card/window drop-shadows, compositor transparency, and multi-frame hover animations so GPU utilization stays at ~0% idle when the window is open.
- **Live Telemetry** spawns a 1 Hz background hardware poller only while toggled on and stops the worker thread immediately when toggled off.

## Zero-Value Read-Only Protection & Safe Editing
- **Read-Only Guard (`0`-Valued & Unsupported Fields)**: Any numeric field whose baseline VBIOS value is `0` (such as unused thermal slots or unpopulated rails) and any 32-bit SMU feature mask equal to `0x00000000` (e.g., on Vega 10 workstation VBIOSes) are automatically locked **read-only** with a warning tooltip so padding or reserved bytes cannot be accidentally corrupted.

## Pre-Save Binary Diff & VBIOS Checksum Audit
- Clicking **Diff & Save (N B)** in the bottom action bar opens a live byte-level diff table resolving every modified PPTable offset and VBIOS ROM offset to its **Struct Field** name alongside old/new byte values and automatic AtomBIOS `0x21` checksum recalculation before writing to disk.
- **Automated ROM Integrity & Read-Back Verification**: Saving a `.rom` image (via **Save**, **Diff & Save -> Patch VBIOS ROM (.rom)**, or CLI `--export-rom`) automatically audits:
  1. Exact file size preservation (`orig.len() == patched.len()`) and valid `0x55AA` ROM header.
  2. AtomBIOS 8-bit modular checksum across the active ROM block (`rom[0x02] * 512`), verifying `sum mod 256 == 0x00` at offset `0x21`.
  3. Strict byte-change boundary check ensuring only bytes inside `vbios_ppt_offset..vbios_ppt_offset + ppt_len` or `0x21` were modified.
  4. Post-write disk read-back verification, displaying a confirmation modal popup (`[ROM OK | off 0x.... | csum 0x.. | .. B changed | blk .. B]`).
- **RDNA 2 ULV Voltage (`0x380..0x388`)**: Displays `UlvVoltageVminGfx` (`0x380`), `UlvVoltageVminSoc` (`0x382`), `UlvVoltageVmaxGfx` (`0x384`), and `UlvVoltageVmaxSoc` (`0x386`) in mV (`raw / 4`).

### Windows Workflow (Registry, MorePowerTool `.mpt`, & Live Telemetry)
- **Automatic UAC Elevation**: `navitune.exe` requests Administrator privileges on launch so **More -> 3. Advance -> Windows -> Write to Registry / Delete Registry Key** can write `PP_PhmSoftPowerPlayTable` directly to `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000` without manual elevation. Press **`Win + Ctrl + Shift + B`** after writing or deleting to reload the AMD graphics driver immediately.
- **Saving `.mpt` & `.reg` Files**: Save into a user subfolder (e.g. `D:\NaviTune\`) rather than a drive root (`D:\`) to avoid Windows root-directory ACL restrictions. `.mpt` exports include the `0x100`-byte MorePowerTool header and PCI Device ID so they open directly via **Load** in MorePowerTool; `.reg` exports can be double-clicked in Explorer to merge into the Windows Registry.
- **Native Live Telemetry**: Clicking **Live Telemetry** polls AMD Display Library (`atiadlxx.dll` PMLog sensors) and Windows Performance Data Helper (`pdh.dll`) for real-time GFX/Memory clocks, Edge/Hotspot temperatures, GPU load, board power, fan RPM/%, and VRAM usage.

- **Thermal Vmin Cold-Boot Guard & 5-Point Fan Curve (`0x22A`)**: `Safe Tune` verifies `Vmin Low >= Vmin High` below `TVmin` (`55°C`) to prevent 7nm FinFET cold-boot instability, while `Fan -> Fan Curve` exposes direct editing of `od_fan_curve_min[0..4]` (`0x22A..0x24A`) on RDNA 2.
