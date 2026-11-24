# NaviTune GUI Quick Reference (When to Click What & Where)

## 1. Screen Layout Overview

- **Top Card**: macOS `IORegistry` GPU selector dropdown, **`Theme`** customizer, and **`Live Telemetry`** sensor bar.
- **Tab Strip**: `Features` · `OC Limits` · `Power` · `Frequency` · `Curve` · `Fan` · `Advance` · `Safe Tune` · `More` · `Log`.
- **Two-Column Cards**:
  - **Gray border**: Unmodified factory baseline.
  - **Crimson border / Red `●`**: Modified value or checkbox.
  - **Hover Tooltip**: Displays AMD struct field name, `+0xXXXX` binary offset, factory default, and live `+Δ` / `-Δ` delta.
- **Bottom Action Bar**: `Load` · `Save` · `Diff & Save (N B)` · `Quick Export ▾` · `Reset All` · `Exit`.
- **Status Footer**: Detected architecture & table size (left), hovered field & offset (center), and live `✔ Verified` / `⚠` validation badge (right).

## 2. Task-by-Task Guide (When to Go Where)

| Goal | Where to Click in the GUI | What It Does |
| :--- | :--- | :--- |
| **Open a `.rom`, `.mpt`, `.reg`, `.plist`, or `.bin`** | Bottom Bar → **`Load`** | Loads the `PPTable` (and keeps full VBIOS ROM bytes intact if a `.rom` is opened). |
| **Load active macOS GPU table directly** | Top Bar → **`Select GPU Path...`** | Reads live `PP_PhmSoftPowerPlayTable` from macOS `IORegistry`. |
| **Monitor live GPU clocks, temp, power & fan** | Top Bar → **`Live Telemetry`** | Streams real-time sensor telemetry every second. |
| **Fix macOS RX 6600 (`Navi 23`) stuck at `500 MHz`** | **`Advance`** → **`macOS`** → **`Apply RX 6600 (Navi 23) 500 MHz Fix`** | Disables `VCN_PG` (Bit `22`) and sets `300..2350 MHz` GFX, `1050 mV`, `100 W` PPT, `100 A` TDC, and safe fan hysteresis. |
| **Raise OverDrive slider ceilings** | **`OC Limits`** | Edits `od_gfx_max`, `od_mem_max`, `od_pwr_max` (+%), and `od_fan_max`. |
| **Undervolt or change Power / TDC limits** | **`Power`** | Adjusts `SocketPowerLimitAc` (W), `TdcLimitGfx/Soc` (A), ULV floors, and `VDDCI`/`MVDD` voltages. |
| **Adjust GFX, Memory (UCLK), or FCLK ranges** | **`Frequency`** | Sets min/max DPM frequencies and PCIe Gen/Lane limits. |
| **Edit V/F Curve or Thermal Throttle Limits** | **`Curve`** | Drag points on the 2D V/F plot or edit quadratic curve coefficients and thermal limits (`°C`). |
| **Configure Zero RPM or 5-point Fan Curve** | **`Fan`** | Sets `Stop Temp < Start Temp`, min PWM %, max RPM, and drags points on the 5-point fan plot. |
| **Apply one-click safe presets / fix warnings** | **`Safe Tune`** | Applies **Quiet Undervolt**, **Balanced OC**, or **Auto-Fix Risks** for your detected silicon. |
| **Inspect exact hex offsets & struct layout** | **`More`** → **`3. Native Offset`** / **`4. Struct Window`** | Filters and inspects every struct field and `0x` byte offset across all 6 architectures. |
| **Export OpenCore `.plist`, `.kext`, or Base64/Hex** | Bottom Bar → **`Quick Export ▾`** | Instant export from any tab without switching views. |
| **Audit every modified byte before saving** | Bottom Bar → **`Diff & Save (N B)`** | Shows old vs. new bytes with struct field names and updates the AtomBIOS `0x21` checksum. |
| **Clear all changes** | Bottom Bar → **`Reset All`** | Resets all UI fields back to initial defaults. |
