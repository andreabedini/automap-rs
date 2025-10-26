# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

automap-rs is a Rust library for controlling Novation ZeRO MkII hardware via USB MIDI. The project is structured as:
- **Library crate** (`src/lib.rs`) - Core functionality for USB device communication and protocol handling
- **Example applications** (`examples/demo_tokio.rs`, `examples/demo_smol.rs`) - Interactive demonstrations using different async runtimes

The library provides bidirectional communication: receiving events from physical controls (buttons, encoders, pots, sliders) and sending commands to control LEDs and LCD displays.

The library supports both **tokio** and **smol** async runtimes via feature flags, with **smol as the default**.

## Common Development Commands

### Build
```bash
cargo build         # Build with smol (default)
cargo build --release  # Release build
cargo build --no-default-features --features tokio  # Build with tokio
cargo build --lib   # Build library only
```

### Run
```bash
# Run with smol runtime (default)
cargo run --example demo_smol

# Run with tokio runtime
cargo run --example demo_tokio --no-default-features --features tokio
```

### Testing
```bash
cargo test         # Run tests with smol (default)
cargo test --no-default-features --features tokio  # Run tests with tokio
cargo test --lib   # Run library tests only
cargo test <test_name>  # Run specific test (e.g., cargo test roundtrip_lcd)
```

### Linting
```bash
cargo clippy       # Run lints
cargo fmt          # Format code
cargo check        # Fast syntax/type checking without codegen
```

## Version Control

This project uses **Jujutsu (jj)** for version control, not Git. Jujutsu is a modern VCS with a different workflow:

```bash
jj status          # Show working copy status
jj diff            # Show changes in working copy
jj commit -m "..."  # Create a new commit (doesn't move working copy)
jj describe -m "..." # Amend current change description
jj log             # View change history
jj new             # Create a new change on top of current
```

While the repository maintains Git compatibility (you can see `.git` artifacts), prefer using `jj` commands over `git` commands when working in this repository.

## Runtime Support

The library supports both **tokio** and **smol** async runtimes via feature flags:

### Using with smol (default)
```toml
[dependencies]
automap = "0.1"
```

### Using with tokio
```toml
[dependencies]
automap = { version = "0.1", default-features = false, features = ["tokio"] }
```

**Key points:**
- Features are **mutually exclusive** - only one runtime can be enabled at a time
- The library enforces this at compile-time with `compile_error!` checks
- smol is the default for its lightweight footprint and efficiency
- Both runtimes are supported natively by the underlying `nusb` USB library
- Only `src/automap/device.rs` contains runtime-specific code; all protocol/MIDI layers are runtime-agnostic

See `examples/demo_tokio.rs` and `examples/demo_smol.rs` for complete working examples.

## Architecture

The codebase is organized in three architectural layers:

### 1. USB Device Layer (`src/automap/device.rs`)
- **AutomapDevice** manages USB communication with Novation ZeRO MkII (VID:PID 1235:000c)
- Uses `nusb` library with runtime-agnostic async I/O (tokio or smol)
- Communicates via Interface 2, Endpoints IN=0x86, OUT=0x06
- Provides high-level async methods:
  - `send_command()` - Send LED/encoder control commands
  - `send_sysex()` - Send SysEx messages (LCD, templates, etc.)
  - `read_events()` - Read and decode hardware events
- Automatically handles USB-MIDI packet encoding/decoding
- Runtime selection via conditional compilation (`#[cfg(feature = "tokio")]` / `#[cfg(feature = "smol")]`)

### 2. Protocol Layer (`src/automap/protocol/`)
Multi-file protocol implementation with encode/decode separation:

- **command.rs** - Host→Device commands (ButtonLed, EncoderRingMode, RowSelectLed, AllLedsOff)
  - Commands are enums with `to_bytes()` encoding to MIDI CC messages on channel 15 (0xBF)
  - Each command variant knows its own wire format

- **event.rs** - Device→Host events (Button, Encoder, Pot, Slider, Touch, Transport)
  - Events are enums with `decode_event()` parsing MIDI CC/channel messages
  - Stateless decoding from raw MIDI bytes

- **sysex.rs** - SysEx protocol for complex operations
  - Two protocol families: Automap (0x03:0x03) and DbSim (0x05)
  - LCD operations: clear, cursor positioning, text display
  - Data-block read/write for templates/globals/controls
  - Parameter requests and simulation commands
  - Full encode (`to_bytes()`) and decode (`from_bytes()`) with error handling

