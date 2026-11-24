#!/usr/bin/env bash
set -euo pipefail

APP_NAME="NaviTune"
BIN_NAME="navitune.exe"
TARGET="x86_64-pc-windows-gnu"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${APP_NAME}-${VERSION}-Windows-x86_64"

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "Error: mingw-w64 not found. Install via: brew install mingw-w64"
    exit 1
fi

echo "==> Adding Rust target ${TARGET}..."
rustup target add "${TARGET}"

echo "==> Cross-compiling ${BIN_NAME} for Windows from macOS..."
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
    cargo build --release --target "${TARGET}"

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"
cp "target/${TARGET}/release/${BIN_NAME}" "${STAGE_DIR}/${BIN_NAME}"
[ -f "README.md" ] && cp "README.md" "${STAGE_DIR}/"

ZIP_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-Windows-x86_64.zip"
rm -f "${ZIP_PATH}"
(cd "${DIST_DIR}" && zip -rq "${APP_NAME}-${VERSION}-Windows-x86_64.zip" "${APP_NAME}-${VERSION}-Windows-x86_64")

echo "==> Done! Output artifacts:"
ls -lh "${ZIP_PATH}"
