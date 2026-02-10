use crate::automap::{
    cc::{
        AUTOMAP_CC_STATUS, AlertType, AutomapButton, Button, Encoder, PageButton, Pot, ProductType,
        RingMode, RowSelect, Slider, TransportButton,
    },
    sysex::DecodeError,
};

use derive_more::{Debug, TryFrom};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomapEvent {
    ModWheel {
        cc: u8,
        value: u8,
    },

    Button {
        button: Button,
        pressed: bool,
    },

    TransportButton {
        button: TransportButton,
        pressed: bool,
    },

    AutomapButton {
        button: AutomapButton,
        pressed: bool,
    },

    Encoder {
        encoder: Encoder,
        clicks: i8,
    },

    Pot {
        pot: Pot,
        value: i8,
    },

    Slider {
        slider: Slider,
        value: i8,
    },

    RowSelect {
        row: RowSelect,
        selected: bool,
    },

    /// LH Row-Select LED bitmap (CC 0x60)
    RowLhBitmap {
        bits: u8,
    },

    /// RH Row-Select LED bitmap (CC 0x61)
    RowRhBitmap {
        bits: u8,
    },

    EncoderTouch {
        encoder: Encoder,
        touched: bool,
    },

    PotTouch {
        pot: Pot,
        touched: bool,
    },

    SliderTouch {
        slider: Slider,
        touched: bool,
    },

    CrossFadeTouch {
        touched: bool,
    },

    SpeedDialTouch {
        touched: bool,
    },

    /// Page button events (0x58-0x5B) - Section 5, PDF page 10
    PageButton {
        button: PageButton,
        pressed: bool,
    },

    /// Sustain pedal (0x40) - Section 5, PDF page 10
    SustainPedal {
        pressed: bool,
    },

    /// Expression pedal (0x41) - Section 5, PDF page 10
    ExpressionPedal {
        value: u8, // 0-127
    },

    CrossFader {
        value: u8, // 0-127
    },

    TouchpadX1 {
        value: u8, // 0-127
    },

    TouchpadY1 {
        value: u8, // 0-127
    },

    TouchpadX2 {
        value: u8, // 0-127
    },

    TouchpadY2 {
        value: u8, // 0-127
    },

    /// Alert from device about configuration changes (0x5C)
    /// Section 6, PDF page 10
    Alert {
        alert_type: AlertType,
    },

    SpeedDial {
        clicks: i8,
    },

    SpeedDialButton {
        pressed: bool,
    },

    PreviewButton {
        pressed: bool,
    },

    TransportLockStatus {
        enabled: bool,
    },

    TempoMsb {
        value: u8,
    },

    TempoLsb {
        value: u8,
    },

    EchoResponse {
        value: u8,
    },

    ParameterResponse {
        response: u8,
    },

    Raw {
        cc: u8,
        value: u8,
    },
}

fn decode_clicks(vv: u8) -> i8 {
    let clockwise = (vv & 0x40) == 0;
    let clicks = vv & 0x3F;
    if clockwise {
        clicks as i8
    } else {
        -(clicks as i8)
    }
}

