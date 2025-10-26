# Template Structure Documentation

Based on "SLMKII Template Offsets.pdf" and "SL Control Members etc.pdf"

## Overview

Templates define how physical controls on the SLMKII/ZeroMKII map to MIDI messages and display information. A template contains:
- **Header**: Template metadata (name, version, type, group info)
- **Zones**: Up to 4 MIDI zones with routing configuration
- **Controls**: Configuration for 90 physical controls (encoders, pots, sliders, buttons, drumpads, transport)

## Template Header (Offsets 0x00-0x196)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 8 | Name | "NOCTURN" (null-padded ASCII) |
| 0x30 | 1 | Template Number | 0x00-0x1F (0-31), 0xFF=Automap |
| 0x31 | 2 | Version | RemoteSL version (BCD) |
| 0x33 | 1 | Template Type | 0x00=Normal, 0x01=Reason3, 0x02=Logic Controller |
| 0x34 | 1 | Group Size | Number of templates in group |
| 0x35 | 1 | Group Position | Position within group |
| 0x36 | 2 | Control Count | Number of controls (FTTB FIX format) |
| 0x38 | 2 | Control Size | Size of each control (29H = 41 bytes) |
| 0x39 | 1 | SysEx Length | Length of SysEx message |
| 0x3A | 10 | SysEx Message | SysEx message template |
| 0x44-0x52 | - | Row Select Config | Last row-select starts for each row |
| 0x54 | 1 | Last Left RS | Last left row-select |
| 0x55 | 1 | Last Right RS | Last right row-select |
| 0x56-0x60 | - | MIDI Config | Keyboard/Common MIDI channels, ports, bank, program, velocity, octave |
| 0x61 | 1 | Zone Attributes | B0=PB, B1=MW, B2=AT |
| 0x62-0x87 | - | 4 Zones | Zone 1-4 configuration (10 bytes each) |
| 0x8A | 4 | Touchpad Attr | X1,Y1,X2,Y2 attributes |
| 0x8E | 8×10 | Drumpad Auto | Auto-off settings for 8 drumpads |
| 0x9E | 8 | Drumpad Sync | Off-sync settings |
| 0xA6-0xAD | 8 | HUI/Plugin Config | HUI type, plugin modes, display attributes |
| 0xAE | 1 | Drumpad Trans | Drumpad transpose |
| 0xAF | 1 | Trans Spare | Spare field |
| 0xB0 | 8 | Encoder String | Encoder ring configuration |
| 0x196 | 1 | END | End marker |

## Zone Configuration (10 bytes per zone)

Each zone (offsets 0x62-0x87):

| Offset | Field | Description |
|--------|-------|-------------|
| +0 | MIDI Channel | 0x00-0x0F (channels 1-16) |
| +1 | Routing | Port routing configuration |
| +2 | Velocity | MIDI velocity |
| +3 | Low Note | Lowest note in range |
| +4 | High Note | Highest note in range |
| +5 | Transpose | Transpose amount |
| +6 | Attributes | PB+MW+AT attribute bits |
| +7-9 | Spares | Reserved fields |

## Control Structure (41 bytes per control)

