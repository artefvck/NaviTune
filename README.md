# NaviTune (`pptool`)

Cross-platform Rust GUI inspector & editor for AMD Radeon PowerPlayTables (`PPTable`) and VBIOS ROMs across **Polaris (`SMU 8.0`)**, **Vega 10 (`SMU 9.0`)**, **Vega 20 (`SMU 11.0`)**, **RDNA 1 (`V4_5`)**, **RDNA 2 (`V4_9`)**, and **RDNA 3 (`V4_12`)**.

## Key Features
- **Full 6-Generation Silicon Support**: Decodes and round-trips standalone `.bin`/`.ppt` tables and full `.rom` VBIOS images across Polaris (`833 B`), Vega 10, Vega 20 (`1534 B` & `1924 B`), RDNA 1, RDNA 2 (`2470 B`), and RDNA 3 (`5424 B`).
- **Architecture-Aware Adaptive UI**: Automatically hides cards and checkboxes not supported by the loaded GPU architecture, collapses single-column layouts to 100% width, and tracks every edit against its factory baseline.
- **Live VBIOS Offset Audit & Safe Tune**: Displays exact table & VBIOS ROM hex offsets (`Native Offset`, `Dump Audit`) and validates electrical/thermal limits (`Safe Tune`) before saving.

## Documentation
- [Architecture Overview](doc/ARCHITECTURE.md)
- [User Guide](doc/USER_GUIDE.md)
- [Polaris PPTable (`SMU 8.0`, `rev 0x07`)](doc/POLARIS_PPTABLE.md)
- [Vega 10 & Vega 20 PPTable (`rev 0x08` / `0x0B`)](doc/VEGA_PPTABLE.md)
- [RDNA 1 PPTable (`V4_5`)](doc/RDNA1_PPTABLE.md)
- [RDNA 2 PPTable (`V4_9`)](doc/RDNA2_PPTABLE.md)
- [RDNA 3 PPTable (`V4_12`)](doc/RDNA3_PPTABLE.md)
- [Deployment & Cross-Platform Build](doc/DEPLOYMENT.md)

## Quick Start
```bash
cargo test
cargo run --release
```

## Project Structure

```text
src/
├── main.rs                    # Desktop GUI entrypoint & PowerPlayApp shell
├── cli.rs                     # Headless CLI (--inspect, --export-oc, --export-reg, --export-bin)
├── arch/                      # Silicon binary layouts & capability matrix (Polaris -> RDNA 3)
│   ├── detect.rs              # PptArch detection, tab/card capability matrix
│   ├── polaris.rs             # Polaris SMU 8.0 (rev 0x07)
│   ├── rdna1.rs               # RDNA 1 V4_5 (Navi 10/12/14)
│   ├── rdna2.rs               # RDNA 2 V4_9 (Navi 21/22/23/24)
│   ├── rdna3.rs               # RDNA 3 V4_12 (Navi 31/32/33)
│   ├── smu_features.rs        # SMU feature bitmask definitions
│   └── vega.rs                # Vega 10 (rev 0x08) & Vega 20 (rev 0x0B) + VegaParsedState
├── io/                        # Binary load/sync, VBIOS ROM & export formats
│   ├── export.rs              # macOS OpenCore .plist, Windows .reg, Linux sysfs
│   ├── import.rs              # .bin, .ppt, .rom, .plist, .reg importers
│   ├── ppt.rs                 # PPTable load & bidirectional field sync
│   └── vbios.rs               # AtomBIOS / PSP VBIOS extraction & 0x21 checksum patching
├── platform/                  # OS detection & live GPU telemetry
│   ├── advance_os.rs          # Host OS target helpers
│   ├── gpu_telemetry.rs       # Live clocks, temp, power & fan telemetry
│   └── macos_ioreg.rs         # macOS IORegistry GPU discovery
├── tabs/                      # Modular UI tabs (alphabetical)
│   ├── advance.rs             # Multi-OS export & injection tab
│   ├── curve.rs               # V/F curve coefficients + interactive drag-plot
│   ├── dump_audit.rs          # Full system & decoded variable dump
│   ├── fan.rs                 # Fan settings, Zero RPM + interactive drag-plot
│   ├── features.rs            # OverDrive & SMU feature checklists
│   ├── frequency.rs           # GFX, SoC, FCLK & Memory DPM clocks
│   ├── log.rs                 # Capture log, offset audit & struct inspector
│   ├── more.rs                # Power settings & throttler windows
│   ├── native_offset.rs       # Live PPTable & VBIOS hex offset map
│   ├── oc_limits.rs           # OverDrive ceilings
│   ├── power.rs               # Power limit, TDC & voltage rails
│   └── safe_tune.rs           # Architecture-calibrated safety guidelines & presets
└── ui/                        # Reusable cards, plots, presets, modals & theme
    ├── cards.rs               # Adaptive card grid, field boxes & tracked checklists
    ├── curve_plot.rs          # Interactive Fan & V/F curve 2D drag-plots
    ├── more_popups.rs         # Pop-out power & throttler dialogs
    ├── safe_presets.rs        # Quiet Undervolt, Balanced OC & Auto-Fix Risks
    ├── save_diff_modal.rs     # Pre-Save Binary Diff & VBIOS Checksum Audit modal
    └── theme.rs               # Modern dark theme & styling
```
