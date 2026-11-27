#!/usr/bin/env bash
set -euo pipefail

APP_NAME="NaviTune"
BIN_NAME="navitune"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
ARCH="$(uname -m)"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-${ARCH}"

if command -v apt-get >/dev/null 2>&1; then
    MISSING_PKGS=()
    for pkg in build-essential pkg-config libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev; do
        dpkg -s "${pkg}" >/dev/null 2>&1 || MISSING_PKGS+=("${pkg}")
    done
    if [ ${#MISSING_PKGS[@]} -gt 0 ]; then
        echo "Warning: Missing Ubuntu build dependencies: ${MISSING_PKGS[*]}"
        echo "Install with: sudo apt-get update && sudo apt-get install -y ${MISSING_PKGS[*]}"
    fi
fi

echo "==> Building ${BIN_NAME} for Linux (${ARCH})..."
cargo build --release

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/bin" "${STAGE_DIR}/share/applications"

cp "target/release/${BIN_NAME}" "${STAGE_DIR}/bin/${BIN_NAME}"
chmod +x "${STAGE_DIR}/bin/${BIN_NAME}"
[ -f "README.md" ] && cp "README.md" "${STAGE_DIR}/"

cat << 'INSTALL_SH' > "${STAGE_DIR}/install.sh"
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE}")" && pwd)"
sudo install -Dm755 "${SCRIPT_DIR}/bin/navitune" /usr/local/bin/navitune
sudo install -Dm644 "${SCRIPT_DIR}/share/applications/navitune.desktop" /usr/share/applications/navitune.desktop
echo "Installed navitune to /usr/local/bin/navitune"
INSTALL_SH
chmod +x "${STAGE_DIR}/install.sh"

cat << DESKTOP > "${STAGE_DIR}/share/applications/navitune.desktop"
[Desktop Entry]
Type=Application
Name=NaviTune
GenericName=AMD Radeon PowerPlay Table & VBIOS Editor
Comment=Inspect, edit, validate, and export AMD Polaris, Vega & RDNA 1-3 PowerPlay Tables
Exec=navitune
Icon=navitune
Terminal=false
Categories=System;Settings;HardwareSettings;Utility;
Keywords=AMD;Radeon;Polaris;Vega;RDNA;PowerPlay;PPTable;VBIOS;GPU;
StartupWMClass=navitune
DESKTOP

TAR_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-Linux-${ARCH}.tar.gz"
rm -f "${TAR_PATH}"
tar -czf "${TAR_PATH}" -C "${DIST_DIR}" "${APP_NAME}-${VERSION}-Linux-${ARCH}"

echo "==> Done! Output artifacts:"
ls -lh "${TAR_PATH}"
