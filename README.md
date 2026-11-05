# ts-term

A lightweight, Wayland-only terminal emulator written in [Rust](https://rust-lang.org/), inspired by [foot](https://codeberg.org/dnkl/foot).

> **Status:** Planning / early development. Nothing is usable yet.

## Purpose

It's a learning project, with the quiet hope that it will become a functional, lightweight terminal emulator.

## Goals

- Low memory usage, small binary, fast startup
- CPU-based rendering via Wayland SHM with damage tracking
- Few external dependencies
- Runs on both glibc and musl based Linux systems

## Maybe later

- Server/client mode (for now)

## Non-goals

- X11 support
- GPU acceleration

## Documentation

See [docs/requirements-specification.md](docs/requirements-specification.md) for the full requirements, architecture, and development phases.

## Building

Not available yet. Once the project builds, it will use the standard Cargo workflow:

```sh
cargo build --release
```

## Installation

tba
