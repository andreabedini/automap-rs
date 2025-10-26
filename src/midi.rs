//! USB-MIDI packet format conversion utilities.
//!
//! This module implements conversion between raw MIDI bytes and USB-MIDI packets
//! as defined in the "Universal Serial Bus Device Class Definition for MIDI Devices"
//! specification (v1.0).
//!
//! USB-MIDI packets are 4 bytes: `[CIN, midi_0, midi_1, midi_2]`
//! - CIN (Cable Index Number): High nibble = cable number (0), low nibble = Code Index Number
//! - midi_0..2: Up to 3 MIDI data bytes

/// Converts raw MIDI bytes into 4-byte USB-MIDI event packets.
///
/// Each USB-MIDI packet contains a Cable Index Number (CIN) byte followed by
/// up to 3 MIDI data bytes. The CIN encodes the MIDI message type and length.
///
/// # Arguments
///
/// * `midi` - Raw MIDI message bytes (may contain multiple messages)
///
/// # Returns
///
/// A vector of 4-byte USB-MIDI packets suitable for USB transmission.
///
/// # Example
///
/// ```ignore
/// let midi = vec![0xB0, 0x07, 0x7F]; // MIDI CC message
/// let packets = usbmidi_pack(&midi); // Returns [0x0B, 0xB0, 0x07, 0x7F]
/// ```
pub(crate) fn usbmidi_pack(midi: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((midi.len() / 3 + 1) * 4);
    let mut i = 0;

    while i < midi.len() {
        let status = midi[i];

        // System Real-Time messages (single byte)
        if (0xF8..=0xFF).contains(&status) && status != 0xF9 && status != 0xFD {
            out.extend_from_slice(&[0x0F, status, 0, 0]);
            i += 1;
            continue;
        }

        // SysEx
        if status == 0xF0 {
            let mut end = i + 1;
            while end < midi.len() && midi[end] != 0xF7 {
                end += 1;
            }
            if end < midi.len() {
                end += 1; // include F7
            }

            // Pack SysEx in chunks (simplified - just handle complete sysex in one packet for now)
            let sysex_data = &midi[i..end];
            let len = sysex_data.len();

            if len >= 1 && len <= 3 {
                let cin = match len {
                    1 => 0x05, // Single-byte system common
                    2 => 0x06, // Two-byte system common
                    3 => 0x07, // Three-byte system common
                    _ => 0x04, // SysEx start/continue
                };
                let mut packet = [cin, 0, 0, 0];
                packet[1..=len].copy_from_slice(sysex_data);
                out.extend_from_slice(&packet);
            } else {
                // For longer SysEx, need proper chunking - simplified for now
                for chunk in sysex_data.chunks(3) {
                    let cin = if chunk.contains(&0xF7) {
                        match chunk.len() {
                            1 => 0x05,
                            2 => 0x06,
                            3 => 0x07,
                            _ => 0x07,
                        }
                    } else if chunk[0] == 0xF0 {
                        0x04 // SysEx start
                    } else {
                        0x04 // SysEx continue
                    };

                    let mut packet = [cin, 0, 0, 0];
                    packet[1..=chunk.len()].copy_from_slice(chunk);
                    out.extend_from_slice(&packet);
                }
            }

            i = end;
            continue;
        }

        // Regular messages
        let need = match status {
            0xC0..=0xDF | 0xF1 | 0xF3 => 2, // Program Change, Channel Pressure, Song Select, etc.
            0xF2 => 3,                      // Song Position Pointer
            0x80..=0xBF | 0xE0..=0xEF => 3, // Note Off, Note On, CC, Pitch Bend, etc.
            0xF6 => 1,                      // Tune Request
            _ => 1,
        };

        if i + need > midi.len() {
            break; // Incomplete message
        }

        let cin = (status >> 4) & 0x0F;
        let mut packet = [cin, 0, 0, 0];
        packet[1..=need].copy_from_slice(&midi[i..i + need]);
        out.extend_from_slice(&packet);

        i += need;
    }

    out
}

/// Converts 4-byte USB-MIDI event packets into raw MIDI bytes.
///
/// This is the inverse of `usbmidi_pack()`. It extracts MIDI data bytes from
/// USB-MIDI packets by examining the CIN (Code Index Number) to determine
/// how many bytes to extract from each 4-byte packet.
///
/// # Arguments
///
/// * `buf` - Buffer containing USB-MIDI packets (must be multiple of 4 bytes)
///
/// # Returns
///
/// A vector of raw MIDI bytes extracted from the packets.
pub(crate) fn usbmidi_unpack(buf: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(buf.len());
    for ev in buf.chunks_exact(4) {
        let cin = ev[0] & 0x0F;
        match cin {
            0x8 | 0x9 | 0xA | 0xB | 0xE | 0x3 => out.extend_from_slice(&ev[1..=3]),
            0xC | 0xD | 0x2 => out.extend_from_slice(&ev[1..=2]),
            0x5 | 0xF => out.push(ev[1]),
            0x4 => out.extend_from_slice(&ev[1..=3]),
            0x6 => out.extend_from_slice(&ev[1..=2]),
            0x7 => out.extend_from_slice(&ev[1..=3]),
            _ => {}
        }
    }
    out
}

/// Splits a stream of raw MIDI bytes into complete MIDI messages.
///
/// This function parses a byte stream and extracts complete MIDI messages
/// by analyzing status bytes and message lengths. It handles:
/// - System Real-Time messages (single byte)
/// - SysEx messages (variable length, F0...F7)
/// - Channel messages (2-3 bytes)
/// - System Common messages
///
/// Running status is not supported - each message must have its own status byte.
///
/// # Arguments
///
/// * `bs` - Raw MIDI byte stream (may contain multiple messages)
///
/// # Returns
///
/// A vector where each element is a complete MIDI message.
pub(crate) fn split_midi_messages(mut bs: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    while !bs.is_empty() {
        let b0 = bs[0];
        if (0xF8..=0xFF).contains(&b0) && b0 != 0xF9 && b0 != 0xFD {
            out.push(vec![b0]);
            bs = &bs[1..];
            continue;
        }
        if b0 < 0x80 {
            bs = &bs[1..];
            continue;
        }
        if b0 == 0xF0 {
            let mut i = 1;
            while i < bs.len() && bs[i] != 0xF7 {
                i += 1;
            }
            if i < bs.len() {
                i += 1;
            }
            out.push(bs[..i].to_vec());
            bs = &bs[i..];
            continue;
        }
        let need = match b0 {
            0xC0..=0xDF | 0xF1 | 0xF3 => 2,
            0xF2 => 3,
            0x80..=0xBF | 0xE0..=0xEF => 3,
            0xF6 => 1,
            _ => 1,
        };
        if bs.len() >= need {
            out.push(bs[..need].to_vec());
            bs = &bs[need..];
        } else {
            break;
        }
    }
    out
}
