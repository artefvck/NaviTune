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

DEB_PATH=""
if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
    DEB_STAGE="$(mktemp -d)"
    mkdir -p "${DEB_STAGE}/DEBIAN" "${DEB_STAGE}/usr/bin" "${DEB_STAGE}/usr/share/applications"
    chmod -R g-s "${DEB_STAGE}"
    chmod 00755 "${DEB_STAGE}" "${DEB_STAGE}/DEBIAN" "${DEB_STAGE}/usr" "${DEB_STAGE}/usr/bin" "${DEB_STAGE}/usr/share" "${DEB_STAGE}/usr/share/applications"
    cp "target/release/${BIN_NAME}" "${DEB_STAGE}/usr/bin/${BIN_NAME}"
    chmod 0755 "${DEB_STAGE}/usr/bin/${BIN_NAME}"
    cp "${STAGE_DIR}/share/applications/navitune.desktop" "${DEB_STAGE}/usr/share/applications/navitune.desktop"
    cat << CONTROL > "${DEB_STAGE}/DEBIAN/control"
Package: ${BIN_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Depends: libc6, libgcc-s1, libgtk-3-0, libxcb1, libxkbcommon0, policykit-1 | pkexec
Maintainer: NaviTune Contributors
Description: AMD Radeon PowerPlay Table & VBIOS Editor
 Inspect, edit, validate, and export AMD Polaris, Vega, and RDNA 1-3 PowerPlay Tables.
CONTROL
    DEB_PATH="${DIST_DIR}/${BIN_NAME}_${VERSION}_${DEB_ARCH}.deb"
    rm -f "${DEB_PATH}"
    chmod 0644 "${DEB_STAGE}/DEBIAN/control"
    dpkg-deb --build --root-owner-group "${DEB_STAGE}" "${DEB_PATH}"
    rm -rf "${DEB_STAGE}"
fi

echo "==> Done! Output artifacts:"
ls -lh "${TAR_PATH}" ${DEB_PATH:+"${DEB_PATH}"}
