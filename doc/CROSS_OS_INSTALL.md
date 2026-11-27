# Cross-OS Installation & Deployment Guide

Complete step-by-step instructions for installing, building, and applying soft PowerPlayTables (`PP_PhmSoftPowerPlayTable`) with **NaviTune** across **Linux (Ubuntu/Debian)**, **Windows 10/11**, and **macOS (Hackintosh / eGPU)**.

---

## 1. Linux (Ubuntu / Debian)

### A. Install via `.deb` Package (Recommended)
```bash
sudo apt update
sudo apt install -y ./dist/navitune_1.0.0_amd64.deb
navitune
```
*(Or run/install the portable `.tar.gz` binary directly)*:
```bash
tar -xzf dist/NaviTune-1.0.0-Linux-x86_64.tar.gz
sudo install -m 0755 navitune /usr/local/bin/navitune
navitune
```

### B. Build from Source on Ubuntu
```bash
sudo apt-get update && sudo apt-get install -y \
  build-essential pkg-config libgtk-3-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev zenity policykit-1

chmod +x build_linux.sh
./build_linux.sh
```

### C. Unlock `amdgpu` Soft `pp_table` Writes & Boot Persistence
1. Enable OverDrive bit `0x4000` in `/etc/default/grub`:
   ```bash
   GRUB_CMDLINE_LINUX_DEFAULT="quiet splash amdgpu.ppfeaturemask=0xffffffff"
   ```
   Then run `sudo update-grub && sudo reboot`.
2. **Temporary RAM Test (Reverts on Reboot)**:
   - Open **More -> 3. Advance -> Linux**, click **Copy sysfs apply command**, and run:
     ```bash
     sudo dd if=pp_table.bin of=/sys/class/drm/card1/device/pp_table bs=65536
     ```
3. **Permanent Boot Persistence (`systemd`)**:
   - In **More -> 3. Advance -> Linux**, click **Install** (authenticates via `pkexec`), which installs `/etc/xppt/pp_table.bin` and enables `xppt-pptable.service`.
   - Click **Uninstall** at any time to remove the service and revert to stock VBIOS on next reboot.

---

## 2. Windows 10 / 11

### A. Run Prebuilt Release Binary
1. Extract `NaviTune-1.0.0-Windows-x86_64.zip` and run `navitune.exe`.
2. **Administrator Privileges**: `navitune.exe` embeds a native Windows UAC `requireAdministrator` manifest so it can directly write to `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000X` without manual elevation prompts.

### B. Build from Source on Windows (PowerShell)
```powershell
# Requires Rust MSVC toolchain (x86_64-pc-windows-msvc)
.\build_windows.ps1
```
*(Or cross-compile from macOS via `./build_window_from_macos.sh` using `cargo-xwin`).*

### C. Apply Soft PowerPlayTable on Windows
1. Load a `.rom`, `.bin`, `.mpt`, or `.reg` file (or read live from the selected AMD GPU).
2. Tune clocks, voltages, fan curve, or power limits and verify in **Safe Tune**.
3. Open **More -> 3. Advance -> Windows**:
   - Click **Write to Registry** to inject `PP_PhmSoftPowerPlayTable` into the selected GPU adapter key (`0000`, `0001`, etc.).
   - Click **Restart GPU Driver** (or reboot Windows) to activate the new table.
   - Click **Delete from Registry** and restart the GPU driver/PC to restore factory VBIOS defaults.
4. **MorePowerTool Interop**: Export `.mpt` (includes the 256-byte `55 AA` header + PCI Device ID) or `.reg` files directly via **Save** or **Diff & Save**.

---

## 3. macOS (Hackintosh / Intel & Apple Silicon eGPU)

### A. Install via `.app` Bundle / `.dmg`
1. Open `dist/NaviTune-1.0.0-macOS-universal.dmg` (or `arm64`/`x86_64`) and drag **NaviTune.app** into `/Applications`.
2. If macOS Gatekeeper warns on an unsigned local build, clear the quarantine attribute:
   ```bash
   xattr -cr /Applications/NaviTune.app
   ```

### B. Build from Source on macOS
```bash
chmod +x build_macos.sh
./build_macos.sh
```

### C. Inject Soft PowerPlayTable into OpenCore (`PP,PhmSoftPowerPlayTable`)
1. Select your live AMD GPU from the top dropdown (`IORegistry` auto-discovery) or load your GPU's `.rom` / `.mpt`.
2. *(Optional for RX 6600 / Navi 23)*: In **More -> 3. Advance -> macOS**, click **Unlock RX 6600 500 MHz** to clear `VCN_PG` (Bit 22) and configure the `AMDRadeonX6000`-compatible DPM envelope.
3. Choose one of three injection methods via **Quick Export ▾** or **More -> 3. Advance -> macOS**:
   - **OpenCore `.plist` (`DeviceProperties`)**: Merge the generated `PP,PhmSoftPowerPlayTable` `<data>` entry under your GPU's `PciRoot(...)` path in `EFI/OC/config.plist`.
   - **Codeless Injector `.kext`**: Export `NaviTuneInjector.kext`, copy it to `EFI/OC/Kexts/`, and register it in `config.plist -> Kernel -> Add`.
   - **ACPI `SSDT-sPPT.aml`**: Export the compiled AML table to `EFI/OC/ACPI/` and enable it in `config.plist -> ACPI -> Add`.
4. Reboot and reset NVRAM if needed; verify active injection in **More -> 3. Advance -> macOS** (`IORegistry` status).

---

## 4. Headless CLI Quick Reference (All Operating Systems)

```bash
# Inspect VBIOS headers, \$PSP directory, checksum & limits
navitune --inspect target/Navi23.rom

# Audit UI-to-binary struct parity (0 mismatches expected)
navitune --audit target/stable.mpt

# Byte-for-byte comparison with struct field resolution
navitune --diff target/stable.mpt target/pp_table_patched.mpt

# Apply preset and export cross-platform formats
navitune --preset balanced-oc target/Navi23.rom \
  --export-mpt tuned.mpt \
  --export-reg tuned.reg \
  --export-oc tuned.plist
```
