#!/usr/bin/env bash
# package_macos.sh
# Build, assemble, and package TZWorkQuint suite into macOS DMG and tar.gz archives.
# Targets: aarch64-apple-darwin (Apple Silicon) and x86_64-apple-darwin (Intel).
set -euo pipefail

VERSION="${1:-3.0.0}"
ARCH="${2:-aarch64}"
TARGET="${ARCH}-apple-darwin"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
STAGE_APP="${DIST_DIR}/TZWorkQuint.app"
OUTPUT_DMG="${DIST_DIR}/TZWorkQuint-${VERSION}-macos-${ARCH}.dmg"
OUTPUT_TARGZ="${DIST_DIR}/TZWorkQuint-${VERSION}-macos-${ARCH}.tar.gz"

echo "=== Packaging TZWorkQuint ${VERSION} for macOS (${ARCH}) ==="
mkdir -p "${DIST_DIR}"
rm -rf "${STAGE_APP}"

# 1. Create .app directory skeleton
mkdir -p "${STAGE_APP}/Contents/MacOS"
mkdir -p "${STAGE_APP}/Contents/Resources"
mkdir -p "${STAGE_APP}/Contents/SharedSupport/bin"
mkdir -p "${STAGE_APP}/Contents/SharedSupport/assets"
mkdir -p "${STAGE_APP}/Contents/SharedSupport/docs"

# 2. Populate Info.plist
sed -e "s/@SHORT_VERSION@/${VERSION}/g" \
    -e "s/@VERSION@/${VERSION}/g" \
    "${ROOT_DIR}/packaging/macos/Info.plist.in" > "${STAGE_APP}/Contents/Info.plist"

# 3. Copy binaries (built via cargo build --release --target ${TARGET})
APPS=("tz-workquint" "tz-cellforge" "tz-wordloom" "tz-slidespark" "tz-pdflex" "tz-draftline")
for app in "${APPS[@]}"; do
    BIN_SRC="${ROOT_DIR}/target/${TARGET}/release/${app}"
    if [[ -f "${BIN_SRC}" ]]; then
        cp "${BIN_SRC}" "${STAGE_APP}/Contents/SharedSupport/bin/${app}"
        chmod +x "${STAGE_APP}/Contents/SharedSupport/bin/${app}"
        echo "  Bundled binary: ${app}"
    else
        echo "Error: Required binary ${app} not found at ${BIN_SRC}" >&2
        exit 1
    fi
done

# 4. Main executable entry point (regular file required for codesign and launch services)
cp "${STAGE_APP}/Contents/SharedSupport/bin/tz-workquint" "${STAGE_APP}/Contents/MacOS/tz-workquint"
chmod +x "${STAGE_APP}/Contents/MacOS/tz-workquint"

# 5. Copy shared assets and docs
if [[ -d "${ROOT_DIR}/assets" ]]; then
    cp -R "${ROOT_DIR}/assets" "${STAGE_APP}/Contents/SharedSupport/"
fi
if [[ -d "${ROOT_DIR}/docs" ]]; then
    cp -R "${ROOT_DIR}/docs" "${STAGE_APP}/Contents/SharedSupport/"
fi
cp "${ROOT_DIR}/LICENSE.txt" "${STAGE_APP}/Contents/Resources/"
cp "${ROOT_DIR}/THIRD_PARTY_NOTICES.txt" "${STAGE_APP}/Contents/Resources/"

# 5. Codesign if identity provided
if [[ -n "${MACOS_SIGN_IDENTITY:-}" ]]; then
    echo "Signing application bundle with identity: ${MACOS_SIGN_IDENTITY}..."
    codesign --deep --force --options runtime \
             --entitlements "${ROOT_DIR}/packaging/macos/entitlements.plist" \
             --sign "${MACOS_SIGN_IDENTITY}" "${STAGE_APP}"
else
    echo "No MACOS_SIGN_IDENTITY supplied; applying ad-hoc signature (-)..."
    codesign --deep --force --sign - "${STAGE_APP}"
fi

# 6. Generate DMG
python3 "${ROOT_DIR}/packaging/macos/generate_dmg.py" "${STAGE_APP}" "${OUTPUT_DMG}" --volume-name "TZWorkQuint"

# 7. Generate portable tar.gz
echo "Creating portable archive: ${OUTPUT_TARGZ}..."
tar -czf "${OUTPUT_TARGZ}" -C "${DIST_DIR}" "TZWorkQuint.app"
echo "macOS packaging complete for ${ARCH}: created ${OUTPUT_DMG} and ${OUTPUT_TARGZ}."
