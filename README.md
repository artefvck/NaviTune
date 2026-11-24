# NaviTune (`pptool`)

Cross-platform Rust GUI inspector & editor for AMD Radeon PowerPlayTables (`PPTable`) and VBIOS ROMs across **Polaris (`SMU 8.0`)**, **Vega 10 (`SMU 9.0`)**, **Vega 20 (`SMU 11.0`)**, **RDNA 1 (`V4_5`)**, **RDNA 2 (`V4_9`)**, and **RDNA 3 (`V4_12`)**.

## Key Features
- **Full 6-Generation Silicon Support**: Decodes and round-trips standalone `.bin`/`.ppt` tables, MorePowerTool `.mpt` profiles, Windows `.reg` exports, OpenCore `.plist` blobs, and full `.rom` VBIOS images across Polaris (`384 B` / `833 B`), Vega 10 (`512 B`), Vega 20 (`1534 B` & `1924 B`), RDNA 1 (`1674 B` & `1924 B`), RDNA 2 (`2470 B` & `2690 B`), and RDNA 3 (`5424 B`).
- **Single-Source-of-Truth Offset Maps**: Unifies `PptArch` metadata, `collect_native_struct_entries`, `pptool --inspect`, `pptool --audit`, `pptool --diff`, `Struct Window`, and the Pre-Save Binary Diff Modal.
- **Headless CLI & Safe Presets**: Inspect, audit, diff, and apply calibrated presets (`quiet-undervolt`, `balanced-oc`, `auto-fix`) directly to `.rom` (with automatic `$PSP` discovery and `0x21` Option ROM checksum preservation), `.bin`, `.mpt`, `.plist`, or `.reg`.

## Documentation
- [Architecture Overview](doc/ARCHITECTURE.md)
- [User Guide](doc/USER_GUIDE.md)
- [Polaris PPTable (`SMU 8.0`, `rev 0x07`)](doc/POLARIS_PPTABLE.md)
- [Vega 10 & Vega 20 PPTable (`rev 0x08` / `0x0B`)](doc/VEGA_PPTABLE.md)
- [RDNA 1 PPTable (`V4_5`)](doc/RDNA1_PPTABLE.md)
- [RDNA 2 PPTable (`V4_9`)](doc/RDNA2_PPTABLE.md)
- [RDNA 3 PPTable (`V4_12`)](doc/RDNA3_PPTABLE.md)
- [Deployment & Cross-Platform Build](doc/DEPLOYMENT.md)

## Quick Start & Headless CLI
```bash
cargo clippy --all-targets -- -D warnings && cargo test
cargo run --release

# Headless CLI inspection, parity audit, diff & VBIOS preset patching
cargo run --release -- --inspect target/Navi23.rom
cargo run --release -- --audit target/stable.mpt --json
cargo run --release -- --diff target/Navi23.rom target/stable.mpt
cargo run --release -- --preset auto-fix target/Navi23.rom --export-rom patched.rom --export-mpt patched.mpt
```

## Project Structure

