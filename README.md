# NaviTune (`navitune`)

Cross-platform desktop GUI and headless CLI tool for inspecting, auditing, and tuning AMD Radeon **PowerPlayTables (`PPTable`)** and **VBIOS ROMs** across **Polaris (`SMU 8.0`)**, **Vega 10 (`SMU 9.0`)**, **Vega 20 (`SMU 11.0`)**, **RDNA 1 (`V4_5`)**, **RDNA 2 (`V4_9`)**, and **RDNA 3 (`V4_12`)**.

## Key Features
- **Full 6-Generation Radeon Support**: Decodes and round-trips standalone `.bin`/`.ppt`/`.pp_table` dumps, MorePowerTool `.mpt` profiles, Windows `.reg` exports, OpenCore `.plist` blobs, and full `.rom` VBIOS images across Polaris (`384 B` / `833 B`), Vega 10 (`512 B`), Vega 20 (`1534 B` & `1924 B`), RDNA 1 (`1674 B` & `1924 B`), RDNA 2 (`2470 B` & `2690 B`), and RDNA 3 (`5424 B`).
- **Safe VBIOS & PowerPlay Editing**: Tracks every edit against its factory baseline with live hover struct-offset and `+Δ` / `-Δ` delta tooltips, validates electrical/thermal limits (`Safe Tune`), resolves all 6 architectures' byte offsets to struct field names (`Native Offset`, `Struct Window`, `Diff & Save`, `--diff`), and updates the AtomBIOS `0x21` Option ROM checksum while preserving `$PSP` firmware directories (direct hardware SPI flashing is intentionally excluded to prevent RSA `$PSP` bricks).
- **One-Click macOS RX 6600 (`Navi 23`) `500 MHz` Unlock & `Quick Export ▾`**: Built-in preset in **Advance -> macOS** that clears `VCN_PG` (Bit `22`) and sets the `AMDRadeonX6000`-compatible Navi 23 DPM envelope (`300..2350 MHz` DPM / `2450 MHz` OD GFX, `1050 mV`, `100 W` PPT, `100 A` TDC, safe fan hysteresis), paired with a bottom-bar **`Quick Export ▾`** menu for instant OpenCore `.plist`, Injector `.kext`, or Base64/Hex export.
- **Cross-Platform OS Integration**:
  - **macOS (Hackintosh / eGPU)**: Export OpenCore `DeviceProperties` `.plist` (`PP,PhmSoftPowerPlayTable`), copy Base64 or continuous no-space Hex (`ProperTree` / `OCC`), generate codeless injector `.kext` bundles, or compile ACPI `SSDT-sPPT.aml`.
  - **Windows**: Native UAC Administrator manifest (`requireAdministrator`), console-free GUI release builds with `AttachConsole` CLI fallback, real-time AMD ADL (`atiadlxx.dll`) + PDH sensor telemetry, direct Registry (`HKLM` `PP_PhmSoftPowerPlayTable`) write/delete, and MorePowerTool `.mpt` (with PCI Device ID header) & `.reg` import/export.
    - **Linux (Ubuntu / Debian)**: Read/write `amdgpu` `sysfs` `pp_table` overrides with `systemd` boot persistence (`xppt-pptable.service`), `ppfeaturemask` (`0x4000`) & `Config Sync` checks, `hwmon` (`amdgpu`) sensor + Zero RPM telemetry, portal-safe `zenity`/`kdialog` file dialogs, and `.deb` packaging (`build_linux.sh`).
- **Headless CLI (`navitune`)**: Inspect, audit UI-vs-struct offset parity, diff two ROMs/tables, and apply calibrated presets (`quiet-undervolt`, `balanced-oc`, `auto-fix`) directly from the terminal.

## Quick Start

### Desktop GUI
```bash
cargo run --release
```

### Headless CLI
```bash
# Inspect VBIOS headers, $PSP directory, checksum, clocks, voltages & limits
cargo run --release -- --inspect target/Navi23.rom

# Audit UI-to-binary struct parity (0 mismatches expected)
cargo run --release -- --audit target/stable.mpt
cargo run --release -- --audit target/Navi23.rom --json

# Compare two VBIOS ROMs or PowerPlay profiles byte-for-byte with struct field names
cargo run --release -- --diff target/Navi23.rom target/stable.mpt

# Apply a calibrated preset and export patched .rom / .mpt / .plist / .kext / .reg
cargo run --release -- --preset auto-fix target/Navi23.rom --export-rom patched.rom --export-mpt patched.mpt
```

### Build & Test
```bash
cargo clippy --all-targets -- -D warnings && cargo test
```
Platform packaging scripts are included in the repository root (`build_macos.sh`, `build_linux.sh`, `build_windows.ps1`, `build_linux_from_macos.sh`, `build_window_from_macos.sh`).

## Documentation
- [User Guide](doc/USER_GUIDE.md) — Loading ROMs/profiles, factory baseline tracking, Safe Tune, and Pre-Save Diff
- [Architecture Overview](doc/ARCHITECTURE.md) — Single-source-of-truth offset maps and module boundaries
- [Linux & Ubuntu Integration Guide](doc/LINUX_GUIDE.md) — `amdgpu.ppfeaturemask`, `systemd` service, `hwmon` telemetry & `.deb` packaging
- [Deployment & Cross-Platform Build](doc/DEPLOYMENT.md) — Packaging `.app`, Linux binaries, and Windows `.exe`
- Silicon Binary Layouts: [Polaris](doc/POLARIS_PPTABLE.md) · [Vega 10 & 20](doc/VEGA_PPTABLE.md) · [RDNA 1](doc/RDNA1_PPTABLE.md) · [RDNA 2](doc/RDNA2_PPTABLE.md) · [RDNA 3](doc/RDNA3_PPTABLE.md)

