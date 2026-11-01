#!/usr/bin/env bash

set -euo pipefail

# PTY
mkdir -p src/pty

# Terminal Core
mkdir -p src/terminal

# Rendering
mkdir -p src/render

# Wayland
mkdir -p src/wayland

# Dateien anlegen, falls sie noch nicht existieren
files=(
    src/config.rs

    src/pty/mod.rs
    src/pty/process.rs

    src/terminal/mod.rs
    src/terminal/parser.rs
    src/terminal/grid.rs
    src/terminal/cell.rs
    src/terminal/cursor.rs
    src/terminal/modes.rs
    src/terminal/colors.rs
    src/terminal/scrollback.rs

    src/render/mod.rs
    src/render/cpu.rs
    src/render/font.rs
    src/render/damage.rs

    src/wayland/mod.rs
    src/wayland/window.rs
    src/wayland/input.rs
    src/wayland/clipboard.rs
)

for file in "${files[@]}"; do
    touch "$file"
done

echo "Projektstruktur erfolgreich angelegt."

