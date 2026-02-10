pub const NOVATION_ID: [u8; 5] = [0xF0, 0x00, 0x20, 0x29, 0x03];
pub const EOX: u8 = 0xF7;
pub const PROTO_VER_MAIN: u8 = 0x12; // BCD 1.2 per docs
pub const PROTO_VER_BETA: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtoFamily {
    Automap0303,
    DbSim0305,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    NotSysEx,
    BadHeader,
    BadFamily,
    Truncated,
    Invalid,
    Unsupported,
}

// 14-bit helpers used by Data-Block formats
#[inline]
pub fn pack_u14(v: u16) -> (u8, u8) {
    ((v & 0x7F) as u8, ((v >> 7) & 0x7F) as u8)
}
#[inline]
pub fn unpack_u14(lsb: u8, msb: u8) -> u16 {
    (lsb as u16) | ((msb as u16) << 7)
}

// ============================== AUTOMAP (03:03) ==============================

/// A complete Automap command (the byte after “… 03 03 VV bb 02 00”).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomapSysEx<'a> {
    /// 0x01 – Notify ONLINE/OFFLINE state.
    OnlineOffline { online: bool },

    /// 0x02 – LCD text command is a stream of sub-ops executed in order.
    LcdText(Vec<LcdOp<'a>>),

    /// 0x03 – Request Globals download to RAM (no payload).
    GlobalsDownloadRam,

    /// 0x04 – Prepare OS download (no payload).
    PrepareOsDownload,

    /// 0x05 – Upload Globals (opaque blob).
    UploadGlobals { data: &'a [u8] },

    /// 0x06 – Request Globals download to RAM & Flash (no payload).
    GlobalsDownloadRamAndFlash,

    /// 0x07 – Upload Template (opaque blob; if you need templ# add it in data).
    UploadTemplate { data: &'a [u8] },

    /// 0x08 – Upload OS (opaque blob).
    UploadOs { data: &'a [u8] },

    /// Unknown command preserved for forward-compat.
    Unknown { cmd: u8, data: &'a [u8] },
}

/// LCD sub-ops (the stream inside AutomapMsg::LcdText).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LcdOp<'a> {
    End,                               // 0x00
    Cursor { col: u8, line: LcdLine }, // 0x01
    Clear(LcdClear),                   // 0x02
    CursorBlink(bool),                 // 0x03 (not implemented by unit)
    Text(&'a [u8]),                    // 0x04 null-terminated at encode
    Unknown(u8, &'a [u8]),             // passthrough
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdLine {
    LeftTop = 1,
    RightTop = 2,
    LeftBottom = 3,
    RightBottom = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdClear {
    BothDisplays = 0x01,
    BothTopLines = 0x02,
    BothBottomLines = 0x03,
    LeftAll = 0x04,
    RightAll = 0x05,
    LeftTopLine = 0x06,
    LeftBottomLine = 0x07,
    RightTopLine = 0x08,
    RightBottomLine = 0x09,
    FromCursorCount(u8) = 0x0A, // will be emitted as 0x0A, <count>
}

impl LcdClear {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

// ============================== DB / SIM (03:05) ==============================

/// Data-block targets (globals/template/control).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbTarget {
    Control,
    TemplateHeader,
    Globals,
}

/// A complete Data-Block or Simulation message (after “… 03 05 VV bb 00 00”).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbSimMsg<'a> {
    // -------- Data-Block: Change / Request (main 0x68) --------
    /// Write bytes to a target at offset (14-bit). For Control, include `cn` (1-based).
    DbWrite {
        target: DbTarget,
        cn: Option<u8>,
        offset: u16,
        data: &'a [u8],
    },

    /// Read `len` bytes from a target at offset (14-bit). For Control, include `cn`.
    DbRead {
        target: DbTarget,
        cn: Option<u8>,
        offset: u16,
        len: u16,
    },

    // -------- Data-Block: Response (main 0x69) --------
    /// Response with the bytes read (echo target/offset).
    DbData {
        target: DbTarget,
        cn: Option<u8>,
        offset: u16,
        data: &'a [u8],
    },

    // -------- Simulation (main 0x66 / 0x6A) --------
    /// Simulate a low-level user action (main 0x66).
    Simulate(SimCmd),

    /// High-level device action (main 0x6A).
    HighLevel(SimHighLevel),
}

/// Simulation sub-commands (main 0x66).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimCmd {
    Button {
        number_1_based: u8,
        pressed: bool,
    }, // 0x01
    PotSlider {
        number_1_based: u8,
        value: u8,
    }, // 0x02 (0..127)
    Encoder {
        number_1_based: u8,
        clicks_signed: i8,
    }, // 0x03 (±64)
    LcdTextRequest,    // 0x04
    LcdTextResponse,   // 0x05
    LedBitmapRequest,  // 0x06
    LedBitmapResponse, // 0x07
    Key {
        number_1_based: u8,
        velocity: u8,
    }, // 0x08
    TouchpadXY {
        x: u8,
        y: u8,
    }, // 0x09
    Drumpad {
        number_1_based: u8,
        value: u8,
    }, // 0x0A
    SustainPedal {
        pressed: bool,
    }, // 0x0B
    TouchSensor {
        sensor_1_to_26_or_127: u8,
    }, // 0x0C (127 = none)
    Unknown(u8, Vec<u8>),
}

/// High-level device ops (main 0x6A).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimHighLevel {
    SaveGlobalsToFlash,         // 0x00
    SaveCurrentTemplateToFlash, // 0x01
    UpdateOctaveLeds,           // 0x02
    ForcePlayMode,              // 0x03
    SendCurrentTemplateToHost,  // 0x04
    Unknown(u8),
}

// ============================== Encoding ==============================

impl LcdOp<'_> {
    fn encode_into(&self, out: &mut Vec<u8>, _ver_main: u8, _ver_beta: u8) {
        match self {
            LcdOp::End => out.push(0x00),
            LcdOp::Cursor { col, line } => {
                out.extend_from_slice(&[0x01, *col, *line as u8]);
            }
            LcdOp::Clear(code) => {
                out.push(0x02);
                match code {
                    LcdClear::FromCursorCount(n) => out.extend_from_slice(&[0x0A, *n]),
                    other => out.push(other.discriminant()),
                }
            }
            LcdOp::CursorBlink(on) => out.extend_from_slice(&[0x03, if *on { 1 } else { 0 }]),
            LcdOp::Text(bytes) => {
                out.push(0x04);
                out.extend_from_slice(bytes);
                out.push(0x00);
            }
            LcdOp::Unknown(t, data) => {
                out.push(*t);
                out.extend_from_slice(data);
            }
        }
    }
}

impl<'a> AutomapSysEx<'a> {
    fn encode_into(&self, out: &mut Vec<u8>) {
        // Header: F0 00 20 29 03 03 VV bb 02 00
        out.extend_from_slice(&NOVATION_ID);
        out.push(0x03);

        out.push(PROTO_VER_MAIN);
        out.push(PROTO_VER_BETA);
        out.extend_from_slice(&[0x02, 0x00]);

        match self {
            AutomapSysEx::OnlineOffline {
                online: to_host_online,
            } => {
                out.push(0x01);
                out.push(if *to_host_online { 0x01 } else { 0x00 });
            }
            AutomapSysEx::LcdText(ops) => {
                out.push(0x02);
                for op in ops {
                    op.encode_into(out, PROTO_VER_MAIN, PROTO_VER_BETA);
                }
            }
            AutomapSysEx::GlobalsDownloadRam => out.push(0x03),
            AutomapSysEx::PrepareOsDownload => out.push(0x04),
            AutomapSysEx::UploadGlobals { data } => {
                out.push(0x05);
                out.extend_from_slice(data);
            }
            AutomapSysEx::GlobalsDownloadRamAndFlash => out.push(0x06),
            AutomapSysEx::UploadTemplate { data } => {
                out.push(0x07);
                out.extend_from_slice(data);
            }
            AutomapSysEx::UploadOs { data } => {
                out.push(0x08);
                out.extend_from_slice(data);
            }
            AutomapSysEx::Unknown { cmd, data } => {
                out.push(*cmd);
                out.extend_from_slice(data);
            }
        }

        out.push(EOX);
    }

    /// Convenience method to encode as a new Vec
    pub fn to_bytes(self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_into(&mut buf);
        buf
    }
}

impl<'a> DbSimMsg<'a> {
    fn encode_into(&self, out: &mut Vec<u8>, ver_main: u8, ver_beta: u8) {
        // Header: F0 00 20 29 03 05 VV bb 00 00
        out.extend_from_slice(&NOVATION_ID);
        out.push(0x05);
        out.push(ver_main);
        out.push(ver_beta);
        out.extend_from_slice(&[0x00, 0x00]);

        match self {
            // main 0x68 (Change/Request)
            DbSimMsg::DbWrite {
                target,
                cn,
                offset,
                data,
            } => {
                out.push(0x68);
                out.push(match target {
                    DbTarget::Control => 0x00,
                    DbTarget::TemplateHeader => 0x01,
                    DbTarget::Globals => 0x02,
                });
                if let Some(c) = cn {
                    out.push(*c);
                }
                let (lsb, msb) = pack_u14(*offset);
                out.extend_from_slice(&[lsb, msb]);
                out.extend_from_slice(data);
            }
            DbSimMsg::DbRead {
                target,
                cn,
                offset,
                len,
            } => {
                out.push(0x68);
                out.push(match target {
                    DbTarget::Control => 0x03,
                    DbTarget::TemplateHeader => 0x04,
                    DbTarget::Globals => 0x05,
                });
                if let Some(c) = cn {
                    out.push(*c);
                }
                let (olsb, omsb) = pack_u14(*offset);
                let (llsb, lmsb) = pack_u14(*len);
                out.extend_from_slice(&[olsb, omsb, llsb, lmsb]);
            }
            // main 0x69 (Response)
            DbSimMsg::DbData {
                target,
                cn,
                offset,
                data,
            } => {
                out.push(0x69);
                out.push(match target {
                    DbTarget::Control => 0x03,
                    DbTarget::TemplateHeader => 0x04,
                    DbTarget::Globals => 0x05,
                });
                if let Some(c) = cn {
                    out.push(*c);
                }
                let (lsb, msb) = pack_u14(*offset);
                out.extend_from_slice(&[lsb, msb]);
                out.extend_from_slice(data);
            }
            // Simulation main 0x66 / 0x6A
            DbSimMsg::Simulate(cmd) => {
                out.push(0x66);
                match cmd {
                    SimCmd::Button {
                        number_1_based,
                        pressed,
                    } => out.extend_from_slice(&[
                        0x01,
                        *number_1_based,
                        if *pressed { 1 } else { 0 },
                    ]),
                    SimCmd::PotSlider {
                        number_1_based,
                        value,
                    } => out.extend_from_slice(&[0x02, *number_1_based, *value]),
                    SimCmd::Encoder {
                        number_1_based,
                        clicks_signed,
                    } => {
                        // unit expects unsigned 0..127, but doc says ±64; remap here:
                        let v = (*clicks_signed as i16).clamp(-64, 63);
                        out.extend_from_slice(&[0x03, *number_1_based, (v as i8) as u8]);
                    }
                    SimCmd::LcdTextRequest => out.extend_from_slice(&[0x04]),
                    SimCmd::LcdTextResponse => out.extend_from_slice(&[0x05]),
                    SimCmd::LedBitmapRequest => out.extend_from_slice(&[0x06]),
                    SimCmd::LedBitmapResponse => out.extend_from_slice(&[0x07]),
                    SimCmd::Key {
                        number_1_based,
                        velocity,
                    } => out.extend_from_slice(&[0x08, *number_1_based, *velocity]),
                    SimCmd::TouchpadXY { x, y } => out.extend_from_slice(&[0x09, *x, *y]),
                    SimCmd::Drumpad {
                        number_1_based,
                        value,
                    } => out.extend_from_slice(&[0x0A, *number_1_based, *value]),
                    SimCmd::SustainPedal { pressed } => {
                        out.extend_from_slice(&[0x0B, 1, if *pressed { 1 } else { 0 }])
                    }
                    SimCmd::TouchSensor {
                        sensor_1_to_26_or_127,
                    } => out.extend_from_slice(&[0x0C, *sensor_1_to_26_or_127]),
                    SimCmd::Unknown(sc, bytes) => {
                        out.push(*sc);
                        out.extend_from_slice(bytes);
                    }
                }
            }
            DbSimMsg::HighLevel(h) => {
                out.push(0x6A);
                out.push(match h {
                    SimHighLevel::SaveGlobalsToFlash => 0x00,
                    SimHighLevel::SaveCurrentTemplateToFlash => 0x01,
                    SimHighLevel::UpdateOctaveLeds => 0x02,
                    SimHighLevel::ForcePlayMode => 0x03,
                    SimHighLevel::SendCurrentTemplateToHost => 0x04,
                    SimHighLevel::Unknown(n) => *n,
                });
            }
        }

        out.push(EOX);
    }
}

// ============================== Decoding (framing + dispatch) ==============================

/// Inspect header, choose family, return (family, ver_main, ver_beta, body_without_eox).
fn split_header(frame: &[u8]) -> Result<(ProtoFamily, u8, u8, &[u8]), DecodeError> {
    if !frame.starts_with(&[0xF0]) || !frame.ends_with(&[EOX]) {
        return Err(DecodeError::NotSysEx);
    }
    if frame.len() < 12 {
        return Err(DecodeError::Truncated);
    }
    if frame[0..5] != NOVATION_ID {
        return Err(DecodeError::BadHeader);
    }
    let fam = frame[5];
    let ver_main = frame[6];
    let ver_beta = frame[7];
    let tag_a = frame[8];
    let tag_b = frame[9];
    let (family, start) = match (fam, tag_a, tag_b) {
        (0x03, 0x02, 0x00) => (ProtoFamily::Automap0303, 10),
        (0x05, 0x00, 0x00) => (ProtoFamily::DbSim0305, 10),
        _ => return Err(DecodeError::BadFamily),
    };
    Ok((family, ver_main, ver_beta, &frame[start..frame.len() - 1]))
}

/// Decode a full frame into a semantic command.
pub fn decode_frame<'a>(
    frame: &'a [u8],
) -> Result<(ProtoFamily, u8, u8, DecodedMsg<'a>), DecodeError> {
    let (family, vm, vb, body) = split_header(frame)?;
    let decoded = match family {
        ProtoFamily::Automap0303 => DecodedMsg::Automap(decode_automap(body)?),
        ProtoFamily::DbSim0305 => DecodedMsg::DbSim(decode_dbsim(body)?),
    };
    Ok((family, vm, vb, decoded))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedMsg<'a> {
    Automap(AutomapSysEx<'a>),
    DbSim(DbSimMsg<'a>),
}

// ---- Automap decode of the single “main cmd + payload” that follows  ----
fn decode_automap<'a>(body: &'a [u8]) -> Result<AutomapSysEx<'a>, DecodeError> {
    if body.is_empty() {
        return Err(DecodeError::Truncated);
    }
    let cmd = body[0];
    let rest = &body[1..];
    Ok(match cmd {
        0x01 => AutomapSysEx::OnlineOffline {
            online: rest.first().copied().unwrap_or(0) != 0,
        },
        0x02 => AutomapSysEx::LcdText(decode_lcd_ops(rest)?),
        0x03 => AutomapSysEx::GlobalsDownloadRam,
        0x04 => AutomapSysEx::PrepareOsDownload,
        0x05 => AutomapSysEx::UploadGlobals { data: rest },
        0x06 => AutomapSysEx::GlobalsDownloadRamAndFlash,
        0x07 => AutomapSysEx::UploadTemplate { data: rest },
        0x08 => AutomapSysEx::UploadOs { data: rest },
        c => AutomapSysEx::Unknown { cmd: c, data: rest },
    })
}

fn decode_lcd_ops<'a>(mut s: &'a [u8]) -> Result<Vec<LcdOp<'a>>, DecodeError> {
    let mut out = Vec::new();
    while !s.is_empty() {
        let t = s[0];
        s = &s[1..];
        let op = match t {
            0x00 => LcdOp::End,
            0x01 => {
                if s.len() < 2 {
                    return Err(DecodeError::Truncated);
                }
                let col = s[0];
                let line = match s[1] {
                    1 => LcdLine::LeftTop,
                    2 => LcdLine::RightTop,
                    3 => LcdLine::LeftBottom,
                    4 => LcdLine::RightBottom,
                    _ => return Err(DecodeError::Invalid),
                };
                s = &s[2..];
                LcdOp::Cursor { col, line }
            }
            0x02 => {
                if s.is_empty() {
                    return Err(DecodeError::Truncated);
                }
                let code = s[0];
                s = &s[1..];
                if code == 0x0A {
                    if s.is_empty() {
                        return Err(DecodeError::Truncated);
                    }
                    let n = s[0];
                    s = &s[1..];
                    LcdOp::Clear(LcdClear::FromCursorCount(n))
                } else {
                    LcdOp::Clear(match code {
                        0x01 => LcdClear::BothDisplays,
                        0x02 => LcdClear::BothTopLines,
                        0x03 => LcdClear::BothBottomLines,
                        0x04 => LcdClear::LeftAll,
                        0x05 => LcdClear::RightAll,
                        0x06 => LcdClear::LeftTopLine,
                        0x07 => LcdClear::LeftBottomLine,
                        0x08 => LcdClear::RightTopLine,
                        0x09 => LcdClear::RightBottomLine,
                        _ => return Err(DecodeError::Invalid),
                    })
                }
            }
            0x03 => {
                if s.is_empty() {
                    return Err(DecodeError::Truncated);
                }
                let on = s[0] != 0;
                s = &s[1..];
                LcdOp::CursorBlink(on)
            }
            0x04 => {
                // Text until 0x00 (not including)
                let nul = s
                    .iter()
                    .position(|&b| b == 0x00)
                    .ok_or(DecodeError::Truncated)?;
                let (txt, rest) = s.split_at(nul);
                s = &rest[1..];
                LcdOp::Text(txt)
            }
            x => {
                // unknown → slurp to next known boundary (here we can't know length; pass empty)
                LcdOp::Unknown(x, &[])
            }
        };
        out.push(op);
    }
    Ok(out)
}

