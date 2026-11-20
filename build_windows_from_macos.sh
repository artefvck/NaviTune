#!/usr/bin/env bash
set -euo pipefail

APP_NAME="xPPT"
TARGET="x86_64-pc-windows-msvc"
DIST_DIR="dist/${APP_NAME}-windows-x86_64"

echo "=== Cross-Compiling ${APP_NAME} for Windows (${TARGET}) on macOS ==="
if ! command -v cargo-xwin >/dev/null 2>&1; then
    echo "[ERROR] Install cargo-xwin first: cargo install cargo-xwin && rustup target add ${TARGET}"
    exit 1
fi

rustup target add "${TARGET}" >/dev/null 2>&1 || true
cargo xwin build --release --target "${TARGET}"

mkdir -p "${DIST_DIR}"
cp "target/${TARGET}/release/${APP_NAME}.exe" "${DIST_DIR}/${APP_NAME}.exe"
rm -f "dist/${APP_NAME}-windows-x86_64.zip"
(cd dist && zip -rq "${APP_NAME}-windows-x86_64.zip" "${APP_NAME}-windows-x86_64")

echo "[DONE] Windows build ready: dist/${APP_NAME}-windows-x86_64.zip"
