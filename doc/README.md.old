# NaviTune (`pptool`) — Cross-Platform RDNA 2 & RDNA 3 Soft PowerPlay Table Editor

**NaviTune** (CLI/binary: `pptool`) is a native Rust GUI application (`eframe` / `egui`) for inspecting, decoding, editing, validating, and exporting **AMD Radeon RDNA 2 (`Navi 21` / `Navi 22` / `Navi 23` / `Navi 24` — `AtomSmcDpmInfoV4_9`, 2,470 bytes)** and **RDNA 3 (`Navi 31` / `Navi 32` / `Navi 33` — `AtomSmcDpmInfoV4_12`, 5,424 bytes)** Soft PowerPlay Tables (`PP_PhmSoftPowerPlayTable`) across **macOS**, **Windows**, and **Linux**.

---

## Key Features

### 1. Native Rust RDNA 2 & RDNA 3 Binary Engine (`V4_9` & `V4_12`)
* **Zero-Copy Packed Struct Mapping**: Directly maps both the 2,470-byte `AtomSmcDpmInfoV4_9` (RDNA 2) and 5,424-byte `AtomSmcDpmInfoV4_12` (RDNA 3) `PP_PhmSoftPowerPlayTable` layouts (`#[repr(C, packed)]`) with compile-time `core::mem::offset_of!` offset verification.
* **Tab-by-Tab Workspace**:
  * **Features**: OverDrive 8 feature flags (`od_feature_flags[0..16]`), `PPTable Features` 64-bit SMU bitmask (`DPM_GFXCLK`, `GFXOFF`, `BACO`, `VCN_PG`, etc.), Power Modes, and **VBIOS ROM & Table Header Verification** (Silicon Family, PCI `[VID:DID]`, AIB Partner `[SVID:SSID]`, BIOS Version, `$PSP` security status, AtomBIOS `ATOM -> MasterData` chain, and Option ROM checksum).
  * **OC Limits**: GFX Clock (`Min/Max`), Memory Clock (`Min/Max`), Power Limit (`%`), Fan Speed (`RPM`), Acoustic Limit (`RPM`), and Power Mode limits (`Quiet`, `Balanced`, `Turbo`, `Rage`).
  * **Power**: Socket Power Limit (`PPT0` in `W`), TDC Limits (`GFX` / `SoC` in `A`), ULV `Vmin`, GFX/SoC voltage rails (`0.25 mV` native resolution), `Tvmin` thermal thresholds, `DcBtc`, and `DcTol`.
  * **Frequency**: GFXCLK (`Min`, `Max`, and secondary mirror), SoC, FCLK, VCLK, DCLK, DCEFCLK, DISPCLK, PIXCLK, PHYCLK, DTBCLK, UCLK DPM states (`0..3`), `VDDCI`, `MVDD`, PCIe Gen/Lane/LCLK states.
  * **Curve & Fan**: Static V/F voltage curve coefficients (`a`, `b`, `c`), Min PWM (`%`), Acoustic Limit, Throttling/Max RPM, Target Temp, Zero RPM enable, and Fan Stop/Start hysteresis temperatures.
  * **Safe Tune & More**: Calibrated safety guardrail checks (`Safe Tune`) plus inline cards (`More`) for **Secondary Power Limits (`PPT1..3`)** and **Thermal Throttler Limits** (`Edge`, `Hotspot`, `Mem`, `VR GFX/Mem/SoC`, `Liquid`).
  * **Log**: 5-view diagnostic suite (**Offset Audit**, **Capture Log**, **Native Offset**, **Struct Window**, and **Dump Audit**) with RDNA 2 / RDNA 3 struct offset inspection.

### 2. Multi-Format Decode, Edit & Encode Round-Trip