// ---- DB/SIM decode (main byte then typed payload) ----
fn decode_dbsim<'a>(body: &'a [u8]) -> Result<DbSimMsg<'a>, DecodeError> {
    if body.is_empty() {
        return Err(DecodeError::Truncated);
    }
    let main = body[0];
    let mut s = &body[1..];
    Ok(match main {
        0x68 => {
            // Change/Request
            if s.is_empty() {
                return Err(DecodeError::Truncated);
            }
            let sub = s[0];
            s = &s[1..];
            match sub {
                0x00..=0x02 => {
                    // write
                    let (target, need_cn) = match sub {
                        0x00 => (DbTarget::Control, true),
                        0x01 => (DbTarget::TemplateHeader, false),
                        _ => (DbTarget::Globals, false),
                    };
                    let cn = if need_cn {
                        if s.is_empty() {
                            return Err(DecodeError::Truncated);
                        }
                        let c = s[0];
                        s = &s[1..];
                        Some(c)
                    } else {
                        None
                    };
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    let off = unpack_u14(s[0], s[1]);
                    s = &s[2..];
                    DbSimMsg::DbWrite {
                        target,
                        cn,
                        offset: off,
                        data: s,
                    }
                }
                0x03..=0x05 => {
                    // read
                    let (target, need_cn) = match sub {
                        0x03 => (DbTarget::Control, true),
                        0x04 => (DbTarget::TemplateHeader, false),
                        _ => (DbTarget::Globals, false),
                    };
                    let cn = if need_cn {
                        if s.is_empty() {
                            return Err(DecodeError::Truncated);
                        }
                        let c = s[0];
                        s = &s[1..];
                        Some(c)
                    } else {
                        None
                    };
                    if s.len() < 4 {
                        return Err(DecodeError::Truncated);
                    }
                    let off = unpack_u14(s[0], s[1]);
                    let len = unpack_u14(s[2], s[3]);
                    DbSimMsg::DbRead {
                        target,
                        cn,
                        offset: off,
                        len,
                    }
                }
                _ => return Err(DecodeError::Invalid),
            }
        }
        0x69 => {
            // Response
            if s.is_empty() {
                return Err(DecodeError::Truncated);
            }
            let sub = s[0];
            s = &s[1..];
            let (target, need_cn) = match sub {
                0x03 => (DbTarget::Control, true),
                0x04 => (DbTarget::TemplateHeader, false),
                0x05 => (DbTarget::Globals, false),
                _ => return Err(DecodeError::Invalid),
            };
            let cn = if need_cn {
                if s.is_empty() {
                    return Err(DecodeError::Truncated);
                }
                let c = s[0];
                s = &s[1..];
                Some(c)
            } else {
                None
            };
            if s.len() < 2 {
                return Err(DecodeError::Truncated);
            }
            let off = unpack_u14(s[0], s[1]);
            s = &s[2..];
            DbSimMsg::DbData {
                target,
                cn,
                offset: off,
                data: s,
            }
        }
        0x66 => {
            // Simulate
            if s.is_empty() {
                return Err(DecodeError::Truncated);
            }
            let sc = s[0];
            s = &s[1..];
            use SimCmd::*;
            let cmd = match sc {
                0x01 => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    Button {
                        number_1_based: s[0],
                        pressed: s[1] != 0,
                    }
                }
                0x02 => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    PotSlider {
                        number_1_based: s[0],
                        value: s[1],
                    }
                }
                0x03 => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    Encoder {
                        number_1_based: s[0],
                        clicks_signed: s[1] as i8,
                    }
                }
                0x04 => LcdTextRequest,
                0x05 => LcdTextResponse,
                0x06 => LedBitmapRequest,
                0x07 => LedBitmapResponse,
                0x08 => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    Key {
                        number_1_based: s[0],
                        velocity: s[1],
                    }
                }
                0x09 => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    TouchpadXY { x: s[0], y: s[1] }
                }
                0x0A => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    Drumpad {
                        number_1_based: s[0],
                        value: s[1],
                    }
                }
                0x0B => {
                    if s.len() < 2 {
                        return Err(DecodeError::Truncated);
                    }
                    SustainPedal { pressed: s[1] != 0 }
                }
                0x0C => {
                    if s.is_empty() {
                        return Err(DecodeError::Truncated);
                    }
                    TouchSensor {
                        sensor_1_to_26_or_127: s[0],
                    }
                }
                _ => SimCmd::Unknown(sc, s.to_vec()),
            };
            DbSimMsg::Simulate(cmd)
        }
        0x6A => {
            // High-level
            if s.is_empty() {
                return Err(DecodeError::Truncated);
            }
            let sub = s[0];
            let hl = match sub {
                0x00 => SimHighLevel::SaveGlobalsToFlash,
                0x01 => SimHighLevel::SaveCurrentTemplateToFlash,
                0x02 => SimHighLevel::UpdateOctaveLeds,
                0x03 => SimHighLevel::ForcePlayMode,
                0x04 => SimHighLevel::SendCurrentTemplateToHost,
                n => SimHighLevel::Unknown(n),
            };
            DbSimMsg::HighLevel(hl)
        }
        _ => return Err(DecodeError::Invalid),
    })
}

// ============================== Example usage ==============================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_lcd() {
        let msg = AutomapSysEx::LcdText(vec![
            LcdOp::Clear(LcdClear::LeftAll),
            LcdOp::Cursor {
                col: 9,
                line: LcdLine::LeftTop,
            },
            LcdOp::Text(b"Hello"),
            LcdOp::End,
        ]);
        let mut buf = Vec::new();
        msg.encode_into(&mut buf);
        let (_, _, _, DecodedMsg::Automap(r)) = decode_frame(&buf).unwrap() else {
            panic!()
        };
        assert_eq!(r, msg);
    }
}
