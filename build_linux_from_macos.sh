#!/usr/bin/env bash
set -euo pipefail

APP_NAME="NaviTune"
BIN_NAME="navitune"
TARGET="x86_64-unknown-linux-gnu"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-x86_64"

echo "==> Adding Rust target ${TARGET}..."
rustup target add "${TARGET}"

if command -v cargo-zigbuild >/dev/null 2>&1 && command -v zig >/dev/null 2>&1; then
    echo "==> Cross-compiling ${BIN_NAME} for Linux via cargo-zigbuild..."
    cargo zigbuild --release --target "${TARGET}.2.31"
elif command -v cross >/dev/null 2>&1; then
    echo "==> Cross-compiling ${BIN_NAME} for Linux via cross (Docker)..."
    cross build --release --target "${TARGET}"
else
    echo "Error: Install cargo-zigbuild + zig ('brew install zig && cargo install cargo-zigbuild') or 'cargo install cross'."
    exit 1
fi

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin" "${STAGE_DIR}/share/applications"
cp "target/${TARGET}/release/${BIN_NAME}" "${STAGE_DIR}/bin/${BIN_NAME}"
chmod +x "${STAGE_DIR}/bin/${BIN_NAME}"
[ -f "README.md" ] && cp "README.md" "${STAGE_DIR}/"

cat << DESKTOP > "${STAGE_DIR}/share/applications/navitune.desktop"
[Desktop Entry]
Type=Application
Name=NaviTune
GenericName=RDNA 2 PowerPlay Table Editor
Comment=Inspect, edit, validate, and export AMD RDNA 2 Soft PowerPlay Tables
Exec=navitune
Icon=navitune
Terminal=false
Categories=System;Settings;HardwareSettings;Utility;
Keywords=AMD;Radeon;RDNA2;PowerPlay;PPTable;VBIOS;GPU;
StartupWMClass=navitune
DESKTOP

TAR_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-x86_64.tar.gz"
rm -f "${TAR_PATH}"
tar -czf "${TAR_PATH}" -C "${DIST_DIR}" "${APP_NAME}-${VERSION}-Linux-x86_64"

echo "==> Done! Output artifacts:"
ls -lh "${TAR_PATH}"
