# automap-rs

[![CI](https://github.com/andreabedini/automap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/andreabedini/automap-rs/actions/workflows/ci.yml)

A Rust library for controlling Novation ZeRO MkII hardware via USB MIDI.

> **⚠️ Early Development:** This library is in active development. The API may change.

## Features

- Bidirectional USB communication with Novation ZeRO MkII (VID:PID 1235:000c)
- Control LEDs, encoder rings, and LCD displays
- Receive events from buttons, encoders, pots, sliders, and touch sensors
- Type-safe protocol encoding/decoding
- Runtime-agnostic: supports both tokio and smol async runtimes

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
# Using smol runtime (default, lightweight)
automap = { git = "https://github.com/andreabedini/automap-rs" }

# Or using tokio runtime
automap = { git = "https://github.com/andreabedini/automap-rs", default-features = false, features = ["tokio"] }
```

**Note:** The `tokio` and `smol` features are mutually exclusive. The library will fail to compile if both are enabled.

## Runtime Support

automap-rs supports both tokio and smol async runtimes via feature flags, with **smol as the default** for its lightweight footprint.

## Quick Start

```rust
use automap::{AutomapDevice, AutomapCommand, AutomapEvent, AutomapSysEx, LcdOp, LcdClear, LcdLine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let mut device = AutomapDevice::new().await?;

        // Bring device online
        device.send_sysex(AutomapSysEx::OnlineOffline { online: true }).await?;

        // Display "Hello" on the LCD
        device.send_sysex(AutomapSysEx::LcdText(vec![
            LcdOp::Clear(LcdClear::LeftAll),
            LcdOp::Cursor { col: 0, line: LcdLine::LeftTop },
            LcdOp::Text(b"Hello, World!"),
            LcdOp::End,
        ])).await?;

        // Turn off all LEDs
        device.send_command(&AutomapCommand::AllLedsOff).await?;

        // Read and respond to button presses
        loop {
            let events = device.read_events().await?;
            for event in events {
                if let AutomapEvent::Button { button, pressed } = event {
                    // Echo button state to LED
                    device.send_command(&AutomapCommand::ButtonLed { button, on: pressed }).await?;
                }
            }
        }
    })
}
```

**See also:** `examples/demo_tokio.rs` and `examples/demo_smol.rs` for complete working examples with graceful shutdown.

## Development

```bash
# Build with smol (default)
cargo build

# Build with tokio
cargo build --no-default-features --features tokio

# Run examples
cargo run --example demo_smol
cargo run --example demo_tokio --no-default-features --features tokio

# Run tests
cargo test                                          # with smol
cargo test --no-default-features --features tokio   # with tokio
```

## Hardware Requirements

- **Novation ZeRO MkII** MIDI controller (discontinued, but available secondhand)
- USB connection
- Linux, macOS, or Windows (any platform supported by [nusb](https://github.com/kevinmehall/nusb))

**Note:** This library communicates with the device's "hidden" vendor-specific USB interface (Interface 2), not the standard MIDI interface. This allows direct control of hardware features like LEDs and LCD displays that aren't accessible via standard MIDI.

## Architecture

The library is organized into three layers:

- **USB Device Layer** (`device.rs`): Runtime-agnostic async I/O using nusb
- **Protocol Layer** (`protocol/`): Type-safe command encoding, event decoding, and SysEx operations
- **MIDI Codec Layer** (`midi.rs`): USB-MIDI packet conversion utilities

The design maintains strict separation between semantic domain models and wire encodings, ensuring type safety and maintainability.

## Documentation

- API documentation: Run `cargo doc --open` to browse the generated docs
- Protocol details: See `docs/` directory for Novation MIDI protocol documentation
- Architecture guide: See `CLAUDE.md` for detailed architectural information

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Acknowledgments

Protocol implementation based on Novation's MIDI Programmer's Reference documentation.

## License

Licensed under the [MIT license](LICENSE).
