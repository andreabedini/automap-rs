use crate::automap::cc::{
    AUTOMAP_CC_STATUS, Button, Encoder, EncoderPosition, ParameterRequestType, RingMode, RowSelect,
    RowSelectLhSet, RowSelectRhSet,
};

/// Commands that the host can send TO the device (Host → Device).
/// Section 7, 8, 9 of PDF documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomapCommand {
    /// Turn a specific button LED on or off (Section 8, PDF page 16)
    /// Covers CCs 0x18-0x37, 0x48-0x4D for various button groups
    ButtonLed { button: Button, on: bool },

    /// Turn a row select LED on or off (Section 8, PDF page 16)
    /// CCs 0x50-0x54, 0x56-0x57
    RowSelectLed { row: RowSelect, on: bool },

    /// Set encoder ring display mode (Section 9, PDF page 17)
    /// CCs 0x78-0x7F with mode values
    EncoderRingMode { encoder: Encoder, mode: RingMode },

    /// Set encoder ring LED position (Section 9, PDF page 18)
    /// CCs 0x70-0x77 with value 0-11
    EncoderRingValue {
        encoder: Encoder,
        position: EncoderPosition,
    },

    /// Set transport lock mode (Section 8, PDF page 16)
    /// CC 0x4F with value 0=off, 1=on
    TransportLockSet { enabled: bool },

    /// Turn off ALL LEDs (Section 7, PDF page 14)
    /// CC 0x4E value 0x00 - NOT implemented on RemoteSL/ZeroSL
    AllLedsOff,

    /// Set left-hand row select LEDs as bitmap (Section 7, PDF page 15)
    /// CC 0x60 - controls RS1-RS5 LEDs
    RowLhBitmap { rows: RowSelectLhSet },

    /// Set right-hand row select LEDs as bitmap (Section 7, PDF page 15)
    /// CC 0x61 - controls RS6-RS8 and REC LEDs
    RowRhBitmap { rows: RowSelectRhSet },

    /// Request parameter from device (Section 7, PDF page 14)
    /// CC 0x67 - device responds with same CC
    ParameterRequest { request_type: ParameterRequestType },

    /// Echo CC request (Section 7, PDF page 14)
    /// CC 0x63 - originally for Reason template, device echoes back
    EchoRequest { value: u8 },
}

impl AutomapCommand {
    /// Encode this command into MIDI CC bytes for transmission to the device
    pub fn encode_into(self, out: &mut Vec<u8>) {
        match self {
            AutomapCommand::ButtonLed { button, on } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, button as u8, if on { 1 } else { 0 }]);
            }
            AutomapCommand::RowSelectLed { row, on } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, row as u8, if on { 1 } else { 0 }]);
            }
            AutomapCommand::EncoderRingMode { encoder, mode } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, encoder as u8, mode as u8]);
            }
            AutomapCommand::EncoderRingValue { encoder, position } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, (encoder as u8) - 0x08, position as u8]);
            }
            AutomapCommand::TransportLockSet { enabled } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x4F, if enabled { 1 } else { 0 }]);
            }
            AutomapCommand::AllLedsOff => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x4E, 0x00]);
            }
            AutomapCommand::RowLhBitmap { rows } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x60, rows.bits() & 0x7F]);
            }
            AutomapCommand::RowRhBitmap { rows } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x61, rows.bits() & 0x7F]);
            }
            AutomapCommand::ParameterRequest { request_type } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x67, request_type as u8]);
            }
            AutomapCommand::EchoRequest { value } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x63, value]);
            }
        }
    }

    /// Convenience method to encode as a new Vec
    pub fn to_bytes(self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_into(&mut buf);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_led() {
        let cmd = AutomapCommand::ButtonLed {
            button: Button::ButtonA3,
            on: true,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x1A, 0x01]);
    }

    #[test]
    fn test_encoder_ring_mode() {
        let cmd = AutomapCommand::EncoderRingMode {
            encoder: Encoder::Encoder1,
            mode: RingMode::ContinuousCw,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x78, 0x00]);
    }

    #[test]
    fn test_encoder_ring_value() {
        let cmd = AutomapCommand::EncoderRingValue {
            encoder: Encoder::Encoder1,
            position: EncoderPosition::Pos5,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x70, 0x05]);
    }

    #[test]
    fn test_encoder_ring_value_constants() {
        let cmd_min = AutomapCommand::EncoderRingValue {
            encoder: Encoder::Encoder2,
            position: EncoderPosition::MIN,
        };
        assert_eq!(cmd_min.to_bytes(), vec![0xBF, 0x71, 0x00]);

        let cmd_max = AutomapCommand::EncoderRingValue {
            encoder: Encoder::Encoder2,
            position: EncoderPosition::MAX,
        };
        assert_eq!(cmd_max.to_bytes(), vec![0xBF, 0x71, 0x0B]);

        let cmd_center = AutomapCommand::EncoderRingValue {
            encoder: Encoder::Encoder2,
            position: EncoderPosition::CENTER,
        };
        assert_eq!(cmd_center.to_bytes(), vec![0xBF, 0x71, 0x06]);
    }

    #[test]
    fn test_all_leds_off() {
        let cmd = AutomapCommand::AllLedsOff;
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x4E, 0x00]);
    }

    #[test]
    fn test_row_lh_bitmap() {
        let cmd = AutomapCommand::RowLhBitmap {
            rows: RowSelectLhSet::RS2 | RowSelectLhSet::RS5,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x60, 0x12]);
    }

    #[test]
    fn test_row_rh_bitmap() {
        let cmd = AutomapCommand::RowRhBitmap {
            rows: RowSelectRhSet::RS6 | RowSelectRhSet::REC,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x61, 0x09]); // RS6(bit0) + REC(bit3) = 0x09
    }

    #[test]
    fn test_parameter_request() {
        let cmd = AutomapCommand::ParameterRequest {
            request_type: ParameterRequestType::UnitProductType,
        };
        assert_eq!(cmd.to_bytes(), vec![0xBF, 0x67, 0x00]);
    }
}