## Project Structure

```text
NaviTune/
├── src/
│   ├── arch/                  # Silicon binary structs, header detection & SMU feature masks
│   │   ├── detect.rs          # PptArch detection, capability matrix & single-source offset labels
│   │   ├── polaris.rs         # Polaris SMU 8.0 (rev 0x07, 384 B / 833 B)
│   │   ├── vega.rs            # Vega 10 (rev 0x08, 512 B) & Vega 20 (rev 0x0B, 1534 B / 1924 B)
│   │   ├── rdna1.rs           # RDNA 1 V4_5 & compact 1674 B (Navi 10 / 12 / 14)
│   │   ├── rdna2.rs           # RDNA 2 V4_9 (Navi 21 / 22 / 23 / 24, 2470 B / 2690 B)
│   │   ├── rdna3.rs           # RDNA 3 V4_12 (Navi 31 / 32 / 33, 5424 B)
│   │   └── smu_features.rs    # SMU 11 / 13 FeaturesToRun bitmask definitions
│   ├── cli/                   # Headless CLI subcommands (`navitune --inspect/--audit/--diff/--preset`)
│   │   ├── mod.rs             # CLI argument router & unified file loader
│   │   ├── inspect.rs         # `--inspect`, `--dump-all`, and `--help` output
│   │   ├── audit.rs           # `--audit` parity checker & `--diff` field-resolved comparator
│   │   └── export.rs          # `--export-bin`, `--export-oc`, `--export-reg`, and `--preset`
│   ├── io/                    # Bounded file I/O, VBIOS ROM patching & format codecs
│   │   ├── mod.rs             # 64 MiB bounded reader & atomic file writer
│   │   ├── import.rs          # `.mpt`, `.reg`, `.plist` (UTF-8/UTF-16/BOM) & raw table parsers
│   │   ├── export.rs          # Unified `.bin`, `.rom`, `.mpt`, `.reg`, and OpenCore `.plist` serializers
│   │   ├── vbios.rs           # AtomBIOS PowerPlay extraction, `$PSP` discovery & `0x21` checksum
│   │   └── ppt/               # Bidirectional UI <-> binary PPTable codecs per generation
│   │       ├── mod.rs         # Shared loader, RDNA 2 codec & save/export dispatcher
│   │       ├── polaris.rs     # Polaris load & sync
│   │       ├── vega.rs        # Vega 10 & Vega 20 load & sync
│   │       ├── rdna1.rs       # RDNA 1 (1674 B & 1924 B) load & sync
│   │       └── rdna3.rs       # RDNA 3 (incl. rev >= 0x16 Navi 33) load & sync
│   ├── platform/              # Host OS integration & live GPU telemetry
│   │   ├── advance_os/        # macOS `.kext`/ACPI `.aml`, Windows Registry & Linux sysfs helpers
│   │   ├── gpu_telemetry.rs   # Real-time clocks, temp, power & fan sensor polling
│   │   └── macos_ioreg.rs     # macOS IORegistry GPU & live PowerPlayTable discovery
│   ├── tabs/                  # Modular egui editor tabs
│   │   ├── features.rs        # OverDrive & SMU feature flags + chip/header verification
│   │   ├── oc_limits.rs       # OverDrive clock, power, fan & acoustic ceilings
│   │   ├── power.rs           # Socket PPT, TDC, ULV & memory DPM voltages
│   │   ├── frequency.rs       # GFX, SoC, FCLK, UCLK & PCIe/LCLK clock domains
│   │   ├── curve.rs           # V/F curve coefficients, thermal limits & 2D drag-plot
│   │   ├── fan.rs             # Fan controller, Zero RPM hysteresis & 5-point curve drag-plot
│   │   ├── advance*.rs        # macOS, Windows, and Linux OS injection tabs
│   │   ├── safe_tune.rs       # Silicon-calibrated electrical/thermal safety checks
│   │   ├── more*.rs           # Secondary PPT1–PPT3 limits & throttler temperature windows
│   │   ├── native_offset.rs   # Single-source-of-truth PPTable & VBIOS hex offset map
│   │   ├── dump_audit.rs      # Full system & decoded variable dump
│   │   └── log*.rs            # Debug capture log, offset parity audit & Struct Window
│   ├── ui/                    # Reusable UI widgets, plots, presets & modals
│   │   ├── cards.rs           # Adaptive two-column card grid & factory-tracked inputs
│   │   ├── curve_plot.rs      # Interactive 2D Fan & V/F curve drag-plots
│   │   ├── more_popups.rs     # OpenCore Base64 & continuous no-space Hex modal
│   │   ├── safe_presets.rs    # Quiet Undervolt, Balanced OC & Auto-Fix Risks engines
│   │   ├── save_diff_modal.rs # Pre-Save Binary Diff & VBIOS Checksum Audit modal
│   │   └── theme.rs           # Theme presets, window persistence & Agave Nerd Font loader
│   ├── app.rs                 # `PowerPlayApp` state & tunable validation
│   └── main.rs                # Desktop GUI entrypoint & main shell
├── tests/                     # Integration & real-VBIOS round-trip test suites
│   ├── offset_audit_parity.rs # CLI/UI parity & byte-exact `.bin/.mpt/.plist/.reg/.kext` tests
│   ├── pptable_roundtrip.rs   # Multi-generation codec & offset label unit tests
│   └── real_vbios_roundtrip.rs# Real VBIOS `.rom` extraction & checksum tests
├── doc/                       # Architecture guides & per-generation PPTable specs
└── Cargo.toml
```