Each control has 41 bytes at a specific offset in the control array. Fields at specific offsets:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 8 | CNNAME | Control name (ASCII, space-padded) |
| 0x08 | 1 | CNTYPE | Control type (see Control Types below) |
| 0x09 | 1 | CNLOWU | Low value upper byte |
| 0x0A | 1 | CNLOW | Low value lower byte (14-bit value) |
| 0x0B | 1 | CNHIGHU | High value upper byte |
| 0x0C | 1 | CNHIGH | High value lower byte (14-bit value) |
| 0x0D | 1 | CNATTR1 | Control attributes byte 1 |
| 0x0E | 1 | CNATTR2 | Control attributes byte 2 |
| 0x0F | 1 | CNATTR3 | Control attributes byte 3 |
| 0x10 | 1 | CNCNMSB | MIDI CC MSB |
| 0x11 | 1 | CNCNLSB | MIDI CC LSB |
| 0x12 | 1 | CNPORTS | Port routing |
| 0x13 | 1 | CNMCHAN | MIDI channel |
| 0x14 | 1 | CNSETMSB | Set value MSB |
| 0x15 | 1 | CNSETLSB | Set value LSB |
| 0x16 | 1 | CNMSTEPV | MIDI step value |
| 0x18 | 3 | CNSPARE | Spare bytes |
| 0x19 | 1 | CNSXATTR1 | SysEx attribute 1 |
| 0x1A | 1 | CNSXSIZE | SysEx buffer size |
| 0x1B | 1 | CNUPDPSN | Update position |
| 0x1C | 12 | CNSXBUF | SysEx buffer |
| 0x28 | 1 | NORMCNSIZE | Normal control size (0x29 = 41 bytes) |

## Control Types (CNTYPE field)

| Value | Type | Description |
|-------|------|-------------|
| 0 | CTSPARE | Spare/unused control |
| 1 | CTCC | Continuous Controller (CC) |
| 2 | CTNRPN | NRPN (14-bit) |
| 3 | CTRPN | RPN (14-bit) |
| 4 | CTSYSEX | SysEx message |
| 5 | CTMMC | MMC (MIDI Machine Control) |
| 6 | CTNOTEON | Note On |
| 7 | CTNOTEOFF | Note Off |
| 8 | CTBANKSEL | Bank Select |
| 9 | CTPROGCG | Program Change |
| 10 | CTPBEND | Pitch Bend |
| 11 | CTDRUMNOTE | Drum Note |
| 12 | CTTEMPCG | Template Change |
| 13 | CTREALTIME | Real-time message |
| 14 | CTTEMPGRP | Template Group |

## Control Attributes (CNATTR1 byte)

Bit positions in CNATTR1:

| Bit | Name | Description |
|-----|------|-------------|
| 0 | CASLSV1ST | Send MS value first (for 2-byte values) |
| 1 | CASEND2BV | Send a 2-byte value |
| 2 | CARELTOO | Release as well as press value sent |
| 3 | CATOGGLE | Toggle the value sent (HIGH/LOW) |
| 4 | CACYCBTN | Cyclic/step button action |
| 6 | CARAWDATA | Use raw data in control SysEx buffer |

## Control Attributes (CNATTR2 byte)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | CASEND1CN | Only send one control number (not used) |
| 1 | CALSCN1ST | Send LS control number first (not used) |
| 2 | CANOSNAP | Snap-shot: don't send during snap-shot |
| 3 | CAINVERT | Invert the control value |
| 4 | CASEENCATCHUP | 0=Seen, 1=Not seen (encoder catchup) |

## Control Attributes (CNATTR3 byte)

Bits 0-4: Display format (CMDISPTYP)

| Value | Display Type | Description |
|-------|--------------|-------------|
| 0 | FT127 | 0-127 |
| 1 | FT6463 | -64 to +63 |
| 2 | FTMMC | MMC display |
| 3 | FTOFFON | OFF/ON |
| 4 | FTBLANK | Blank display |
| 5 | FTLABEL | Display control name |
| 6 | FTREL1 | Relative encoder 1X |
| 7 | FTREL2 | Relative encoder 2X |
| 8 | FTNOTE | Note display |
| 9 | FT16K | 14-bit encoder display |
| 15 | FTLABEL2 | Display name + string in CNSXBUFFER |
| 16 | FTLED | LED on/off display |
| 17 | FTVPOT | Logic VPot display |

## Control Groups by Physical Hardware

### Encoders (RS1): Controls 1-8
- CC# 7, 10, 74, 71, 73, 72, 91, 92
- Offsets in template start at CONTROLSSTART1

### Pots (RS4): Controls 9-16
- CC# 78, 79, 80, 81, 82, 83, 85, 86
- Continuous values 0-127

