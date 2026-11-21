# NaviTune (`pptool`) Architecture

**NaviTune** (`pptool`) is a cross-platform Rust GUI application (`eframe` / `egui`) for inspecting, editing, auditing, and deploying AMD Radeon **RDNA 1**, **RDNA 2**, and **RDNA 3** Soft PowerPlay Tables (`PP_PhmSoftPowerPlayTable`) and VBIOS ROM images.

## 1. Packed Binary Models & Architecture Dispatch
* **`src/ppt_arch.rs` (`PptArch`)**: Centralized architecture detection (`Vega10`, `Vega20`, `Rdna1`, `Rdna2`, `Rdna3`) and per-family capability helpers.
* **`src/pptable_vega.rs` (`AtomVega10PowerPlayTable`, `AtomVega20PowerPlayTable`)**: Zero-copy mapping for Vega 10 (`0x08`) and Vega 20 (`0x0B`, `1,924` bytes).
* **`src/pptable_rdna1.rs` (`AtomSmcDpmInfoV4_5`, `1,924` bytes)**: Zero-copy `#[repr(C, packed)]` mapping for RDNA 1 (`Navi 10`, `Navi 12`, `Navi 14`, `SMU 11.0`, `table_format_revision` `0x0C` / `0x0D`), plus `PptArch::detect(&[u8])` unified architecture dispatch and `NaviRDNA1Chip` silicon detection.
* **`src/pptable_rdna2.rs` (`AtomSmcDpmInfoV4_9`, `2,470` bytes)**: Zero-copy `#[repr(C, packed)]` mapping for RDNA 2 (`Navi 21`, `Navi 22`, `Navi 23`, `Navi 24`, `SMU 11.0`, `table_format_revision` `0x0F`–`0x13`).
* **`src/pptable_rdna3.rs` (`AtomSmcDpmInfoV4_12`, `5,424` bytes)**: Zero-copy `#[repr(C, packed)]` mapping for RDNA 3 (`Navi 31`, `Navi 32`, `Navi 33`, `SMU 13.0`, `table_format_revision` `0x14`–`0x16`).

## 2. Import, Export & VBIOS ROM Patching
* **`src/io_ppt.rs`**: `apply_ppt_bytes` and `sync_fields_to_buffer` synchronize `PowerPlayApp` UI state with the active packed buffer (`V4_5`, `V4_9`, or `V4_12`), preserving native hardware units (`0.25 mV` voltage steps, tenths-of-°C vs °C fan target scaling, and 0-indexed PCIe Gen speed).
* **`src/import_formats.rs` & `src/export_formats.rs`**: Lossless round-trip parsers and serializers for raw `.bin`, MorePowerTool `.mpt`, Windows Registry `.reg` (UTF-8 & UTF-16LE), OpenCore `.plist` (Base64), and patched VBIOS `.rom`.
* **`src/vbios.rs`**: Locates the embedded AtomBIOS `PP_PhmSoftPowerPlayTable` inside Option ROMs, patches it in-place (`patch_ppt_into_rom`), and recalculates the PCI Option ROM 8-bit checksum (`recalc_vbios_checksum`).

## 3. Workspace UI & Live Offset Auditing (`src/tabs/`)
* **Tuning Tabs**: `features.rs`, `oc_limits.rs`, `power.rs`, `frequency.rs`, `curve.rs`, `fan.rs`, `safe_tune.rs` (calibrated per-chip guardrails), `more_tab.rs`, `more_power_settings_tab.rs`, and `more_throttler_tab.rs`.
* **Diagnostics (`log_*.rs`)**: `log_offset_audit_tab.rs` (live byte-offset diff between raw buffer and UI), `log_native_offset_tab.rs` (struct field offset map), `log_struct_window_tab.rs`, `log_capture_log_tab.rs`, and `log_dump_audit_tab.rs`.

## 4. Cross-Platform OS Integration & Verification
* **`src/macos_ioreg.rs` & `src/advance_os.rs`**: Live GPU enumeration and deployment for **macOS** (`SSDT-sPPT.dsl` + `RadeonPowerPlayInjector.kext` covering RDNA 1/2/3 PCI IDs), **Windows** (dynamic `VEN_1002` `0000..0015` registry subkey resolution), and **Linux** (`amdgpu` `pp_table` sysfs).
* **`tests/pptable_roundtrip.rs`**: Automated `cargo test` suite verifying `V4_5`, `V4_9`, and `V4_12` struct sizes, scaling round-trips, `.reg`/`.mpt`/`.plist` serialization, and Option ROM 8-bit checksum integrity.
