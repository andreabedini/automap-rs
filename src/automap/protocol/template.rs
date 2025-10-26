#![allow(dead_code)]

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlType {
    Spare = 0,
    CC = 1,
    NRPN = 2,
    RPN = 3,
    SysEx = 4,
    MMC = 5,
    NoteOn = 6,
    NoteOff = 7,
    BankSelect = 8,
    ProgChange = 9,
    PitchBend = 10,
    DrumNote = 11,
    TempChange = 12,
    RealTime = 13,
    TempGroup = 14,
}
// DisplayType (field CNDISP in Novation’s terminology) tells the SL Mk II’s firmware how a control’s current value should appear on the LCD when you select or touch it. It doesn’t affect the actual MIDI data; it’s purely a presentation hint stored in the template.
//
// According to the SL Control Members and MIDI Programmer’s Reference documents:
//
// Each physical control (pot, slider, button, encoder) has an entry in the template memory block.
//
// That entry contains a CNDISP byte specifying one of the “FT…” (format type) values—these are what we modelled as the DisplayType enum.
//
// The firmware uses that code to choose what to draw in the right-hand text cell of the LCD whenever the control’s value changes.
//
// Examples from the manual :
//
// DisplayType code	LCD behaviour	Typical control type
// FT127 (0)	Show numeric 0–127	knobs, sliders
// FTOFFON (3)	Show “OFF” / “ON” text	toggle buttons
// FTREL1 (6)	Relative 1-LED ring / incremental value	encoders
// FTNOTE (8)	Show musical note name (C3, F#4 …)	keyboard note assignment
// FTLED (16)	LED text (“ ” / “ON”)	LED indicators
// FTVPOT (17)	Virtual pot (bar-graph style)	continuous encoders
// FTLABEL (5 or 15)	Display static text label	decorative / grouping
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayType {
    Ft127 = 0,   // 0..127 numeric
    Ft6463 = 1,  // 64/63 style
    FtMmc = 2,   // MMC
    FtOffOn = 3, // OFF/ON boolean
    FtBlank = 4, // blank field
    FtLabel = 5, // show CNNAME when no control
    FtRel1 = 6,  // relative encoder display (type 1)
    FtRel2 = 7,  // relative encoder display (type 2)
    FtNote = 8,  // note name
    Ft16k = 9,   // 14-bit encoder display
    // 10..14 are unused in the public table
    FtLabel2 = 15, // CNNAME + string in CN-SXBUFFER
    FtLed = 16,    // " " / "ON" LED text
    FtVpot = 17,   // logic VPOT
                   // FtEnd = 17 in the doc; kept for completeness
}

// ===================== PORT ROUTING (CNPORTS) =====================
// Two ways exist in firmware: the "type 6+5" scheme *or* the simple "specific" codes.
// We expose both: a precise enum for the specific codes and bitflags for the mask.

/// Concrete CNPORTS "specific" codes (type bits already set to 'Specific').
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortRoute {
    None = 0x40,        // don't transmit
    MidiOut1 = 0x41,    // M1 Out
    MidiOut2 = 0x42,    // M2 Out
    Usb1 = 0x44,        // USB Port1
    Usb2 = 0x48,        // USB Port2
    Usb3Hidden = 0x50,  // USB Port3 (Automap hidden)
    Usb1AndUsb3 = 0x54, // USB1 + USB3
}

// Bit positions for the lower 5 bits when using the CNPORTS "Specific" mask form.
// (Only meaningful if the top bits encode 'Specific' per the "type 6+5" scheme.)
bitflags::bitflags! {
    pub struct PortBits: u8 {
        const M1_OUT    = 1 << 0;
        const M2_OUT    = 1 << 1;
        const USB1      = 1 << 2;
        const USB2      = 1 << 3;
        const USB3_HID  = 1 << 4;
    }
}

/// The top-bits selector in the “type 6+5” encoding (for completeness).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortType {
    Common = 0x00,   // 0Xh: common ports (low bits ignored/cleared)
    Keyboard = 0x20, // 2Xh: keyboard ports (low bits ignored/cleared)
    Specific = 0x40, // 4Xh: specific → low bits indicate ports
                     // 6Xh invalid
}

/// Helper: build a CNPORTS byte from PortType + PortBits (only use bits with Specific).
pub const fn cnports(port_type: PortType, bits: PortBits) -> u8 {
    (port_type as u8) | (bits.bits() & 0x1F)
}

// ===================== MIDI CHANNEL SPEC (CNMCHAN) =====================
/// CNMCHAN can be "Common", "Keyboard", or "Specific ch1..16".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelSpec {
    Common,      // 0x00 → get channel from ComnChan
    Keyboard,    // 0x20 → get channel from KeybChan
    Channel(u8), // 1..=16 → encoded as 0x40..0x4F
}

impl ChannelSpec {
    /// Encode to CNMCHAN byte.
    pub fn to_byte(self) -> u8 {
        match self {
            ChannelSpec::Common => 0x00,
            ChannelSpec::Keyboard => 0x20,
            ChannelSpec::Channel(n) => {
                debug_assert!((1..=16).contains(&n));
                0x40 + (n - 1)
            }
        }
    }
    /// Decode CNMCHAN byte back to ChannelSpec.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(ChannelSpec::Common),
            0x20 => Some(ChannelSpec::Keyboard),
            0x40..=0x4F => Some(ChannelSpec::Channel((b - 0x40) + 1)),
            _ => None, // includes invalid 0x60.. etc.
        }
    }
}
