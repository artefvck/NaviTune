# NaviTune (`pptool`) — Cross-Platform RDNA 2 Soft PowerPlay Table Editor

**NaviTune** (CLI/binary: `pptool`) is a native Rust GUI application (`eframe` / `egui`) for inspecting, decoding, editing, validating, and exporting **AMD Radeon RDNA 2 (`Navi 21` / `Navi 22` / `Navi 23` / `Navi 24`)** Soft PowerPlay Tables (`PP_PhmSoftPowerPlayTable`, `AtomSmcDpmInfoV4_9` — 2,470 bytes) across **macOS**, **Windows**, and **Linux**.

---

## Key Features

### 1. Native Rust RDNA 2 Binary Engine (`AtomSmcDpmInfoV4_9`)
* **Zero-Copy Packed Struct Mapping**: Directly maps the 2,470-byte `PP_PhmSoftPowerPlayTable` layout (`#[repr(C, packed)]`) with compile-time `core::mem::offset_of!` offset verification.
* **Supported Parameter Groups**:
  * **Features & SMU Flags**: OverDrive 8 feature flags (`od_feature_flags[0..16]`), `PPTable Features` 64-bit bitmask (`DPM_GFXCLK`, `GFXOFF`, `BACO`, `VCN_PG`, etc.), and Power Modes (`Quiet`, `Balanced`, `Turbo`, `Rage`).
  * **OverDrive Limits**: GFX Clock (`Min/Max`), Memory Clock (`Min/Max`), Power Limit (`%`), Fan Speed (`RPM`), Acoustic Limit (`RPM`), and Fan Target Temperature (`°C`).
  * **Power & Voltage**: Socket Power Limit (`W`), TDC Limits (`GFX` / `SoC` in `A`), AC Power Limits, ULV `Vmin`, GFX/SoC voltage rails (`0.25 mV` native resolution), `Tvmin` thermal thresholds, `DcBtc`, and `DcTol`.
  * **Frequencies & Memory**: GFXCLK (`Min`, `Max`, and secondary mirror), SoC, FCLK, VCLK, DCLK, DCEFCLK, DISPCLK, PIXCLK, PHYCLK, DTBCLK, UCLK DPM states (`0..3`), `VDDCI`, `MVDD`, PCIe Gen/Lane/LCLK states.
  * **V/F Curve & Thermal**: Static voltage curve coefficients (`a`, `b`, `c`) and `TemperatureLimit` array (Edge, Hotspot, Memory, VR GFX/Mem/SoC, Liquid).
  * **Fan Control**: Min PWM (`%`), Acoustic Limit, Throttling/Max RPM, Target Temp, Zero RPM enable, Fan Stop/Start hysteresis temperatures, and OverDrive fan ranges.

### 2. Multi-Format Decode, Edit & Encode Round-Trip

| Format | Extension | Load / Decode | Save / Encode | Notes |
| :--- | :--- | :---: | :---: | :--- |
| **Raw PowerPlay Binary** | `.bin`, `.dat`, `.ppt`, `pp_table` | Yes | Yes | Direct 2,470-byte `AtomSmcDpmInfoV4_9` buffer |
| **AMD VBIOS ROM** | `.rom` | Yes | Yes | Extracts via ATOM `MasterDataTable`, patches in-place, recalculates 8-bit ROM checksum, & inspects `$PSP` / RSA-2048 signatures |
| **MorePowerTool** | `.mpt` | Yes | Yes | Parses & generates XML `<PowerPlayTable>` hex payloads |
| **Windows Registry** | `.reg` | Yes | Yes | Parses & generates UTF-16LE/UTF-8 `PP_PhmSoftPowerPlayTable=hex:...` files |
| **OpenCore Property List** | `.plist` | Yes | Yes | Parses & generates Base64 `<data>` entries under `PP_PhmSoftPowerPlayTable` |

### 3. Cross-Platform OS Deployment (`Advance` Tab)
* **macOS (OpenCore / Hackintosh)**:
  * **Copy OpenCore Base64/Hex**: Detached OS window offering one-click Base64 or continuous raw Hex clipboard copy.
  * **Export `.plist`**: Saves OpenCore `DeviceProperties` `.plist`.
  * **Export ACPI `.aml` / `.dsl`**: Auto-detects the live GPU `IOACPIPlane` path (e.g., `_SB_.PCI0.GPP0.VGA` or `_SB_.PCI0.PEG0.PEGP.GFX0`) from `IORegistry`, generates `SSDT-sPPT.dsl` with `_OSI ("Darwin")` guard, and compiles directly to `SSDT-sPPT.aml` via `iasl` (with automatic missing-`iasl` notification).
  * **Export Injector `.kext`**: Generates a codeless `RadeonPowerPlayInjector.kext` (`Contents/Info.plist`) to merge `PP_PhmSoftPowerPlayTable` onto `IOPCIDevice` before `AMDRadeonX6000.kext` initializes.
