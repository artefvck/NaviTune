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
