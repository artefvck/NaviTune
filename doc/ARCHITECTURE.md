# NaviTune Architecture Overview

NaviTune (`pptool`) is a cross-platform Rust + `egui`/`eframe` GPU PowerPlayTable (`PPTable`) and VBIOS ROM inspector/editor supporting **6 AMD Radeon silicon generations**:

1. **Polaris** (`SMU 8.0`, `rev 0x07`, e.g., RX 470/480/570/580/590)
2. **Vega 10** (`SMU 9.0`, `rev 0x08`, e.g., RX Vega 56/64)
3. **Vega 20** (`SMU 11.0`, `rev 0x0B`, e.g., Radeon VII / Radeon Pro VII, `1534 B` & `1924 B`)
4. **RDNA 1** (`SMU 11.0`, `rev 0x0C` / `V4_5`, e.g., RX 5500/5600/5700 XT)
5. **RDNA 2** (`SMU 11.0`, `rev 0x0F..0x13` / `V4_9`, `2470 B` & `2690 B`, e.g., RX 6400/6500/6600/6700/6800/6900 XT)
6. **RDNA 3** (`SMU 13.0`, `rev 0x12+` / `V4_12`, `5424 B`, e.g., RX 7600/7700/7800/7900 XTX)

## Core Modules
- **`src/ppt_arch.rs`**: Central silicon classifier (`PptArch`) and card capability matrix (`PptCard`), controlling card visibility, safe-tune bounds, and bitmask decoding (`PHM_PlatformCaps` vs `FeaturesToRun`).
- **`src/pptable_polaris.rs` / `pptable_vega.rs` / `pptable_rdna1.rs` / `pptable_rdna2.rs` / `pptable_rdna3.rs`**: Zero-copy / padded-slice binary parsers and writers for each generation.
- **`src/vbios.rs`**: VBIOS ROM scanner (`0x55AA` PCI expansion ROM header + AtomBIOS Master Data Table `PowerPlayInfo` locator) supporting direct ROM open & surgical in-place VBIOS save.
- **`src/card_layout.rs`**: Adaptive UI layout (`two_col_cards_adaptive`, `draw_card`, `draw_arch_note_card`) that hides non-usable cards per architecture, collapses single-column tabs to 100% width, and renders framed architectural summary cards when a tab has no applicable sub-tables.
- **`src/tabs/`**:
  - `features.rs`: OverDrive & Platform Capabilities checklist with factory baseline tracking (`●` dirty indicator) + 64-bit `PPTable Features` viewport (`Vega 20` & `RDNA 1–3`).
  - `oc_limits.rs`, `power.rs`, `frequency.rs`, `curve.rs`, `fan.rs`: Architecture-gated tuning editors with factory baseline tracking (`draw_tracked_drag`).
  - `safe_tune.rs`: Architecture-aware electrical, clock, thermal, and SMU safety guard validator.
  - `log_dump_audit_tab.rs`, `log_native_offset_tab.rs`, `log_offset_audit_tab.rs`, `log_struct_window_tab.rs`: Live hex dump, VBIOS offset map, and decoded variable auditor.

## Modular Source Layout (`src/`)

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

## Zero-Byte Unmodified Round-Trip Parity
- All 6 GPU generations preserve sub-millivolt (`qmv = mV * 4`) remainders, upper reserved/flag bits, and compact sub-table boundaries so saving an untouched table yields **0 modified bytes** (`tests/offset_audit_parity.rs`).
