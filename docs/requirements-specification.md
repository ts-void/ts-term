# Requirements Specification: Lightweight Terminal Emulator in Rust

**Technical Specification · Version 1.0 · 2026-10-31**

## 1. Goal and Motivation

The goal is to develop a custom, lightweight terminal emulator in Rust that is functionally and architecturally modeled on projects such as [foot](https://codeberg.org/dnkl/foot) and [Alacritty](https://github.com/alacritty/alacritty).

The focus is on:

- Low memory consumption.
- Small executable binary.
- Fast startup time.
- Efficient CPU-based rendering.
- As few external dependencies as possible.
- Correct support for common terminal applications.

The emulator is to run exclusively under Wayland. X11 support is explicitly not planned.

## 2. Mandatory Requirements

| Area | Requirement |
|---|---|
| Programming language | Rust |
| Windowing system | Wayland only |
| X11 | No support |
| Rendering | CPU-based |
| Graphics API | No OpenGL, no Vulkan |
| Buffer | Wayland SHM |
| Rendering optimization | Damage tracking |
| Escape parser | `vte` |
| Configuration | TOML |
| TOML library | `basic-toml` |
| Terminal model | Character and cell grid |
| Process management | PTY with shell |
| Unicode | UTF-8 support |
| C library | Compilable and executable on glibc-based and musl-based Linux systems |

## 3. Quality Goals

The following values are guidelines and not mandatory acceptance criteria.

| Property | Goal |
|---|---:|
| Binary size | Preferably under 1 MB |
| Idle RAM usage | A few MB |
| Startup time | Under 20 ms |
| Idle CPU usage | Preferably close to 0 % |
| Dependencies | As few as possible |
| Rendering | Update damaged areas only |

Optimization is carried out incrementally. Functionality and correct terminal emulation take priority over maximum size and runtime optimization at first.

## 4. Technical Architecture

The software is built in a modular fashion. Terminal emulation, process management, rendering, and the Wayland interface are kept separate from one another.

### 4.1 Architecture Overview

```text
                 ┌────────────────────────────┐
                 │      Wayland Frontend      │
                 │ Window · Keyboard · Mouse  │
                 │ Clipboard · Resize         │
                 └─────────────┬──────────────┘
                               │
                 ┌─────────────┴──────────────┐
                 │                            │
        ┌────────▼─────────┐        ┌─────────▼────────┐
        │   CPU Renderer   │        │   Terminal Core  │
        │ Font · SHM       │        │ Parser · Grid    │
        │ Damage Tracking  │        │ Cursor · Modes   │
        └──────────────────┘        └─────────┬────────┘
                                              │
                                   ┌──────────▼──────────┐
                                   │     PTY Backend     │
                                   │ PTY · Shell · I/O   │
                                   │ Process management  │
                                   └──────────┬──────────┘
                                              │
                                   ┌──────────▼──────────┐
                                   │     Child Process   │
                                   │ Shell / Applications│
                                   └─────────────────────┘
```

**Basic principle:** The Terminal Core knows neither Wayland nor the concrete rendering implementation. This allows the terminal emulation to be tested independently of the graphical interface.

### 4.2 Proposed Project Structure

```text
terminal-emulator/
├── Cargo.toml
├── README.md
├── docs/
│   └── requirements-specification.md
└── src/
    ├── main.rs
    ├── config.rs
    │
    ├── pty/
    │   ├── mod.rs
    │   └── process.rs
    │
    ├── terminal/
    │   ├── mod.rs
    │   ├── parser.rs
    │   ├── grid.rs
    │   ├── cell.rs
    │   ├── cursor.rs
    │   ├── modes.rs
    │   ├── colors.rs
    │   └── scrollback.rs
    │
    ├── render/
    │   ├── mod.rs
    │   ├── cpu.rs
    │   ├── font.rs
    │   └── damage.rs
    │
    └── wayland/
        ├── mod.rs
        ├── window.rs
        ├── input.rs
        └── clipboard.rs
```

The structure is an architectural proposal. Modules may be adjusted during development.

## 5. Technical Components and Dependencies

| Area | Crate | Status |
|---|---|---|
| Escape parser | `vte` | Fixed |
| TOML | `basic-toml` | Fixed |
| Wayland client | `wayland-client` | Planned |
| Wayland protocols | `wayland-protocols` | Planned |
| SHM management | `memmap2` | Planned |
| PTY | `rustix` | Preferred |
| Keyboard | `xkbcommon` | Planned |
| Font rendering | `fontdue` | Planned |
| Serialization | `serde` | Optional |

The actual dependencies are to be reduced to the necessary minimum.

In particular, it must be checked which Wayland functionality is already covered by the selected crates in order to avoid duplicate dependencies.

## 6. PTY and Process Management

The PTY provides the connection between the terminal emulator and the shell or terminal application.

### Requirements

- Opening a pseudoterminal.
- Starting a configurable shell.
- Forwarding keyboard input.
- Reading PTY output.
- Transmitting the terminal size.
- Handling EOF and process termination.
- Cleanly terminating the shell.
- Restoring the terminal state when the emulator exits.

Process management must not block the Wayland event loop.

PTY input and output should therefore be handled via non-blocking file descriptors or suitable event integration.

## 7. Terminal Core and Escape Sequences

The Terminal Core is responsible for the actual terminal emulation.

### 7.1 Parser

`vte` is used as the parser.

The parser recognizes, among other things:

- Control characters.
- CSI sequences.
- OSC sequences.
- DCS sequences.
- UTF-8 characters.

The interpretation of the sequences is performed by a custom handler.

`vte` explicitly does not take over the management of the terminal state.

### 7.2 Terminal Grid

The screen is represented as a two-dimensional cell grid.

A cell contains at least:

- Character or text reference.
- Foreground color.
- Background color.
- Text attributes.

Example of an initial data model:

```rust
#[derive(Clone, Copy)]
struct Cell {
    character: char,
    foreground: Color,
    background: Color,
    attributes: Attributes,
}

#[derive(Clone, Copy)]
enum Color {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Default)]
struct Attributes {
    bold: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
    strikethrough: bool,
}
```

The model is a starting point. For full Unicode support, it must be extended with additional information.

### 7.3 Cursor

The Terminal Core must manage the following cursor properties:

- Position (row and column).
- Visibility.
- Shape.
- Cursor color, where required.
- Cursor movement via escape sequences.

### 7.4 Colors and Attributes

The following are to be supported:

- Default colors.
- 16 ANSI colors.
- 256-color mode.
- 24-bit RGB colors.
- Bold.
- Italic.
- Underline.
- Inverse.
- Strikethrough.

### 7.5 Terminal Modes

In particular, the following modes must be taken into account:

- Alternate Screen.
- Application Cursor Keys.
- Bracketed Paste.
- Cursor Visibility.
- Origin Mode.
- Autowrap.
- Mouse Reporting.

## 8. Rendering

### 8.1 Basic Principle

Rendering is performed entirely on the CPU.

The generated image content is handed over to the compositor via Wayland SHM buffers.

No GPU-based graphics API is required.

### 8.2 Font Rendering

`fontdue` is used as the starting point.

The font is initially either configured statically or selected via the TOML configuration.

Glyphs are initially rasterized directly. A complex glyph cache is not necessary for the first version.

### 8.3 Damage Tracking

A dirty-flag system at row level is to be implemented.

Example:

```rust
struct TerminalGrid {
    rows: Vec<Row>,
}

struct Row {
    cells: Vec<Cell>,
    dirty: bool,
}
```

When a change occurs, the affected row is marked as damaged.

During rendering, only damaged rows are redrawn.

Later optimizations:

1. Dirty cells instead of dirty rows.
2. Contiguous damage regions.
3. Separate cursor layer.
4. Optimized scroll rendering.
5. Bounded glyph caches.

Important: Damage tracking reduces CPU work during rendering. It does not replace the correct management of Wayland buffers and their synchronization.

## 9. Wayland Integration

The graphical interface is implemented exclusively for Wayland.

### Requirements

- Creating a Wayland window.
- Managing SHM buffers.
- Displaying the terminal content.
- Handling window resize.
- Processing keyboard events.
- Processing mouse events.
- Clipboard support.
- Integration into the event loop.

X11 support is provided neither directly nor via an additional compatibility layer.

## 10. Configuration

Configuration is done via TOML.

A possible structure:

```toml
[terminal]
shell = "/bin/bash"
scrollback = 10000

[font]
family = "monospace"
size = 12.0

[window]
width = 900
height = 600

[colors]
foreground = "#ffffff"
background = "#1e1e1e"

[behavior]
cursor_blink = true
```

The actual configuration options will be determined during development.

A simple configuration structure is sufficient for the first version. A custom TOML parser is explicitly deferred.

## 11. Development Phases

Development proceeds incrementally. Each phase should deliver an independently testable intermediate result.

### Phase 1 – PTY Prototype

**First milestone**

- Open PTY.
- Start shell.
- Read output.
- Forward input.
- Implement raw mode.
- Support resize.
- Terminate process cleanly.

**Result:** A shell can be addressed via the PTY.

### Phase 2 – Parser

- Integrate `vte`.
- Implement handler.
- Process control characters.
- Recognize CSI, OSC, and basic escape sequences.

**Result:** The PTY output can be interpreted.

### Phase 3 – Terminal Grid

- Cells and rows.
- Cursor.
- SGR colors.
- Text attributes.
- Line wrapping.
- Scrolling.
- Dirty flags.

**Result:** A functional terminal state without a graphical interface.

### Phase 4 – Headless Tests

- Output the terminal state as text.
- Test escape sequences.
- Verify cursor positions.
- Verify color and attribute processing.

**Result:** The Terminal Core is independently testable.

### Phase 5 – Wayland and SHM

- Wayland window.
- SHM buffer.
- Fixed font.
- CPU rendering.
- Damage tracking.

**Result:** The terminal content is displayed graphically.

### Phase 6 – Interaction

- Keyboard.
- Resize.
- Scrollback.
- Text selection.
- Clipboard.

**Result:** An interactively usable terminal emulator.

### Phase 7 – Compatibility

- Unicode widths.
- Combining characters.
- Alternate screen.
- Mouse reporting.
- Bracketed paste.
- Extended escape sequences.

**Result:** Support for more demanding terminal applications.

### Phase 8 – Optimization

- Binary size.
- Memory consumption.
- Startup time.
- Glyph cache.
- Rendering optimizations.

**Result:** Achieving the targeted quality goals.

## 12. Functional Acceptance Criteria

The following programs serve as practical compatibility tests:

| Program | Functionality to be verified |
|---|---|
| `ls --color` | ANSI colors and basic text display |
| `htop` | Cursor movement, screen updates, colors, and alternate screen |
| `vim` | Alternate screen, keyboard modes, cursor, and extensive escape sequences |

The programs should be operable without display errors. Compatibility is built up incrementally and is not required for the first runnable version.

## 13. Deferred Features

The following features are deliberately not part of the first implementation:

- Lazy glyph rasterization.
- Bounded glyph cache.
- Detailed damage tracking at pixel level.
- Advanced rendering optimizations.
- Custom TOML parser.
- Server/client mode.
- GPU acceleration.
- X11 support.

These features may be added later if they offer a measurable benefit.

## 14. Risks and Technical Challenges

| Risk | Significance | Countermeasure |
|---|---|---|
| Incomplete VT compatibility | Very high | Reference tests and incremental implementation |
| Unicode widths | High | Dedicated cell model |
| Alternate screen | High | Separate screen buffers |
| Wayland SHM synchronization | High | Manage buffer lifecycle cleanly |
| PTY/event loop integration | High | Non-blocking I/O |
| Binary size | Medium | Optimize only after functionality is in place |
| Memory consumption | Medium | Contiguous data structures |

### References

- [foot](https://codeberg.org/dnkl/foot) – Architectural template and reference implementation.
- [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) – Reference for escape sequences.
- [Wayland Protocols](https://gitlab.freedesktop.org/wayland/wayland-protocols) – Wayland protocols.
- [vte documentation](https://docs.rs/vte/latest/vte/) – Parser.
- [fontdue](https://github.com/mooman219/fontdue) – Font rasterization.
- [vttest](https://invisible-island.net/vttest/) – Terminal compatibility tests.

## 15. Summary of Technical Decisions

| Area | Decision |
|---|---|
| Language | Rust |
| Platform | Linux (glibc and musl) / Wayland |
| Windowing system | Wayland-only |
| Rendering | CPU |
| Buffer | SHM |
| Damage tracking | Row-based, finer-grained later |
| Parser | `vte` |
| Configuration | TOML / `basic-toml` |
| PTY | `rustix` preferred |
| Keyboard | `xkbcommon` |
| Font | `fontdue` |
| Terminal model | Custom grid |
| Architecture | Modular and renderer-independent |