### Sliders (RS6): Controls 17-24
- CC# 16, 17, 18, 19, 20, 21, 22, 23
- Continuous values 0-127

### Buttons A (Top Left, RS1): Controls 25-32
- CC# 20, 21, 22, 23, 24, 25, 26, 27
- Press/release with toggle mode

### Buttons B (Bottom Left, RS3): Controls 33-40
- CC# 87, 88, 89, 90, 91, 92, 93, 94
- Press/release with toggle mode

### Buttons C (Top Right, RS7): Controls 41-48
- CC# 40, 41, 42, 43, 44, 45, 46, 47
- Press/release with toggle mode

### Buttons D (Bottom Right, RS8): Controls 49-56
- CC# 28, 29, 30, 31, 85, 86, 'Not used', 'Not used'
- Press/release with toggle mode

### Drumpads (RS5): Controls 57-64
- Note on/off with velocity
- Pad names: "Pad 1" through "Pad 8"

### Misc Controls: Controls 65-72
- 65: Expression pedal (CC# 'Exp')
- 66: Sustain pedal (CC# 'Sus Ped')
- 67: ModWheel (CC# 'ModWheel')
- 68: PitchBend
- 69-70: Touchpad X1, Y1
- 71-72: Spare

### Transport Controls: Controls 73-78
- 73: Rewind
- 74: Forward
- 75: Stop
- 76: Play
- 77: Record
- 78: Loop

### Spares1 & Spares2: Controls 79-90
- Reserved for future use

## Template Attributes (Offset 0x5E)

Bit flags:

| Bit | Name | Description |
|-----|------|-------------|
| 0 | PAPOTMODE EQU 0 | Template pot mode |
| 1 | PANOSNAPS EQU 1 | No auto-snapshot bit (not used) |
| 2 | PAAFTERDIS EQU 2 | After-touch disable bit |

## Template Upgrade Bits (Offset 0x60)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | PBSRTRANS | Transport transpose |
| 1 | PBSRPORTS | Port routing |
| 2 | PBCLRSPARES | Clear spares |
| 3 | PBDRTRANS | Drumpad transpose bit |
| 4 | PBZNCMNKBD | Template zone ports/MIDI channel CMN/KBD upgrade |
| 5 | PBSWAPTOGGLES | Swap SL-Template toggle button LO/HIGH values |

## Port Routing Bits (TYPE field)

Used in CNPORTS and zone routing:

| Bits | Description |
|------|-------------|
| 0-1 | Port selection (00=Comm, 01=KeybPort, 10=Specific, 11=invalid) |
| 2-5 | Specific port (41h=M1-Out, 42h=M2-Out, 44h=USB Port1, 48h=USB Port2, 50h=USB Port3/Hidden) |

## MIDI Channel Bits (CNMCHAN field)

| Bits | Description |
|------|-------------|
| 0-3 | MIDI channel (0-15 for channels 1-16) |
| 4-5 | Type (00=Comm, 01=KeybPort, 10=Specific, 11=invalid) |

## Implementation Notes

1. **Control Size**: All controls are exactly 41 bytes (0x29)
2. **14-bit Values**: Low/High values use 14-bit format (7 bits per byte)
3. **Name Padding**: Names are space-padded ASCII, not null-terminated
4. **SysEx Buffer**: 12 bytes for custom SysEx data (CNSXBUF)
5. **Attribute Bits**: Extensive bit flags control behavior (toggle, invert, 2-byte, etc.)
6. **Display Formats**: 18 different display types for LCD feedback

## Memory Layout Summary

```
Template Header:    0x000 - 0x196  (406 bytes)
Control 1 (Enc1):   0x197 - 0x1BF  (41 bytes)
Control 2 (Enc2):   0x1C0 - 0x1E8  (41 bytes)
...
Control 90:         calculated offset
```

Total template size varies based on number of controls, but header is always 406 bytes (0x196).
