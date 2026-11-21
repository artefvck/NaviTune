# Tab-by-Tab User Guide

## Top Toolbar & Status Footer

* **Open / Save / Reset**: Load `.rom`, `.bin`, `.reg`, `.mpt`, `.plist`, or `pp_table` files; export patched binaries; or reset all fields to defaults.
* **Window Title & GPU Selector**: Displays the active filename in the OS title bar while keeping live hardware GPU enumeration (`IORegistry` / `DRM` / `Registry`) in the top selector.
* **Bottom Status Footer**: Displays active architecture (`RDNA 2` or `RDNA 3`), live hovered/dragged field name and hex offset, and verification status.

## Workspace Tabs

### 1. Features
* **OverDrive Features**: Toggle `od_feature_flags[0..16]` capabilities (`GPU Clock Limit`, `Zero RPM`, `Fan Curve`, `Power Limit`, `Timing Tuning`, `Power Mode`, etc.).
* **PPTable Features**: Inspect and toggle the 64-bit `FeaturesToRun` SMU feature mask (`DPM_GFXCLK`, `GFXOFF`, `BACO`, `VCN_PG`, etc.).
* **VBIOS & Table Header**: Displays `Loaded Source`, `VBIOS ROM` PCI IDs, `Silicon Family`, subsystem IDs, and Option ROM checksum status.

### 2. OC Limits
* **RDNA 2**: Configures `Min` / `Max` OverDrive sliders (`GFX Clock`, `Memory Clock`, `Power Limit %`, `Fan Speed RPM`, `Fan Acoustic RPM`), control flags, plus separate **Power Modes** (`Limit %` / `Temp °C`) and **Power Mode Acoustic** (`Acoustic RPM` / `Target °C`) cards.
* **RDNA 3**: Configures `Max` OverDrive bounds (`od_gfx_max`, `od_mem_max`, `od_pwr_max`, `od_fan_max`, `od_acoustic_max`).

### 3. Power
* **Core Voltage (mV)**: Edit `GFX` and `Soc` `Vmin` / `Vmax` bounds (`0.25 mV` native step resolution).
* **Power & Current**: Configure `Power Limit (W)` (`PPT0`), `TDC GFX (A)`, and `TDC Soc (A)`.
* **ULV & Memory Voltage**: Tune `ULV Vmin` and `Memory Voltage` (`VDDCI` / `MVDD` across DPM 0..3), plus `Thermal Vmin` on RDNA 2.

### 4. Frequency
* Edit `GFX`, `Soc`, and `FCLK` `Min` / `Max` clocks, display/media clocks (`VCLK`, `DCLK`, `DCEFCLK`, `DISPCLK`, `PIXCLK`, `PHYCLK`, `DTBCLK`), `Memory DPM` (`Freq MHz` & `Div`), and `PCIe` Gen/Lane/LCLK states.

### 5. Curve
* **RDNA 2**: Edit quadratic V/F curve coefficients (`a`, `b`, `c`), static voltage offsets, `Thermal Limits`, and `DcBtc & DcTol (mV)`.
* **RDNA 3**: Edit `Thermal Limits` (`Edge`, `Hotspot`, `Memory`, `VR`, `Liquid`) while AVFS manages per-die V/F curves dynamically.

### 6. Fan
* Configure `PWM Min (%)`, `Acoustic Limit (RPM)`, `Throttling RPM`, `Max RPM`, `Target Temp (°C)`, `Target GFX Clock (MHz)`, and **Zero RPM** (`Enable Zero RPM`, `Stop Temp °C`, `Start Temp °C`) with automatic warning if the OD8 Zero RPM flag is disabled.

### 7. Advance
* Stacks the **OS Status** card on top and the **Export / Deploy** card on the bottom for **macOS**, **Windows**, and **Linux**.

### 8. Safe Tune & More
* **Safe Tune**: Validates edited parameters against calibrated thermal, voltage, and power guardrails.
* **More**: Inline cards for secondary power limits (`PPT1..3`) and `Thermal Throttler` limits.

### 9. Log
* **1. Offset Audit**: Compares live UI values against native RDNA 2 / RDNA 3 struct offsets (`OK` / `DIFF`).
* **2. Capture Log**: Displays `~/.config/navitune/debug.log` in-app.
* **3. Native Offset**: Lists every Rust struct field, byte offset, size, and decoded value.
* **4. Struct Window**: Hex/byte inspector for raw PPTable offsets.
* **5. Dump Audit**: Exports a full diagnostic snapshot.
