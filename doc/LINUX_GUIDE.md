# Linux & Ubuntu Integration Guide

NaviTune provides native `amdgpu` sysfs inspection, live hardware telemetry, persistent `systemd` PowerPlayTable overrides, and Debian/Ubuntu `.deb` packaging.

## 1. Prerequisites & Kernel Unlock (`amdgpu.ppfeaturemask`)

By default, the Linux `amdgpu` driver restricts OverDrive and soft `pp_table` writes unless bit `0x4000` (`PP_OVERDRIVE_MASK`) is enabled in `ppfeaturemask`.

1. Edit `/etc/default/grub` and append `amdgpu.ppfeaturemask=0xffffffff` to `GRUB_CMDLINE_LINUX_DEFAULT`:
   ```bash
   GRUB_CMDLINE_LINUX_DEFAULT="quiet splash amdgpu.ppfeaturemask=0xffffffff"
   ```
2. Regenerate GRUB configuration and reboot:
   ```bash
   sudo update-grub
   sudo reboot
   ```
3. Verify in **More -> 3. Advance -> Linux**:
   - **`ppfeaturemask`**: Shows `UNLOCKED` (`0xFFFFFFFF (OverDrive bit 0x4000 ON)`).
   - **`sysfs Target`**: Shows `READY` for `/sys/class/drm/cardX/device/pp_table`.

## 2. Applying & Persisting Soft PowerPlay Tables (`systemd`)

In **More -> 3. Advance -> Linux**:
- **Install**: Writes the active PowerPlayTable buffer to `/etc/xppt/pp_table.bin`, applies it directly to `/sys/class/drm/cardX/device/pp_table` (verified via `cmp -s`), and enables `/etc/systemd/system/xppt-pptable.service` so the override reapplies automatically on every boot.
- **Uninstall**: Disables `xppt-pptable.service`, removes `/etc/xppt/pp_table.bin`, and resets `power_dpm_force_performance_level` to `auto` (reboot to reload stock VBIOS tables).
- **Config Sync**: Verifies byte-for-byte parity between the editor buffer and the live `sysfs` `pp_table`.


> **Note on `sysfs` `pp_table` writes:** Because `amdgpu`'s `pp_table` node requires a single contiguous `write()` syscall of the exact table size (e.g., `2470 B`) and does not support truncation, use `sudo dd if=pp_table.bin of=/sys/class/drm/cardX/device/pp_table bs=65536` rather than `cp` or `>`. Changes written via `sysfs` live purely in kernel RAM and automatically revert to stock VBIOS on reboot unless `xppt-pptable.service` is installed.

## 3. Live Hardware Telemetry (`sysfs` & `hwmon`)

NaviTune polls AMD GPU sensors directly from `/sys/class/drm/cardX/device/`:
- **Clocks & Load**: `pp_dpm_sclk` / `freq1_input` (GFX MHz), `pp_dpm_mclk` / `freq2_input` (MEM MHz), and `gpu_busy_percent`.
- **Thermals & Power**: Filters `hwmon` nodes specifically where `name == "amdgpu"` to read `temp1_input` (Edge), `temp2_input` (Hotspot/Junction), and `power1_average` / `power1_input` (Total Socket Power in W).
- **Fan & Zero RPM**: Reads `fan1_input` (RPM) and `pwm1` (`0..255` -> `0..100%`), accurately reporting `0 RPM (0%)` when Zero RPM mode is active at idle.
- **VRAM**: Reads `mem_info_vram_used` and `mem_info_vram_total`.

## 4. Wayland / X11 File Dialogs & Cross-Platform `.mpt` Parity

- **Portal-Safe File Picker**: On Linux desktops (`xdg-desktop-portal-gnome` / `xdg-desktop-portal-gtk` under Wayland or X11), NaviTune invokes `zenity` or `kdialog` out-of-process before falling back to `rfd`, preventing synchronous GTK3 portal event-loop aborts when clicking **Load** or **Save**.
- **MorePowerTool `.mpt` Interop**: Supports loading and exporting Igor's Lab binary `.mpt` (256-byte `55 AA` + PCI Device ID header), raw PowerPlayTable `.mpt`, and XML `<PowerPlayTable>`/`.mpt` files with 0-byte delta round-trip parity (`navitune --diff`).

