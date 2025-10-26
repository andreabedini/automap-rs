//! # automap
//!
//! A Rust library for controlling Novation ZeRO MkII hardware via USB MIDI.
//!
//! This crate provides:
//! - USB device communication layer for Novation ZeRO MkII (VID:PID 1235:000c)
//! - Protocol encoding/decoding for commands and events
//! - MIDI codec for USB-MIDI packet conversion
//!
//! ## Example
//!
//! ```no_run
//! use automap::{AutomapDevice, AutomapCommand};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut device = AutomapDevice::new().await?;
//!
//!     // Turn off all LEDs
//!     let cmd = AutomapCommand::AllLedsOff;
//!     device.send_command(&cmd).await?;
//!
//!     Ok(())
//! }
//! ```

// Ensure exactly one runtime feature is enabled
#[cfg(all(feature = "tokio", feature = "smol"))]
compile_error!("Features 'tokio' and 'smol' are mutually exclusive. Enable only one.");

#[cfg(not(any(feature = "tokio", feature = "smol")))]
compile_error!("Must enable exactly one runtime feature: 'tokio' or 'smol'");

pub mod automap;
pub(crate) mod midi;

// Re-export commonly used types for convenience
pub use automap::protocol::{
    cc::{Button, Encoder, EncoderPosition, RingMode, RowSelect, RowSelectLhSet, RowSelectRhSet},
    command::AutomapCommand,
    event::AutomapEvent,
    sysex::{AutomapSysEx, LcdClear, LcdLine, LcdOp},
};
pub use automap::{AutomapDevice, USB_BUF};