<pre>
├── <b>doc/</b>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/ARCHITECTURE.md">ARCHITECTURE.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/DEPLOYMENT.md">DEPLOYMENT.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/POLARIS_PPTABLE.md">POLARIS_PPTABLE.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/RDNA1_PPTABLE.md">RDNA1_PPTABLE.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/RDNA2_PPTABLE.md">RDNA2_PPTABLE.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/RDNA3_PPTABLE.md">RDNA3_PPTABLE.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/README.md">README.md</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/README.md.old">README.md.old</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/USER_GUIDE.md">USER_GUIDE.md</a>
│   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/doc/VEGA_PPTABLE.md">VEGA_PPTABLE.md</a>
├── <b>src/</b>
│   ├── <b>arch/</b>  # Silicon binary layouts & capability matrix (Polaris -> RDNA 3)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/detect.rs">detect.rs</a>  # PptArch detection, tab/card capability matrix
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/mod.rs">mod.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/polaris.rs">polaris.rs</a>  # Polaris SMU 8.0 (rev 0x07)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/rdna1.rs">rdna1.rs</a>  # RDNA 1 V4_5 (Navi 10/12/14)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/rdna2.rs">rdna2.rs</a>  # RDNA 2 V4_9 (Navi 21/22/23/24)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/rdna3.rs">rdna3.rs</a>  # RDNA 3 V4_12 (Navi 31/32/33)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/smu_features.rs">smu_features.rs</a>  # SMU feature bitmask definitions
│   │   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/arch/vega.rs">vega.rs</a>  # Vega 10 (rev 0x08) & Vega 20 (rev 0x0B) + VegaParsedState
│   ├── <b>io/</b>  # Binary load/sync, VBIOS ROM & export formats
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/io/export.rs">export.rs</a>  # macOS OpenCore .plist, Windows .reg, Linux sysfs
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/io/import.rs">import.rs</a>  # .bin, .ppt, .rom, .plist, .reg importers
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/io/mod.rs">mod.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/io/ppt.rs">ppt.rs</a>  # PPTable load & bidirectional field sync
│   │   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/io/vbios.rs">vbios.rs</a>  # AtomBIOS / PSP VBIOS extraction & 0x21 checksum patching
│   ├── <b>platform/</b>  # OS detection & live GPU telemetry
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/platform/advance_os.rs">advance_os.rs</a>  # Host OS target helpers
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/platform/gpu_telemetry.rs">gpu_telemetry.rs</a>  # Live clocks, temp, power & fan telemetry
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/platform/macos_ioreg.rs">macos_ioreg.rs</a>  # macOS IORegistry GPU discovery
│   │   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/platform/mod.rs">mod.rs</a>
│   ├── <b>tabs/</b>  # Modular UI tabs (alphabetical)
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/advance.rs">advance.rs</a>  # Multi-OS export & injection tab
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/advance_linux.rs">advance_linux.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/advance_macos.rs">advance_macos.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/advance_windows.rs">advance_windows.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/curve.rs">curve.rs</a>  # V/F curve coefficients + interactive drag-plot
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/dump_audit.rs">dump_audit.rs</a>  # Full system & decoded variable dump
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/fan.rs">fan.rs</a>  # Fan settings, Zero RPM + interactive drag-plot
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/features.rs">features.rs</a>  # OverDrive & SMU feature checklists
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/frequency.rs">frequency.rs</a>  # GFX, SoC, FCLK & Memory DPM clocks
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/log.rs">log.rs</a>  # Capture log, offset audit & struct inspector
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/log_capture.rs">log_capture.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/log_offset_audit.rs">log_offset_audit.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/log_struct_window.rs">log_struct_window.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/mod.rs">mod.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/more.rs">more.rs</a>  # Power settings & throttler windows
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/more_power_settings.rs">more_power_settings.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/more_throttler.rs">more_throttler.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/native_offset.rs">native_offset.rs</a>  # Live PPTable & VBIOS hex offset map
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/oc_limits.rs">oc_limits.rs</a>  # OverDrive ceilings
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/power.rs">power.rs</a>  # Power limit, TDC & voltage rails
│   │   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/tabs/safe_tune.rs">safe_tune.rs</a>  # Architecture-calibrated safety guidelines & presets
│   ├── <b>ui/</b>  # Reusable cards, plots, presets, modals & theme
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/cards.rs">cards.rs</a>  # Adaptive card grid, field boxes & tracked checklists
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/curve_plot.rs">curve_plot.rs</a>  # Interactive Fan & V/F curve 2D drag-plots
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/mod.rs">mod.rs</a>
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/more_popups.rs">more_popups.rs</a>  # Pop-out power & throttler dialogs
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/safe_presets.rs">safe_presets.rs</a>  # Quiet Undervolt, Balanced OC & Auto-Fix Risks
│   │   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/save_diff_modal.rs">save_diff_modal.rs</a>  # Pre-Save Binary Diff & VBIOS Checksum Audit modal
│   │   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/ui/theme.rs">theme.rs</a>  # Modern dark theme & styling
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/cli.rs">cli.rs</a>  # Headless CLI (--inspect, --export-oc, --export-reg, --export-bin)
│   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/src/main.rs">main.rs</a>  # Desktop GUI entrypoint & PowerPlayApp shell
├── <b>tests/</b>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/tests/offset_audit_parity.rs">offset_audit_parity.rs</a>
│   ├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/tests/pptable_roundtrip.rs">pptable_roundtrip.rs</a>
│   └── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/tests/real_vbios_roundtrip.rs">real_vbios_roundtrip.rs</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/build_linux.sh">build_linux.sh</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/build_linux_from_macos.sh">build_linux_from_macos.sh</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/build_macos.sh">build_macos.sh</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/build_window_from_macos.sh">build_window_from_macos.sh</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/build_windows.ps1">build_windows.ps1</a>
├── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/Cargo.toml">Cargo.toml</a>
└── <a href="https://raw.githubusercontent.com/artefvck/NaviTune/refs/heads/main/README.md">README.md</a>
</pre>

## Architecture & Verification Matrix

- **Supported AMD Radeon Generations**: Polaris (`SMU 8.0 / rev 0x07`), Vega 10 (`SMU 9.0 / rev 0x08`), Vega 20 (`SMU 11.0 / rev 0x0B`), RDNA 1 (`1674 B rev 0x0C` & `V4_5 / rev 0x0D..0x0F`), RDNA 2 (`V4_9 / rev 0x0F..0x13`), and RDNA 3 (`V4_12 / rev 0x14..0x16`).
- **Modular Codebase Layout**:
  - `src/arch/`: Generation-specific `PowerPlayTable` codecs, header detector (`detect.rs`), and overflow-safe `SMU` feature mask helpers (`smu_features.rs`).
  - `src/cli/`: Modular CLI commands split into `inspect.rs`, `export.rs`, and `audit.rs`.
  - `src/io/`: Bounded `64 MiB` file readers (`read_bounded`), atomic file writers (`write_atomic`), VBIOS AtomBIOS extractor (`vbios.rs`), and `.plist`/`.reg`/`sysfs` codecs.
  - `src/platform/`: Cross-platform telemetry (`gpu_telemetry.rs`), macOS `IORegistry` parser (`macos_ioreg.rs`), and per-OS system integration (`advance_os/{macos,windows,linux}.rs`).
  - `src/ui/` & `src/tabs/`: `eframe`/`egui` tabs, V/F & fan curve visualizer (`curve_plot.rs`), bounded Save-Diff inspector (`save_diff_modal.rs`), and safe preset validation (`safe_presets.rs`).
- **Verification**: Run `cargo clippy --all-targets -- -D warnings && cargo test && cargo check --release`.