## 5. Building & Installing `.deb` / `.tar.gz` on Ubuntu

### Build Dependencies
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential pkg-config libgtk-3-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev zenity policykit-1
```

### Build Release Artifacts
```bash
chmod +x build_linux.sh
./build_linux.sh
```
This produces:
- `dist/NaviTune-<version>-Linux-<arch>.tar.gz` (portable archive with `install.sh` and `.desktop` entry)
- `dist/navitune_<version>_<arch>.deb` (native Debian/Ubuntu package using `mktemp -d` with setgid bits stripped for `dpkg-deb` compliance)

### Install `.deb` Package
```bash
sudo apt install ./dist/navitune_1.0.0_amd64.deb
navitune
```

## 6. Human-Readable `.cfg` In-Place Patching, Direct RAM Write & Power Profiles

In **More -> 3. Advance -> Linux**:
- **Install (`.cfg` In-Place Patch + `systemd` Service)**: Instead of overwriting `/sys/class/drm/cardX/device/pp_table` with a static binary blob, NaviTune exports all 1:1 UI tuning variables into `/etc/navitune/ppt_override.cfg`. On boot, `xppt-pptable.service` runs `/usr/local/sbin/navitune-apply-ppt.sh`, which:
  1. Checks `/proc/cmdline` for emergency bypass flags (`noppt`, `single`, `recovery`, `nomodeset`).
  2. Waits via `loginctl` until a real desktop user session (`UID >= 1000`, skipping the `gdm` / `sddm` / `lightdm` login greeter) is active so the login screen never freezes during display modesetting.
  3. Runs `navitune --apply-cfg /etc/navitune/ppt_override.cfg --target /sys/class/drm/cardX/device/pp_table` to patch the live kernel `pp_table` in-place while preserving native VBIOS headers and cold-boot GDDR6 training parameters, then automatically synchronizes `pp_od_clk_voltage` (`OD_SCLK` / `OD_MCLK`).
- **Write to GPU (sysfs)**: Validates buffer size parity and writes the active `PPTable` directly into kernel RAM via an elevated `setsid` terminal session with automatic `OD_SCLK` synchronization.
- **Power Profile (`pp_power_profile_mode`)**: Reads the kernel `amdgpu` DPM heuristic profiles (`BOOTUP_DEFAULT`, `3D_FULL_SCREEN`, `POWER_SAVING`, `VIDEO`, `VR`, `COMPUTE`, `CUSTOM`) directly from `sysfs` and allows switching profiles live.

### RDNA 2 (`Navi 23`) Undervolt, GDDR6 Training & 500 MHz Clock-Lock Findings
- **Never alter `mem_dpm_freq` (`UCLK DPM 0..3`) during runtime `sysfs` patching**: On RDNA 2 (`Navi 23`), GDDR6 PHY link training happens once during cold VBIOS initialization (`875 MHz` on RX 6600). Changing `mem_dpm_freq` at runtime causes SMU to lose GDDR6 PHY sync (resulting in a system freeze or a `500 MHz` SCLK failsafe lock). `navitune --apply-cfg` automatically preserves the live VBIOS `mem_dpm_freq` during runtime injection.
- **AVFS / DCBTC Droop Window (`GFX Vmax` vs `MCLK 875 MHz`)**: When `UCLK` runs at peak `DPM 3` (`875 MHz`) within a `100 W` (`PPT0`) / `100 A` (`TDC GFX`) envelope, keeping `GFX Vmax` at stock `1150 mV` alongside a runtime-patched table can exceed the `DCBTC` droop guardband and trigger SMU's `500 MHz` discrete fallback. Lowering `GFX Vmax` to `1050–1100 mV` keeps the quadratic AVFS curve (`qStaticVoltageOffset`) inside the valid `DCBTC` window, unlocking full boost (`2400+ MHz`) at low power and temperatures.
