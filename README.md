<p align="center">
  <img src="assets/logo.png" alt="LinStress" width="600">
</p>

# LinStress(GPUI)

A native Rust app with a GPUI interface.

## Features
- Live CPU usage, frequency, CPU temperature, GPU temperature
- CPU stress test (choose the thread count)
- RAM stress test (writes patterns, verifies them, counts errors)
- GPU stress test (Vulkan compute shader, works on NVIDIA, AMD and Intel)
- Automatic safety stop for all tests at a temperature you choose
- CPU max-frequency limit and governor control (cpufreq)
- NVIDIA power/clock control through `nvidia-smi`

## Project layout
| File | Purpose |
|---|---|
| `src/main.rs` | Starts the app and opens the window |
| `src/app.rs` | App state, button logic, page layout |
| `src/theme.rs` | Colors and window sizes |
| `src/components/` | One file per piece of the interface (see below) |
| `src/sensors.rs` | Reads /proc and /sys sensors |
| `src/cpu_stress.rs` | CPU stress worker threads |
| `src/ram_stress.rs` | RAM pattern test with error counting |
| `src/gpu_stress.rs` | GPU compute-shader stress test (wgpu) |
| `src/overclock.rs` | Frequency, governor and NVIDIA tuning |
| `assets/` | Icon (SVG + PNG sizes) and logo |
| `packaging/` | Desktop entries, AppStream info, root launcher |
| `scripts/` | Dependency installer, AppImage and icon scripts |

### Components
| File | What it draws |
|---|---|
| `button.rs`, `stepper.rs`, `metric_card.rs`, `panel.rs` | Small reusable building blocks |
| `header.rs` | Title bar, status badge, "Stop all" |
| `metrics.rs` | Row of live value cards |
| `cpu_panel.rs`, `ram_panel.rs`, `gpu_panel.rs` | The three stress tests |
| `safety_panel.rs` | Safety temperature |
| `tuning_panel.rs` | CPU frequency and governor |
| `log_panel.rs` | Recent messages |

To add a new section: create a file in `components/`, write an
`impl StressApp { pub fn my_panel(...) -> Div }` method, register it in
`components/mod.rs`, and call it from `render` in `app.rs`.

## Build
```bash
make deps      # installs system libraries (apt, dnf or pacman)
make run       # development build, starts the app
make build     # optimized release build
make run-root  # release build, run as root for tuning
make dist      # maximum optimization (slow to compile)
make lint      # format check + clippy
```
Run `make help` to list every command. The pinned Rust toolchain is
installed automatically by `rustup` from `rust-toolchain.toml`.

## Packages (.deb, .rpm, AppImage)
```bash
make tools      # once: installs cargo-deb and cargo-generate-rpm
make packages   # creates all three in dist/
```
Or one at a time: `make deb`, `make rpm`, `make appimage`.

| Package | Install |
|---|---|
| `.deb` | `sudo apt install ./dist/linstress_*.deb` |
| `.rpm` | `sudo dnf install ./dist/linstress-*.rpm` |
| AppImage | `chmod +x dist/*.AppImage && ./dist/*.AppImage` |

- The `.deb` and `.rpm` add an application-menu entry and a second entry,
  "LinStress (Administrator)", which asks for your password through
  polkit and starts the app as root so the tuning buttons work.
- The AppImage does not include system libraries (Vulkan, Wayland, xkbcommon).
  Build it on an older distribution such as Ubuntu 22.04 for best compatibility,
  and start it with `sudo -E` when you need the tuning buttons.
- Before releasing, edit the maintainer name and e-mail in `Cargo.toml`
  (`authors` and `[package.metadata.deb]`) and `packaging/*.xml`.

## Icon and logo
Sources are in `assets/` (`icons/linstress.svg` and `logo.svg`).
After changing an SVG run `make icons` to re-create all PNG sizes.

### Build profiles
| Profile | Use | LTO | Codegen units |
|---|---|---|---|
| `dev` | daily work, GPUI dependencies still optimized | off | default |
| `release` | normal builds | thin | 8 |
| `dist` | published binaries | fat | 1 |

### Faster linking (optional)
Install `mold` and `clang`, then uncomment the linker lines in `.cargo/config.toml`.

## Run tuning features
Writing to /sys needs root:
```bash
make run-root
```

## GPU test requirements
The GPU test needs a working Vulkan driver. Check with `vulkaninfo --summary`.
Install your GPU's Vulkan driver first (NVIDIA proprietary driver, or Mesa
for AMD and Intel). A software renderer such as llvmpipe is rejected on purpose.

## Safety
- Frequencies are clamped to the hardware limits.
- Real overclocking (multiplier, voltage) is done in the BIOS; Linux only changes limits.
- The stress test stops automatically at the safety temperature (default 90 °C).
- Watch your temperatures and use at your own risk.
