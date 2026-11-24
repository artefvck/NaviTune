#!/usr/bin/env bash
set -euo pipefail

APP_NAME="NaviTune"
BIN_NAME="navitune"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
ARCH="$(uname -m)"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-${ARCH}"

echo "==> Building ${BIN_NAME} for Linux (${ARCH})..."
cargo build --release

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin" "${STAGE_DIR}/share/applications"

cp "target/release/${BIN_NAME}" "${STAGE_DIR}/bin/${BIN_NAME}"
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

TAR_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-${ARCH}.tar.gz"
rm -f "${TAR_PATH}"
tar -czf "${TAR_PATH}" -C "${DIST_DIR}" "${APP_NAME}-${VERSION}-Linux-${ARCH}"

echo "==> Done! Output artifacts:"
ls -lh "${TAR_PATH}"
