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
