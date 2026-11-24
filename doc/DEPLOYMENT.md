# Cross-Platform OS Deployment

> **Safety Notice**: Always keep a backup of your stock VBIOS `.rom`, Windows Registry key, or OpenCore `config.plist` before applying custom PowerPlay tables.

## macOS (OpenCore / Hackintosh)

1. Open the **Advance -> macOS** tab.
2. Verify the detected `IOACPIPlane` GPU device path in the **macOS Status** card on top.
3. Choose one of three injection methods in the **OpenCore Export** card:
   * **DeviceProperties (`config.plist`)**: Copy Base64 or Hex, or toggle `Key: PP,PhmSoftPowerPlayTable` / `PP_PhmSoftPowerPlayTable` and export a ready-to-merge `.plist` snippet.
   * **ACPI SSDT (`SSDT-sPPT.aml`)**: Automatically generates and compiles an ACPI table (requires `iasl` from ACPICA).
   * **Codeless Injector (`RadeonPowerPlayInjector.kext`)**: Generates a standalone kext bundle for `EFI/OC/Kexts`.
4. Reboot to apply the injected `PP_PhmSoftPowerPlayTable`.

## Windows (Registry & MorePowerTool)

1. Open the **Advance -> Windows** tab.
2. Check the target display adapter class path (`HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000`) in **Windows Status**.
3. Export a `.reg` or `.mpt` profile, or write `PP_PhmSoftPowerPlayTable` (`REG_BINARY`) directly (requires Administrator privileges).
4. Restart the GPU driver (`Win + Ctrl + Shift + B`) or reboot for changes to take effect.

## Linux (`amdgpu` sysfs & systemd)

1. Open the **Advance -> Linux** tab.
2. Check active DRM card detection (`/sys/class/drm/cardX/device/pp_table`) in **Linux Status**.
3. Apply `pp_table` live to `sysfs` for immediate testing, or install `/etc/xppt/pp_table.bin` with the `xppt-pptable.service` `systemd` unit so custom limits persist across reboots.