| Format | Extension | Load / Decode | Save / Encode | Notes |
| :--- | :--- | :---: | :---: | :--- |
| **Raw PowerPlay Binary** | `.bin`, `.dat`, `.ppt`, `pp_table` | Yes | Yes | Direct `AtomSmcDpmInfoV4_9` (2,470 B) or `AtomSmcDpmInfoV4_12` (5,424 B) buffer |
| **AMD VBIOS ROM** | `.rom` | Yes | Yes | Extracts via ATOM `MasterDataTable` / `$PSP` scan, patches in-place, recalculates 8-bit Option ROM checksum, & verifies `$PSP` signatures |
| **MorePowerTool** | `.mpt` | Yes | Yes | Parses & generates binary/XML MorePowerTool profiles (RDNA 2 Windows driver compatible) |
| **Windows Registry** | `.reg` | Yes | Yes | Parses & generates UTF-16LE/UTF-8 `PP_PhmSoftPowerPlayTable=hex:...` files |
| **OpenCore Property List** | `.plist` | Yes | Yes | Parses & generates Base64 `<data>` entries under `PP_PhmSoftPowerPlayTable` |

### 3. Cross-Platform OS Deployment (`Advance` Tab)
* **macOS (OpenCore / Hackintosh)**: Copy OpenCore Base64/Hex, export `DeviceProperties` `.plist`, auto-detect live `IOACPIPlane` path and compile `SSDT-sPPT.aml` via `iasl`, or generate a codeless `RadeonPowerPlayInjector.kext`.
* **Windows**: Write or delete `PP_PhmSoftPowerPlayTable` (`REG_BINARY`) in `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000`, export `.reg` / `.mpt`, or copy continuous raw Hex.
* **Linux (`amdgpu`)**: Apply `pp_table` live to `/sys/class/drm/cardX/device/pp_table` and manage `/etc/xppt/pp_table.bin` + `xppt-pptable.service` (`systemd`) persistence with byte-for-byte `Config Sync` verification.

---

## Project Tree

