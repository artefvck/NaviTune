# NaviTune Architecture Overview

NaviTune (`pptool`) is a cross-platform Rust + `egui`/`eframe` GPU PowerPlayTable (`PPTable`) and VBIOS ROM inspector/editor supporting **6 AMD Radeon silicon generations**:

1. **Polaris** (`SMU 8.0`, `rev 0x07`, e.g., RX 470/480/570/580/590)
2. **Vega 10** (`SMU 9.0`, `rev 0x08`, e.g., RX Vega 56/64)
3. **Vega 20** (`SMU 11.0`, `rev 0x0B`, e.g., Radeon VII / Radeon Pro VII, `1534 B` & `1924 B`)
4. **RDNA 1** (`SMU 11.0`, `rev 0x0C` / `V4_5`, e.g., RX 5500/5600/5700 XT)
5. **RDNA 2** (`SMU 11.0`, `rev 0x0F` / `V4_9`, `2470 B`, e.g., RX 6600/6700/6800/6900 XT)
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
