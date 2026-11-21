# Architecture & Binary Engine

## Overview

**NaviTune** (`pptool`) is built in pure Rust using `eframe` / `egui` for cross-platform rendering and `rfd` for native file dialogs. Rather than relying on fragile text-scraping or external Python scripts, NaviTune maps AMD `PP_PhmSoftPowerPlayTable` binary buffers directly to packed C-compatible Rust structs.

## Packed Struct Mapping

* **RDNA 2 (`src/pptable_rdna2.rs`)**: `AtomSmcDpmInfoV4_5 / AtomSmcDpmInfoV4_9` (`2,470` bytes, `SMU 11.0` — Navi 21, Navi 22, Navi 23, Navi 24).
* **RDNA 3 (`src/pptable_rdna3.rs`)**: `AtomSmcDpmInfoV4_12` (`5,424` bytes, `SMU 13.0` — Navi 31, Navi 32, Navi 33).
* Both definitions use `#[repr(C, packed)]` and compile-time `core::mem::offset_of!` assertions to guarantee exact byte alignment against AMD AtomBIOS headers.

## VBIOS Option ROM Patcher (`src/vbios.rs`)

When loading a full `.rom` image:
1. Scans the PCI Expansion ROM header, `$PSP` directory, and `ATOM` `MasterDataTable` pointer chain to locate the embedded `PP_PhmSoftPowerPlayTable`.
2. Modifies only the PowerPlay payload in-place while preserving `$PSP` firmware signatures.
3. Recalculates the 8-bit Option ROM checksum automatically upon saving.

## Responsive UI & Unit Inference (`src/card_layout.rs`, `src/main.rs`)

* **Responsive Columns**: `two_col_cards` renders side-by-side cards when viewport width is `>= 620 px` and automatically stacks into a single full-width column on narrower windows.
* **Left-Box Alignment**: Numeric `DragValue` controls (`86 px` fixed width) sit on the left with field labels immediately on the right.
* **Automatic Hardware Unit Suffixes**: `draw_tracked_drag` infers per-column and per-field units (`°C`, `MHz`, `mV`, `W`, `A`, `RPM`, `%`) for both RDNA 1, RDNA 2, and RDNA 3 tables.
* **Modified Value Tracking**: Unmodified factory values render in muted neutral text; edited values highlight in crimson with factory baseline hover tooltips and a pinned bottom status footer.

## Diagnostic Logging (`src/ui_theme.rs`)

Runtime events (`APP`, `LOAD`, `SAVE`, `GPU`, `RESET`, `SNAPSHOT`, `ERROR`) are written with UTC ISO-8601 timestamps (`[YYYY-MM-DD HH:MM:SSZ]`) and automatic 512 KB log rotation to:

```text
~/.config/navitune/debug.log
```
