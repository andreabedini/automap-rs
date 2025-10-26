use derive_more::{Debug, TryFrom};

pub const AUTOMAP_CC_STATUS: u8 = 0xBF;

#[derive(TryFrom, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum RingMode {
    ContinuousCw = 0x00,
    ContinuousAcw = 0x10,
    CenteredBand = 0x20,
    DoubleCenter = 0x30,
    SingleLedCw = 0x40,
}

bitflags::bitflags! {
    /// Control attribute byte 1 flags (CNATTR1)
    pub struct Attr1: u8 {
        const SEND_MSB_FIRST = 1 << 0;
        const SEND_2B_VALUE  = 1 << 1;
        const SEND_ON_RELEASE= 1 << 2;
        const TOGGLE_VALUE   = 1 << 3;
        const CYCLIC_BUTTON  = 1 << 4;
        const RAWDATA_MODE   = 1 << 6;
    }

    /// Control attribute byte 2 flags (CNATTR2)
    pub struct Attr2: u8 {
        const SNAPSHOT_SKIP  = 1 << 2;
        const INVERT_VALUE   = 1 << 3;
        const POTMODE_JUMP    = 0b00 << 5;
        const POTMODE_PICKUP  = 0b01 << 5;
        const POTMODE_GLOBAL  = 0b10 << 5;
        const POTMODE_TEMPLATE= 0b11 << 5;
    }
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum Pot {
    Pot1 = 0x08,
    Pot2 = 0x09,
    Pot3 = 0x0A,
    Pot4 = 0x0B,
    Pot5 = 0x0C,
    Pot6 = 0x0D,
    Pot7 = 0x0E,
    Pot8 = 0x0F,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum Slider {
    Slider1 = 0x10,
    Slider2 = 0x11,
    Slider3 = 0x12,
    Slider4 = 0x13,
    Slider5 = 0x14,
    Slider6 = 0x15,
    Slider7 = 0x16,
    Slider8 = 0x17,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum Button {
    ButtonA1 = 0x18,
    ButtonA2 = 0x19,
    ButtonA3 = 0x1A,
    ButtonA4 = 0x1B,
    ButtonA5 = 0x1C,
    ButtonA6 = 0x1D,
    ButtonA7 = 0x1E,
    ButtonA8 = 0x1F,
    ButtonB1 = 0x20,
    ButtonB2 = 0x21,
    ButtonB3 = 0x22,
    ButtonB4 = 0x23,
    ButtonB5 = 0x24,
    ButtonB6 = 0x25,
    ButtonB7 = 0x26,
    ButtonB8 = 0x27,
    ButtonC1 = 0x28,
    ButtonC2 = 0x29,
    ButtonC3 = 0x2A,
    ButtonC4 = 0x2B,
    ButtonC5 = 0x2C,
    ButtonC6 = 0x2D,
    ButtonC7 = 0x2E,
    ButtonC8 = 0x2F,
    ButtonD1 = 0x30,
    ButtonD2 = 0x31,
    ButtonD3 = 0x32,
    ButtonD4 = 0x33,
    ButtonD5 = 0x34,
    ButtonD6 = 0x35,
    ButtonD7 = 0x36,
    ButtonD8 = 0x37,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum TransportButton {
    ButtonD1Tl = 0x48,
    ButtonD2Tl = 0x49,
    ButtonD3Tl = 0x4A,
    ButtonD4Tl = 0x4B,
    ButtonD5Tl = 0x4C,
    ButtonD6Tl = 0x4D,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum AutomapButton {
    AutomapButton1 = 0x48,
    AutomapButton2 = 0x49,
    AutomapButton3 = 0x4A,
    AutomapButton4 = 0x4B,
    AutomapButton5 = 0x4C,
    AutomapButton6 = 0x4D,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum RowSelect {
    L1 = 0x50,
    L2 = 0x51,
    L3 = 0x52,
    L4 = 0x53,
    L5 = 0x54,
    R1 = 0x56,
    R2 = 0x57,
}

#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum Encoder {
    Encoder1 = 0x78,
    Encoder2 = 0x79,
    Encoder3 = 0x7A,
    Encoder4 = 0x7B,
    Encoder5 = 0x7C,
    Encoder6 = 0x7D,
    Encoder7 = 0x7E,
    Encoder8 = 0x7F,
}

/// Physical controls on the Novation Zero SL Mk II.
#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum Controls {
    SustainPedal = 0x40,
    ExpressionPedal = 0x41,

    TouchpadX1 = 0x44, // Also crossfader
    TouchpadY1 = 0x45,
    TouchpadX2 = 0x46,
    TouchpadY2 = 0x47,

    ButtonD1TL = 0x48, // Rewind
    ButtonD2TL = 0x49, // Fast-Forward
    ButtonD3TL = 0x4A, // Stop
    ButtonD4TL = 0x4B, // Play
    ButtonD5TL = 0x4C, // Loop
    ButtonD6TL = 0x4D, // Record

    AllLedsOff = 0x4E, // NOT implemented on the RemoteSL or ZeroSL

    // === Transport ===
    TransportLock = 0x4F,

    PageUpL = 0x58,
    PageDnL = 0x59,
    PageUpR = 0x5A,
    PageDnR = 0x5B,

    Alerts = 0x5C,

    MSTempo = 0x5E,
    LSTempo = 0x5F,

    RowLhBitmap = 0x60,
    RowRhBitmap = 0x61,

    EchoRequest = 0x63, // Originally designed for the Reason Special-Template – no longer used.

    ParamRequest = 0x67,

    AvailableRowSelects = 0x68, // Originally designed for the Reason Special-Template – no longer used.
    AvailableRowSelects2 = 0x69, // Originally designed for the Reason Special-Template – no longer used.

    // === Speed Dial ===
    SpeedDialButton = 0x65,
    SpeedDial = 0x66,

    // === Touch Sensor Reports ===
    EncodersTouch = 0x6C,
    PotsTouch = 0x6D,
    SlidersTouch = 0x6E,
    SpeedDialTouch = 0x6F,

    // === Off/Online ===
    OffOnLine = 0x6B,
}

/// Page buttons for LCD navigation (Section 5, PDF page 10)
#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum PageButton {
    PageUpL = 0x58,
    PageDnL = 0x59,
    PageUpR = 0x5A,
    PageDnR = 0x5B,
}

/// Alert types sent by the device (Section 6, PDF page 10)
#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum AlertType {
    MidiChannelChanged = 0x00,
    KeyboardTransposeChanged = 0x01,
    OctaveChanged = 0x02,
    AfterTouchChanged = 0x03,
    VelocityCurveChanged = 0x04,
}

/// Parameter request types (Section 6, PDF page 13)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ParameterRequestType {
    UnitProductType = 0x00,
    TransportLockState = 0x01,
}

/// Product type response values (Section 6, PDF page 13)
#[derive(TryFrom, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[try_from(repr)]
pub enum ProductType {
    RemoteSLorSLMKII = 0x00,
    ZeroSLorZeroMKII = 0x01,
    Compact = 0x02,
}

/// Encoder ring LED position (0-11 on the physical ring)
///
/// Represents a semantic position on the encoder ring LED indicator.
/// Valid positions are 0 (fully counter-clockwise) through 11 (fully clockwise).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EncoderPosition {
    Pos0 = 0,
    Pos1 = 1,
    Pos2 = 2,
    Pos3 = 3,
    Pos4 = 4,
    Pos5 = 5,
    Pos6 = 6,
    Pos7 = 7,
    Pos8 = 8,
    Pos9 = 9,
    Pos10 = 10,
    Pos11 = 11,
}

impl EncoderPosition {
    /// Minimum position (fully counter-clockwise)
    pub const MIN: Self = Self::Pos0;

    /// Maximum position (fully clockwise)
    pub const MAX: Self = Self::Pos11;

    /// Center position
    pub const CENTER: Self = Self::Pos6;
}

bitflags::bitflags! {
    /// Left-hand row-select LED states (RS1-RS5)
    ///
    /// Represents which left-hand row-select LEDs should be illuminated.
    /// Can be combined using bitwise OR operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RowSelectLhSet: u8 {
        const RS1 = 0b00001;
        const RS2 = 0b00010;
        const RS3 = 0b00100;
        const RS4 = 0b01000;
        const RS5 = 0b10000;
    }
}

bitflags::bitflags! {
    /// Right-hand row-select LED states (RS6-RS8, REC)
    ///
    /// Represents which right-hand row-select LEDs should be illuminated.
    /// Can be combined using bitwise OR operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RowSelectRhSet: u8 {
        const RS6 = 0b0001;
        const RS7 = 0b0010;
        const RS8 = 0b0100;
        const REC = 0b1000;  // Record LED
    }
}