* **Windows**:
  * **Write to Registry / Delete Registry Key**: Writes or removes `PP_PhmSoftPowerPlayTable` (`REG_BINARY`) directly in `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000`.
  * **Export `.reg` / `.mpt`** & **Copy Raw Hex (No Space)**.
  * Uses `eframe::Renderer::Glow` with `vsync: true` and `multisampling: 0` on Windows to prevent swapchain flicker on Windows.
* **Linux (`amdgpu`)**:
  * **Install / Uninstall**: Applies `pp_table` live to `/sys/class/drm/cardX/device/pp_table` and installs `/etc/xppt/pp_table.bin` + `xppt-pptable.service` (`systemd`) for boot persistence, with byte-for-byte `Config Sync` verification.

### 4. Live GPU Telemetry & Struct Diagnostics
* **Live GPU Telemetry**: Non-blocking background poller displaying real-time **GFX Clock (MHz)**, **MEM Clock (MHz)**, **Core / Hotspot Temp (°C)**, **GPU Load (%)**, **Board Power (W)**, **Fan Speed (RPM / %)**, and **VRAM Usage (MB)** on macOS (`IOAccelerator`), Linux (`amdgpu` `sysfs`/`hwmon`), and Windows (PDH).
* **More -> Struct & Offset Audit**:
  * `1. Offset Audit` — Verifies every field offset against `AtomSmcDpmInfoV4_9`.
  * `2. Capture Log` — Hex/byte-level diff log.
  * `3. Safe Tune` — Hardware range and safety validation checks.
  * `4. Native Offset` — Full `=== NATIVE RUST OFFSET DEBUG ===` dump with raw little-endian bytes and decoded values.
  * `Struct Window` & `Throttler` — Detached native windows for inspecting full struct layouts and thermal limits.

---

## Project Tree

```text
NaviTune/
├── Cargo.toml                  # Crate dependencies (eframe/egui, rfd) & binary definition (pptool)
├── README.md                   # Project documentation & build guide
├── build_macos.sh              # Build macOS Universal Binary (arm64 + x86_64) .app & .dmg
├── build_windows.ps1           # Build native Windows x86_64 release .zip
├── build_linux.sh              # Build native Linux release .tar.gz with navitune.desktop
├── build_window_from_macos.sh  # Cross-compile Windows x86_64 .zip from macOS (mingw-w64)
├── build_linux_from_macos.sh   # Cross-compile Linux x86_64 .tar.gz from macOS (cargo-zigbuild / cross)
└── src/
    ├── main.rs                 # Application entry point, viewport options, tab bar & telemetry strip
    ├── pptable_rdna2.rs        # Packed #[repr(C, packed)] AtomSmcDpmInfoV4_9 struct & offset_of! table
    ├── io_ppt.rs               # File load/save dispatch, UI <-> binary buffer synchronization
    ├── import_formats.rs       # Native decoders for .reg (hex), .mpt (XML), and .plist (Base64)
    ├── export_formats.rs       # Native encoders for Base64, .reg, .mpt, and OpenCore .plist
    ├── vbios.rs                # AMD VBIOS Option ROM parser, ATOM MasterDataTable, \$PSP & checksum patcher
    ├── advance_os.rs           # macOS (.plist/.aml/.kext), Windows (regedit), & Linux (sysfs/systemd) tools
    ├── gpu_telemetry.rs        # Cross-platform background GPU sensor & clock telemetry poller
    ├── macos_ioreg.rs          # Live GPU enumeration (macOS IORegistry, Linux DRM, Windows Registry)
    ├── debug_tab.rs            # Offset Audit, Capture Log, Safe Tune, Native Offset & Struct Window
    ├── more_popups.rs          # Detached native windows (Thermal Throttler Limits)
    ├── card_layout.rs          # Responsive two-column card & key-value row UI components
    └── ui_theme.rs             # High-contrast dark theme palette & modal error dialog
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
