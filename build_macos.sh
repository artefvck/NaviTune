#!/usr/bin/env bash
set -euo pipefail

APP_NAME="NaviTune"
BIN_NAME="navitune"
BUNDLE_ID="com.navitune.navitune"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
DIST_DIR="dist"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"

echo "==> Installing Rust macOS targets (aarch64 & x86_64)..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin

echo "==> Building ${BIN_NAME} for Apple Silicon (arm64)..."
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target aarch64-apple-darwin

echo "==> Building ${BIN_NAME} for Intel (x86_64)..."
MACOSX_DEPLOYMENT_TARGET=10.15 cargo build --release --target x86_64-apple-darwin

echo "==> Creating Universal Binary with lipo..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS" "${APP_DIR}/Contents/Resources"

lipo -create \
    "target/aarch64-apple-darwin/release/${BIN_NAME}" \
    "target/x86_64-apple-darwin/release/${BIN_NAME}" \
    -output "${APP_DIR}/Contents/MacOS/${BIN_NAME}"

chmod +x "${APP_DIR}/Contents/MacOS/${BIN_NAME}"

if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "${APP_DIR}/Contents/Resources/icon.icns"
fi

echo "==> Writing Info.plist..."
cat << PLIST > "${APP_DIR}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://apple.com"\>
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${BIN_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST

echo "==> Ad-hoc signing ${APP_DIR}..."
codesign --force --deep --sign - "${APP_DIR}"

echo "==> Verifying Universal architectures..."
lipo -info "${APP_DIR}/Contents/MacOS/${BIN_NAME}"

echo "==> Packaging compressed DMG..."
DMG_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-macOS-Universal.dmg"
rm -f "${DMG_PATH}"
hdiutil create -volname "${APP_NAME}" -srcfolder "${APP_DIR}" -ov -format UDZO "${DMG_PATH}"

echo "==> Done! Output artifacts:"
ls -lh "${APP_DIR}/Contents/MacOS/${BIN_NAME}" "${DMG_PATH}"
