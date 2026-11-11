#!/usr/bin/env bash
# Builds dist/linstress-<version>-<arch>.AppImage
#
# Usage:  ./scripts/build-appimage.sh
# Options (environment variables):
#   PROFILE=dist    use the slower, smaller "dist" build profile (default: release)
#
# The AppImage does not bundle system libraries (Vulkan, Wayland, xkbcommon),
# so build it on an older distribution (for example Ubuntu 22.04) to make it
# run on as many systems as possible.
set -euo pipefail

cd "$(dirname "$0")/.."

NAME=linstress
PROFILE="${PROFILE:-release}"
ARCH="$(uname -m)"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
APPDIR="build/AppDir"
TOOL="build/tools/appimagetool-${ARCH}.AppImage"
OUTPUT="dist/${NAME}-${VERSION}-${ARCH}.AppImage"

echo "==> Building the program ($PROFILE profile)"
if [ "$PROFILE" = "release" ]; then
    cargo build --release
else
    cargo build --profile "$PROFILE"
fi

echo "==> Preparing the AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" \
         "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/metainfo" \
         "$APPDIR/usr/share/icons"

install -m 755 "target/${PROFILE}/${NAME}" "$APPDIR/usr/bin/${NAME}"
install -m 644 "packaging/${NAME}.desktop" "$APPDIR/usr/share/applications/"
install -m 644 "packaging/${NAME}.metainfo.xml" "$APPDIR/usr/share/metainfo/"
cp -r assets/icons/hicolor "$APPDIR/usr/share/icons/"

# AppImage expects the desktop file and icon at the top level too.
cp "packaging/${NAME}.desktop" "$APPDIR/${NAME}.desktop"
cp "assets/icons/hicolor/256x256/apps/${NAME}.png" "$APPDIR/${NAME}.png"
cp "assets/icons/hicolor/256x256/apps/${NAME}.png" "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/linstress" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

echo "==> Getting appimagetool"
if [ ! -x "$TOOL" ]; then
    mkdir -p "$(dirname "$TOOL")"
    curl -fL -o "$TOOL" \
        "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    chmod +x "$TOOL"
fi

echo "==> Creating the AppImage"
mkdir -p dist
# --appimage-extract-and-run works on systems without FUSE (for example in CI).
ARCH="$ARCH" "$TOOL" --appimage-extract-and-run --no-appstream "$APPDIR" "$OUTPUT"

echo "Created $OUTPUT"