impl AutomapEvent {
    pub fn decode_event(body: &[u8]) -> Result<AutomapEvent, DecodeError> {
        if body.len() != 3 {
            return Err(DecodeError::Truncated);
        }
        let nn = body[1];
        let vv = body[2];
        match nn {
            0x01 => Ok(AutomapEvent::ModWheel { cc: nn, value: vv }),
            0x08..=0x0F => Ok(AutomapEvent::Pot {
                pot: Pot::try_from(nn).unwrap(), // safe due to match range
                value: vv as i8,                 // MIDI values are 0-127
            }),
            0x10..=0x17 => Ok(AutomapEvent::Slider {
                slider: Slider::try_from(nn).unwrap(), // safe due to match range
                value: vv as i8,                       // MIDI values are 0-127
            }),
            // Button groups A, B, C
            0x18..=0x37 => {
                Ok(AutomapEvent::Button {
                    button: Button::try_from(nn).unwrap(), // safe due to match range
                    pressed: vv != 0,
                })
            }
            0x40 => Ok(AutomapEvent::SustainPedal {
                pressed: vv == 0x7F,
            }),
            0x41 => Ok(AutomapEvent::ExpressionPedal { value: vv }),
            0x42 => Ok(AutomapEvent::CrossFader { value: vv }),
            0x44 => Ok(AutomapEvent::TouchpadX1 { value: vv }),
            0x45 => Ok(AutomapEvent::TouchpadY1 { value: vv }),
            0x46 => Ok(AutomapEvent::TouchpadX2 { value: vv }),
            0x47 => Ok(AutomapEvent::TouchpadY2 { value: vv }),
            // Button group D and Automap buttons
            0x48..=0x4D => {
                match vv {
                    0x00 | 0x01 => Ok(AutomapEvent::TransportButton {
                        button: TransportButton::try_from(nn).unwrap(), // safe due to match range
                        pressed: vv != 0,
                    }),
                    0x40 | 0x41 => Ok(AutomapEvent::AutomapButton {
                        button: AutomapButton::try_from(nn).unwrap(), // safe due to match range
                        pressed: vv != 0x40,
                    }),
                    _ => Ok(AutomapEvent::Raw { cc: nn, value: vv }),
                }
            }
            0x4E => Ok(AutomapEvent::PreviewButton { pressed: vv != 0 }),
            0x4F => Ok(AutomapEvent::TransportLockStatus { enabled: vv != 0 }),
            0x50..=0x57 => Ok(AutomapEvent::RowSelect {
                row: RowSelect::try_from(nn).unwrap(), // safe due to match range
                selected: vv != 0,
            }),
            0x58..=0x5B => Ok(AutomapEvent::PageButton {
                button: PageButton::try_from(nn).unwrap(), // safe due to match range
                pressed: vv != 0,
            }),
            0x5C => match AlertType::try_from(vv) {
                Ok(alert_type) => Ok(AutomapEvent::Alert { alert_type }),
                Err(_) => Ok(AutomapEvent::Raw { cc: nn, value: vv }),
            },
            0x5E => Ok(AutomapEvent::TempoMsb { value: vv }),
            0x5F => Ok(AutomapEvent::TempoLsb { value: vv }),
            0x60 => Ok(AutomapEvent::RowLhBitmap { bits: vv }),
            0x61 => Ok(AutomapEvent::RowRhBitmap { bits: vv }),
            0x63 => Ok(AutomapEvent::EchoResponse { value: vv }),
            0x65 => Ok(AutomapEvent::SpeedDialButton { pressed: vv != 0 }),
            0x66 => Ok(AutomapEvent::SpeedDial {
                clicks: decode_clicks(vv),
            }),
            0x67 => Ok(AutomapEvent::ParameterResponse { response: vv }),
            0x6C => Ok(AutomapEvent::EncoderTouch {
                encoder: Encoder::try_from((vv & 0x0F) + 0x78).unwrap(), // safe due to value range
                touched: (vv & 0x40) != 0,
            }),
            0x6D => Ok(AutomapEvent::PotTouch {
                pot: Pot::try_from((vv & 0x0F) + 0x08).unwrap(), // safe due to value range
                touched: (vv & 0x40) != 0,
            }),
            0x6E => Ok(AutomapEvent::SliderTouch {
                slider: Slider::try_from((vv & 0x0F) + 0x10).unwrap(), // safe due to value range
                touched: (vv & 0x40) != 0,
            }),
            0x6F => {
                if vv & 0x1 == 0 {
                    Ok(AutomapEvent::SpeedDialTouch {
                        touched: (vv & 0x40) != 0,
                    })
                } else {
                    Ok(AutomapEvent::CrossFadeTouch {
                        touched: (vv & 0x40) != 0,
                    })
                }
            }
            _ => Ok(AutomapEvent::Raw { cc: nn, value: vv }),
        }
    }
}