- **cc.rs** - MIDI CC definitions (button IDs, slider IDs, pot IDs, encoder IDs, transport controls)

- **template.rs** - Template data structures for device configuration

**Key Pattern**: Commands and events are strongly-typed Rust enums, not raw byte manipulation. The protocol layer handles all wire-format details.

### 3. MIDI Codec Layer (`src/midi.rs`)
Internal USB-MIDI packet format conversion (not exposed in public API):
- `usbmidi_pack()` - Raw MIDI bytes → 4-byte USB-MIDI packets
- `usbmidi_unpack()` - USB-MIDI packets → raw MIDI bytes
- `split_midi_messages()` - Parse continuous byte stream into complete MIDI messages
- Implements USB Device Class Definition for MIDI Devices v1.0
- Used internally by AutomapDevice; consumers use high-level methods instead

### Data Flow

```
Reading Hardware Events:
USB → read_events() → AutomapEvent enum
     (internally: USB-MIDI packets → raw MIDI bytes → decode)

Controlling Hardware:
AutomapCommand → send_command() → USB
AutomapSysEx   → send_sysex()   → USB
     (internally: encode → MIDI bytes → USB-MIDI packets)
```

## Design Principles

### Separation of Domain Model from Wire Encoding

**Critical principle**: Commands and events represent *semantic meaning*, not wire format.

The protocol layer maintains a strict separation between:
- **Domain model**: What the command/event *means* (command variants, semantic types)
- **Wire encoding**: How it's *transmitted* (bytes, bit patterns, encoding logic)

This is enforced through:

1. **Semantic types in cc.rs**:
   - `EncoderPosition` - plain enum for encoder positions (0-11) with constants (MIN, MAX, CENTER)
   - `RowSelectLhSet` / `RowSelectRhSet` - bitflags for LED sets (e.g., `RS1 | RS3 | RS5`)
   - Not raw `u8` or bit patterns in command types

2. **Encoding isolated in `encode_into()`**:
   - All wire format knowledge lives in encoding methods
   - Command structs remain clean and type-safe
   - Example: `EncoderPosition` variants are cast to wire format only during encoding

**Example of good separation**:
```rust
// ✅ Good: Semantic API with plain enum
let cmd = AutomapCommand::EncoderRingValue {
    encoder: Encoder::Encoder1,
    position: EncoderPosition::Pos6,  // or EncoderPosition::CENTER
};

// ❌ Bad: Wire format in domain model
let cmd = AutomapCommand::EncoderRingValue {
    encoder: Encoder::Encoder1,
    pos_0_to_11: 6,  // Magic number, easy to get wrong
};
```

**Benefits**:
- Compile-time validation (can't create invalid positions)
- Self-documenting code (`EncoderPosition::CENTER` vs `6`)
- Easy to change wire format without touching API
- Less error-prone (type system prevents mistakes)

## Working with Protocol Extensions

When adding new commands or events:

1. **Define enum variant** in `command.rs` or `event.rs`
2. **Implement encoding/decoding** - add match arm to `to_bytes()` or `decode_event()`
3. **Add CC definitions** in `cc.rs` if needed
4. **Write tests** - use existing test patterns (see `test_button_led()`, `roundtrip_lcd()`)
5. **Update examples** if demonstrating new functionality

The demo example in `examples/demo.rs` demonstrates library usage interactively. It:
- Initializes the device (sends OnlineOffline{online: true})
- Displays "Hello" on the LCD
- Echoes button presses by toggling corresponding LEDs
- Gracefully shuts down on Ctrl+C (sends OnlineOffline{online: false})

The library exports all protocol types and utilities via `src/lib.rs`, making them available to external applications.

## Hardware Context

**Target Device**: Novation ZeRO MkII
- USB identifiers: VID=0x1235, PID=0x000c
- "Hidden" programming interface on Interface 2
- Physical controls: 16 buttons, 8 encoders, 8 pots, 8 sliders, touch sensors
- Display: LCD with multiple lines and cursor control
- Protocol documented in "Novation MIDI Programmer's Reference" (referenced in code comments)

## Dependencies

Key external crates and their purposes:
- **tokio** - Async runtime (current_thread flavor in binary)
- **nusb** - USB device communication with tokio support
- **bitflags** - Protocol flag types (ControlAttribute, ButtonAttribute, etc.)
- **derive_more** - Ergonomic derive macros (Debug, TryFrom)
- **midir** - MIDI I/O (partially integrated, some code commented)

Uses Rust edition 2024 (requires recent toolchain).
