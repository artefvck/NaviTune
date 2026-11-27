# Universal SMU Calculator (`Calc`) & Classic Adrenalin WattMan (`Classic`) Guide

NaviTune includes two specialized tuning workspaces that expose the underlying AMD System Management Unit (SMU) mathematical models across **Polaris**, **Vega 10/20**, **RDNA 1**, **RDNA 2**, and **RDNA 3**:

1. **`Calc` (`src/tabs/calc.rs`)**: Universal *what-if* SMU envelope simulator for AVFS quadratic voltage curves, `DCBTC` droop / `500 MHz` failsafe lock prediction, Infinity Fabric coupling (`FCLK >= 2 × UCLK`), and dynamic board power (`PPT0` / `TDC`) headroom.
2. **`Classic` (`src/tabs/classic.rs`)**: Classic AMD Adrenalin WattMan (2019/2020) **3-Point V/F Curve (`Min` / `Mid` / `Max`)** editor with an exact 3×3 quadratic solver (`a, b, c`) and coupled **5-Point Fan Curve (`P1..P5`)**.

---

## 1. Universal SMU Envelope Calculator (`Calc` Tab)

### A. Automatic Factory Baseline Calibration
When you load a VBIOS `.rom`, `.mpt`, `.bin`, `.reg`, `.plist`, or live GPU `pp_table`, `Calc` automatically reads both the **factory baseline** (`baseline_buffer`) and your **current edited values** so every power, voltage, and frequency delta is calibrated specifically to your silicon rather than hardcoded defaults.

### B. Mathematical Models Computed in Real Time
- **AVFS Quadratic Curve (`0x74E` `static_volt_gfx`)**:
  Evaluates V(f) = (a · f² + b · f + c) × 1000 mV (with f in GHz) at your target `Fmax`, plus any static curve offset (`0x7AA` `static_volt_offset_gfx`).
- **`DCBTC` Droop & `500 MHz` Failsafe Predictor (`0x7C2..0x7D6`)**:
  On RDNA 2 (`Navi 21–24`), high `GFX Vmax` combined with peak `UCLK` (`DPM 3`) inside a tight `PPT0` / `TDC` envelope can cause transient rail droop to exceed `dcbtc_max` (`56.25 mV`) minus `dcbtc_gb` (`6.25 mV`), triggering SMU discrete `500 MHz` failsafe clamping. `Calc` highlights **`BOOST SAFE`**, **`MARGINAL`**, or **`DPM LOCK RISK`** before you apply changes to hardware.
- **Infinity Fabric Coupling (`FCLK >= 2 × UCLK`)**:
  Verifies that Infinity Fabric clock (`FCLK`) remains at least `2 × UCLK` so memory bandwidth is not bottlenecked by fabric transfers.
- **Dynamic Power Scaling (`P ∝ f · V²`) & `PPT0` / `TDC` Headroom**:
  Computes percentage GFX power reduction from undervolting relative to your factory `Vmax`, estimates memory subsystem draw vs. core draw, and checks remaining `PPT0` (W) and `TDC GFX` (A) headroom.

---

## 2. Classic Adrenalin WattMan (`Classic` Tab)

Modern AMD Radeon Adrenalin drivers removed the classic 3-point (`Min` / `Mid` / `Max`) V/F curve editor. NaviTune restores this workflow and compiles your 3 points directly into native SMU quadratic polynomial coefficients (`a, b, c`).

### A. How the 3-Point Quadratic Solver Works
Given three frequency-voltage pairs `(f_min, V_min)`, `(f_mid, V_mid)`, and `(f_max, V_max)` (converted to GHz and Volts), NaviTune solves the unique Lagrange / Vandermonde quadratic polynomial:

```text
V(f) = a · f² + b · f + c
```

1. **Point 1 (`Min / Idle`)**: Sets the idle/low-load floor (`freq_gfx_min` & `min_voltage_gfx`).
2. **Point 2 (`Mid / Game`)**: Shapes the mid-range efficiency knee where light and medium workloads spend most of their time. Lowering `V_mid` dramatically reduces average gaming power and heat without sacrificing peak boost.
3. **Point 3 (`Max / Boost`)**: Sets peak boost clock (`freq_gfx_max`) and voltage ceiling (`max_voltage_gfx`).

### B. Target Table Modes (RDNA 2 `V4_9`)
- **`Write to Voltage Offset (0x7AA — Safe AVFS)` (Recommended)**:
  Fits `(a, b, c)` to the **delta** `ΔV(f) = V_target(f) - V_factory(f)` across your three points. This preserves silicon-fused AVFS calibration while shifting the entire curve smoothly to match your `Min / Mid / Max` targets.
- **`Write to Base VID Curve (0x74E)`**:
  Writes `(a, b, c)` directly into `static_volt_gfx` for advanced manual curve modeling.

### C. Coupled 5-Point Fan Curve (`P1..P5`)
- Links `Zero RPM`, `fan_target_gfx_clock`, and the 5-point fan table (`P1..P5 Temp °C` and `PWM %`) to your V/F curve.
- Click **`Auto-Scale Fan to V/F Load`** to automatically proportion PWM duty cycle across `P1..P5` based on your chosen `V_max` voltage envelope, then click **`Apply Classic Curve (a, b, c + Fan)`** to sync all values into the active `PPTable`.