```text
NaviTune/
├── Cargo.toml                      # Crate dependencies (eframe/egui, rfd) & binary definition (pptool)
├── LICENSE                         # MIT License
├── README.md                       # Project documentation & build guide
├── build_macos.sh                  # Build macOS Universal Binary (arm64 + x86_64) .app & .dmg
├── build_windows.ps1               # Build native Windows x86_64 release .zip
├── build_linux.sh                  # Build native Linux release .tar.gz with navitune.desktop
├── build_window_from_macos.sh      # Cross-compile Windows x86_64 .zip from macOS (mingw-w64)
├── build_linux_from_macos.sh       # Cross-compile Linux x86_64 .tar.gz from macOS (cargo-zigbuild / cross)
└── src/
    ├── main.rs                     # Application entry point, viewport options, tab bar & telemetry strip
    ├── pptable_rdna2.rs            # Packed #[repr(C, packed)] AtomSmcDpmInfoV4_9 (RDNA 2) struct & offsets
    ├── pptable_rdna3.rs            # Packed #[repr(C, packed)] AtomSmcDpmInfoV4_12 (RDNA 3) struct & offsets
    ├── smu_features.rs             # SMU feature bit definitions & descriptions
    ├── io_ppt.rs                   # File load/save dispatch & UI <-> binary buffer synchronization
    ├── import_formats.rs           # Native decoders for .reg (hex), .mpt, and .plist (Base64)
    ├── export_formats.rs           # Unified encoders for Base64, .reg, .mpt, .rom, and OpenCore .plist
    ├── vbios.rs                    # AMD VBIOS Option ROM parser, ATOM MasterDataTable, \$PSP & checksum patcher
    ├── advance_os.rs               # macOS (.plist/.aml/.kext), Windows (regedit), & Linux (sysfs/systemd) backend
    ├── gpu_telemetry.rs            # Cross-platform background GPU sensor & clock telemetry poller
    ├── macos_ioreg.rs              # Live GPU enumeration (macOS IORegistry, Linux DRM, Windows Registry)
    ├── more_popups.rs              # Secondary viewport hooks
    ├── card_layout.rs              # Responsive two-column card, dynamic field boxes & tooltip helpers
    ├── ui_theme.rs                 # Theme presets, high-contrast styling & modal error dialog
    └── tabs/
        ├── mod.rs                  # Tab module exports
        ├── features.rs             # OD8 feature flags, SMU bitmask, Power Modes & VBIOS header verification
        ├── oc_limits.rs            # OverDrive clock, power, fan & acoustic limits
        ├── power.rs                # PPT0, TDC, ULV Vmin, GFX/SoC voltage rails, Tvmin, DcBtc & DcTol
        ├── frequency.rs            # GFXCLK, SoC, FCLK, UCLK DPM states, VDDCI/MVDD & PCIe states
        ├── curve.rs                # Static V/F quadratic voltage curve coefficients
        ├── fan.rs                  # PWM, acoustic/throttling RPM, target temp & Zero RPM hysteresis
        ├── advance_tab.rs          # OS deployment tab switcher & OpenCore Base64/Hex copy modal
        ├── advance_macos_tab.rs    # macOS VCN mask, .plist, Injector .kext & ACPI .aml export
        ├── advance_windows_tab.rs  # Windows Registry write/delete & .reg/.mpt export
        ├── advance_linux_tab.rs    # Linux amdgpu sysfs/systemd install/uninstall & sync status
        ├── safe_tune.rs            # Calibrated hardware safety guardrail checks
        ├── more_tab.rs             # Inline secondary settings container
        ├── more_power_settings_tab.rs # Secondary power limits (PPT1..3)
        ├── more_throttler_tab.rs   # Thermal throttler temperature limits
        ├── log_tab.rs              # Diagnostic Log sub-tab navigation
        ├── log_offset_audit_tab.rs # Field-to-struct byte offset audit
        ├── log_capture_log_tab.rs  # Live runtime capture log
        ├── log_native_offset_tab.rs # Native Rust struct field & hex offset dump
        ├── log_struct_window_tab.rs # Raw struct byte/hex window viewer
        └── log_dump_audit_tab.rs   # Full binary dump & header audit exporter
```

---

## Build & Packaging

### Prerequisites
* **Rust toolchain** (`rustc` & `cargo` 1.77+ for `core::mem::offset_of!`): [rustup.rs](https://rustup.rs)
* *(Optional, macOS)* **ACPICA (`iasl`)** for compiling `SSDT-sPPT.aml`:
  ```bash
  brew install acpica
  ```

### Local Development Run
```bash
cargo run --release
```

### Release Packaging Scripts

| Target Platform | Host OS | Command | Output Artifact |
| :--- | :--- | :--- | :--- |
| **macOS Universal (`arm64` + `x86_64`)** | macOS | `./build_macos.sh` | `dist/NaviTune-<ver>-macOS-Universal.dmg` |
| **Windows (`x86_64`)** | Windows | `.\build_windows.ps1` | `dist\NaviTune-<ver>-Windows-x86_64.zip` |
| **Windows (`x86_64`)** | macOS (`mingw-w64`) | `./build_window_from_macos.sh` | `dist/NaviTune-<ver>-Windows-x86_64.zip` |
| **Linux (`x86_64` / `aarch64`)** | Linux | `./build_linux.sh` | `dist/NaviTune-<ver>-Linux-<arch>.tar.gz` |
| **Linux (`x86_64`)** | macOS (`cargo-zigbuild` / `cross`) | `./build_linux_from_macos.sh` | `dist/NaviTune-<ver>-Linux-x86_64.tar.gz` |

---

## Disclaimer
Modifying GPU PowerPlay tables, voltage limits, power limits, or VBIOS images alters hardware operating parameters outside factory defaults. Always back up your stock VBIOS / registry / EFI configuration and validate stability incrementally.

---

## License

This project is licensed under the [MIT License](LICENSE).
