#!/usr/bin/env bash
# Re-creates every PNG icon and the logo from the SVG files in assets/.
# Needs one of: rsvg-convert (librsvg2-bin), inkscape, or ImageMagick.
set -euo pipefail

cd "$(dirname "$0")/.."

SVG="assets/icons/linux-stress-oc.svg"
SIZES="16 22 24 32 48 64 128 256 512"

render() { # render <input.svg> <output.png> <width> <height>
    if command -v rsvg-convert >/dev/null; then
        rsvg-convert -w "$3" -h "$4" "$1" -o "$2"
    elif command -v inkscape >/dev/null; then
        inkscape "$1" -w "$3" -h "$4" -o "$2" >/dev/null
    elif command -v magick >/dev/null; then
        magick -background none -density 384 "$1" -resize "${3}x${4}" "$2"
    elif command -v convert >/dev/null; then
        convert -background none -density 384 "$1" -resize "${3}x${4}" "$2"
    else
        echo "Install rsvg-convert, inkscape or ImageMagick first." >&2
        exit 1
    fi
}

for size in $SIZES; do
    dir="assets/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$dir"
    render "$SVG" "$dir/linux-stress-oc.png" "$size" "$size"
done

mkdir -p assets/icons/hicolor/scalable/apps
cp "$SVG" assets/icons/hicolor/scalable/apps/linux-stress-oc.svg
render assets/logo.svg assets/logo.png 1200 360

echo "Icons updated."
