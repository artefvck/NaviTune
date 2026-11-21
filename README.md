# NaviTune (`pptool`)

**NaviTune** (`pptool`) is a cross-platform Rust GUI utility (`eframe`/`egui`) for editing Soft PowerPlay Tables on AMD Radeon RDNA 2 and RDNA 3 GPUs across macOS, Windows, and Linux.

## Key Features
* **Packed Struct Mapping & Precision**: Zero-copy mapping and native hardware unit support.
* **Workspace Tabs**: Includes Features, OC Limits, Power, Frequency, Curve & Fan, Safe Tune, and diagnostic logs.
* **Multi-Format Support**: Handles raw binaries, VBIOS ROMs, MorePowerTool `.mpt`, Windows Registry `.reg`, and OpenCore `.plist`.
* **Cross-Platform**: Supports macOS OpenCore/kext deployment, Windows registry editing, and Linux `amdgpu` sysfs.

## Documentation & Build
Consult the [`doc/`](doc/) directory for architecture and guides. Run `cargo test` and `cargo run --release` to build locally.

## License
Licensed under the [MIT License](LICENSE).
