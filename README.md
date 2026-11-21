# NaviTune (`pptool`)

**NaviTune** (`pptool`) is a cross-platform Rust GUI utility (`eframe` / `egui`) for editing Soft PowerPlay Tables (`PP_PhmSoftPowerPlayTable`) and patching VBIOS ROMs on AMD Radeon **Vega 10**, **Vega 20**, **RDNA 1**, **RDNA 2**, and **RDNA 3** GPUs across macOS, Windows, and Linux.

## Key Features
* **Multi-Generation Packed Struct & Sub-Table Mapping**: Zero-copy mapping for **Vega 10** (`0x08` + Fan/HardLimit sub-tables), **Vega 20** (`0x0B`), **RDNA 1** (`V4_5`), **RDNA 2** (`V4_9`), and **RDNA 3** (`V4_12`).
* **Workspace Tabs**: Features, OC Limits, Power, Frequency, Curve & Fan, Safe Tune, Advance OS deployment, and live byte-offset auditing (`Log`).
* **Multi-Format Support**: Raw `.bin`, VBIOS `.rom` (with Option ROM 8-bit checksum fix), MorePowerTool `.mpt`, Windows Registry `.reg`, and OpenCore `.plist` / `SSDT-sPPT.dsl`.

## Documentation & Build
Consult the `doc/` directory (`VEGA_PPTABLE.md`, `RDNA1_PPTABLE.md`, `RDNA2_PPTABLE.md`, `RDNA3_PPTABLE.md`, `ARCHITECTURE.md`). Run `cargo test` and `cargo run --release` locally.
