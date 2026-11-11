#!/usr/bin/env bash
# Installs the system libraries GPUI needs on Linux.
set -euo pipefail

if command -v apt-get >/dev/null; then
    echo "Detected Debian/Ubuntu"
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config clang cmake \
        libxkbcommon-x11-dev libwayland-dev libxcb1-dev libvulkan-dev \
        libfontconfig1-dev libssl-dev libasound2-dev \
        mesa-vulkan-drivers vulkan-tools
elif command -v dnf >/dev/null; then
    echo "Detected Fedora"
    sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config clang cmake \
        libxkbcommon-x11-devel wayland-devel libxcb-devel vulkan-loader-devel \
        fontconfig-devel openssl-devel alsa-lib-devel \
        mesa-vulkan-drivers vulkan-tools
elif command -v pacman >/dev/null; then
    echo "Detected Arch"
    sudo pacman -S --needed --noconfirm base-devel clang cmake \
        libxkbcommon-x11 wayland libxcb vulkan-icd-loader fontconfig openssl alsa-lib vulkan-tools
else
    echo "Unknown distribution. Please install the dependencies listed in README.md manually."
    exit 1
fi

if ! command -v cargo >/dev/null; then
    echo "Rust is not installed. Get it from https://rustup.rs"
    exit 1
fi

echo "Done. Run 'make run' to start the app."
