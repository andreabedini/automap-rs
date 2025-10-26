# Template API Implementation Plan

**Status**: 📋 Planned
**Created**: 2026-02-10
**Prerequisites**: Complete protocol coverage (100% ✅), full documentation available

## Overview

Implement high-level APIs for creating, parsing, and manipulating SLMKII/ZeroMKII templates without requiring Novation's Automap Server software.

## Motivation

Currently, the codebase has complete low-level protocol support (CC messages, SysEx, Data-Block read/write), but working with templates requires:
- Manual offset calculations
- Raw byte manipulation
- Understanding of 41-byte control structures
- Knowledge of bit-field layouts

With template structure documentation now available (see `TEMPLATE_STRUCTURE.md`), we can build user-friendly APIs.

## Available Documentation

All necessary documentation is complete:
1. ✅ `SLMKII MIDI Programmers Reference.pdf` - Core protocol
2. ✅ `SLMKII Template Offsets.pdf` - Template layout (406-byte header + controls)
3. ✅ `SL Control Members etc.pdf` - Control structure (41 bytes per control)
4. ✅ `TEMPLATE_STRUCTURE.md` - Comprehensive summary with all offsets and fields
5. ✅ `PROTOCOL_COVERAGE.md` - Implementation status (100% coverage)

## Goals

### Primary Goals
1. **Parse templates** - Read template data from device into Rust structs
2. **Build templates** - Create templates programmatically with fluent API
3. **Modify templates** - Edit existing templates and write back to device
4. **Serialize templates** - Save/load templates to/from disk

### Secondary Goals
5. **Template presets** - Pre-built templates for popular DAWs
6. **Interactive editor** - CLI tool for template creation
7. **Template library** - Backup/restore template collections

## Implementation Phases

### Phase 1: Core Data Structures (2-3 hours)

**Goal**: Rust structs matching the binary template format

**Files to modify**:
- `src/automap/protocol/template.rs` (expand existing file)

**Structures to implement**:

```rust
/// Complete template structure (header + controls)
pub struct Template {
    pub header: TemplateHeader,
    pub controls: Vec<Control>,
}

/// Template header (406 bytes at offset 0x00-0x196)
pub struct TemplateHeader {
    pub name: String,                    // 0x00: 8-byte ASCII
    pub template_number: u8,              // 0x30: 0-31, 0xFF=Automap
    pub version: (u8, u8),                // 0x31: BCD version
    pub template_type: TemplateType,      // 0x33: Normal/Reason3/LogicController
    pub group_size: u8,                   // 0x34: Templates in group
    pub group_position: u8,               // 0x35: Position in group
    pub control_count: u16,               // 0x36: Number of controls (14-bit)
    pub control_size: u16,                // 0x38: Size per control (41)
    pub zones: [Zone; 4],                 // 0x62-0x87: 4 zones × 10 bytes
    pub keyboard_channel: u8,             // 0x56: Keyboard MIDI channel
    pub common_channel: u8,               // 0x58: Common MIDI channel
    pub velocity: u8,                     // 0x5C: Keyboard velocity
    pub octave: u8,                       // 0x5D: Octave setting
    pub attributes: TemplateAttributes,   // 0x5E: Bit flags
    pub upgrade_bits: UpgradeBits,        // 0x60: Template upgrade flags
    // ... other header fields from TEMPLATE_STRUCTURE.md
}

/// MIDI zone configuration (10 bytes)
pub struct Zone {
    pub midi_channel: u8,                 // +0: 0x00-0x0F (channels 1-16)
    pub routing: PortRouting,             // +1: Port routing bits
    pub velocity: u8,                     // +2: MIDI velocity
    pub note_low: u8,                     // +3: Lowest note
    pub note_high: u8,                    // +4: Highest note
    pub transpose: i8,                    // +5: Transpose amount
    pub attributes: ZoneAttributes,       // +6: PB+MW+AT bits
}

/// Control definition (41 bytes)
pub struct Control {
    pub name: String,                     // 0x00: 8-byte ASCII
    pub control_type: ControlType,        // 0x08: Type enum
    pub value_low: u16,                   // 0x09-0x0A: 14-bit low value
    pub value_high: u16,                  // 0x0B-0x0C: 14-bit high value
    pub attributes: ControlAttributes,    // 0x0D-0x0F: 3 attribute bytes
    pub cc_msb: u8,                       // 0x10: MIDI CC MSB
    pub cc_lsb: u8,                       // 0x11: MIDI CC LSB
    pub ports: PortRouting,               // 0x12: Port routing
    pub midi_channel: u8,                 // 0x13: MIDI channel
    pub display_type: DisplayType,        // 0x19: LCD display format
    pub sysex_buffer: [u8; 12],           // 0x1C: SysEx data
}

// Enums
#[repr(u8)]
pub enum TemplateType {
    Normal = 0x00,
    Reason3 = 0x01,
    LogicController = 0x02,
}

#[repr(u8)]
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
    ProgramChange = 9,
    PitchBend = 10,
    DrumNote = 11,
    TemplateChange = 12,
    RealTime = 13,
    TemplateGroup = 14,
}

#[repr(u8)]
pub enum DisplayType {
    FT127 = 0,           // 0-127
    FT6463 = 1,          // -64 to +63
    FTMMC = 2,           // MMC
    FTOFFON = 3,         // OFF/ON
    FTBLANK = 4,         // Blank
    FTLABEL = 5,         // Control name
    FTREL1 = 6,          // Relative encoder 1X
    FTREL2 = 7,          // Relative encoder 2X
    FTNOTE = 8,          // Note display
    FT16K = 9,           // 14-bit encoder
    FTLABEL2 = 15,       // Name + string in SysEx buffer
    FTLED = 16,          // LED on/off
    FTVPOT = 17,         // Logic VPot
}

// Bitflags for attributes
bitflags::bitflags! {
    pub struct ControlAttributes1: u8 {
        const SEND_MS_FIRST = 0b00000001;      // Send MS value first
        const SEND_2BYTE = 0b00000010;         // Send 2-byte value
        const RELEASE_TOO = 0b00000100;        // Send on release
        const TOGGLE = 0b00001000;             // Toggle mode
        const CYCLIC_BUTTON = 0b00010000;      // Cyclic/step button
        const RAW_DATA = 0b01000000;           // Use raw SysEx data
    }
}

bitflags::bitflags! {
    pub struct ControlAttributes2: u8 {
        const SEND_ONE_CN = 0b00000001;        // Send one control number
        const LS_CN_FIRST = 0b00000010;        // LS control first
        const NO_SNAPSHOT = 0b00000100;        // Don't send in snapshot
        const INVERT = 0b00001000;             // Invert value
        const ENCODER_CATCHUP = 0b00010000;    // Encoder catchup mode
    }
}

pub struct ControlAttributes {
    pub attr1: ControlAttributes1,
    pub attr2: ControlAttributes2,
    pub display_type: DisplayType,  // From attr3 bits 0-4
}

// Port routing bitflags
bitflags::bitflags! {
    pub struct PortRouting: u8 {
        const COMM_PORT = 0b00;
        const KEYB_PORT = 0b01;
        const SPECIFIC = 0b10;
        const M1_OUT = 0x41;
        const M2_OUT = 0x42;
        const USB_PORT1 = 0x44;
        const USB_PORT2 = 0x48;
        const USB_PORT3_HIDDEN = 0x50;
    }
}
```

**Success criteria**:
- All structs match binary layout documented in `TEMPLATE_STRUCTURE.md`
- Comprehensive doc comments with offset references
- Type safety with enums and bitflags

---

### Phase 2: Parser & Serializer (2-3 hours)

**Goal**: Convert between binary data and Rust structs

**Implementation**:

```rust
impl Template {
    /// Parse template from Data-Block response bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 0x196 {
            return Err(ParseError::TooShort);
        }

        // Parse header (0x00-0x196)
        let header = TemplateHeader::from_bytes(&data[0..0x196])?;

        // Parse controls (41 bytes each, starting after header)
        let control_count = header.control_count as usize;
        let mut controls = Vec::with_capacity(control_count);

        let mut offset = 0x196;  // Start after header
        for _ in 0..control_count {
            if data.len() < offset + 41 {
                return Err(ParseError::TruncatedControl);
            }
            controls.push(Control::from_bytes(&data[offset..offset+41])?);
            offset += 41;
        }

        Ok(Template { header, controls })
    }

    /// Serialize template to bytes for upload to device
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Serialize header
        self.header.to_bytes(&mut buf);

        // Serialize controls
        for control in &self.controls {
            control.to_bytes(&mut buf);
        }

        buf
    }
}

impl TemplateHeader {
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        // Parse each field from documented offsets
        let name = std::str::from_utf8(&data[0x00..0x08])
            .map_err(|_| ParseError::InvalidName)?
            .trim_end()
            .to_string();

        let template_number = data[0x30];
        let version = (data[0x31], data[0x32]);
        let template_type = TemplateType::try_from(data[0x33])?;

        // ... parse all other fields per TEMPLATE_STRUCTURE.md

        Ok(TemplateHeader { /* ... */ })
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        // Serialize each field to documented offsets
        // Ensure buffer is exactly 0x196 bytes
    }
}

impl Control {
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 41 {
            return Err(ParseError::TooShort);
        }

        let name = std::str::from_utf8(&data[0x00..0x08])
            .map_err(|_| ParseError::InvalidName)?
            .trim_end()
            .to_string();

        let control_type = ControlType::try_from(data[0x08])?;

        // 14-bit values (see unpack_u14 in sysex.rs)
        let value_low = ((data[0x09] as u16) | ((data[0x0A] as u16) << 7));
        let value_high = ((data[0x0B] as u16) | ((data[0x0C] as u16) << 7));

        let attr1 = ControlAttributes1::from_bits_truncate(data[0x0D]);
        let attr2 = ControlAttributes2::from_bits_truncate(data[0x0E]);
        let display_type = DisplayType::try_from(data[0x0F] & 0x1F)?;

        // ... parse remaining fields

        Ok(Control { /* ... */ })
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        // Serialize to exactly 41 bytes
    }
}

#[derive(Debug)]
pub enum ParseError {
    TooShort,
    TruncatedControl,
    InvalidName,
    InvalidControlType(u8),
    InvalidDisplayType(u8),
    InvalidTemplateType(u8),
}
```

**Success criteria**:
- Round-trip: `Template::from_bytes(template.to_bytes()) == template`
- All 41-byte control structure fields parsed correctly
- 406-byte header parsed correctly
- Error handling for malformed data

---

### Phase 3: Builder API (2-3 hours)

**Goal**: Fluent API for creating templates programmatically

**Implementation**:

```rust
/// Fluent template builder
pub struct TemplateBuilder {
    name: String,
    template_type: TemplateType,
    controls: Vec<Control>,
    zones: [Zone; 4],
    keyboard_channel: u8,
    // ... other fields with defaults
}

impl TemplateBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            template_type: TemplateType::Normal,
            controls: Vec::new(),
            zones: [Zone::default(); 4],
            keyboard_channel: 0,
        }
    }

    pub fn template_type(mut self, typ: TemplateType) -> Self {
        self.template_type = typ;
        self
    }

    pub fn keyboard_channel(mut self, channel: u8) -> Self {
        self.keyboard_channel = channel;
        self
    }

    pub fn zone(mut self, index: usize, zone: Zone) -> Self {
        if index < 4 {
            self.zones[index] = zone;
        }
        self
    }

    pub fn encoder(mut self, number: u8, name: impl Into<String>) -> ControlBuilder {
        ControlBuilder::new(self, ControlType::CC, name)
            .hardware_number(number)
            .control_group(ControlGroup::Encoder)
    }

    pub fn pot(mut self, number: u8, name: impl Into<String>) -> ControlBuilder {
        ControlBuilder::new(self, ControlType::CC, name)
            .hardware_number(number)
            .control_group(ControlGroup::Pot)
    }

    pub fn slider(mut self, number: u8, name: impl Into<String>) -> ControlBuilder {
        ControlBuilder::new(self, ControlType::CC, name)
            .hardware_number(number)
            .control_group(ControlGroup::Slider)
    }

    pub fn button(mut self, number: u8, name: impl Into<String>) -> ControlBuilder {
        ControlBuilder::new(self, ControlType::CC, name)
            .hardware_number(number)
            .control_group(ControlGroup::ButtonA)
    }

    pub fn build(self) -> Result<Template, BuildError> {
        // Validate and construct Template
        if self.name.is_empty() {
            return Err(BuildError::EmptyName);
        }

        let header = TemplateHeader {
            name: self.name,
            template_type: self.template_type,
            control_count: self.controls.len() as u16,
            control_size: 41,
            zones: self.zones,
            keyboard_channel: self.keyboard_channel,
            // ... other fields with sensible defaults
        };

        Ok(Template {
            header,
            controls: self.controls,
        })
    }
}

/// Builder for individual controls
pub struct ControlBuilder {
    template_builder: TemplateBuilder,
    control: Control,
}

impl ControlBuilder {
    fn new(tb: TemplateBuilder, typ: ControlType, name: impl Into<String>) -> Self {
        Self {
            template_builder: tb,
            control: Control {
                name: name.into(),
                control_type: typ,
                value_low: 0,
                value_high: 127,
                cc_msb: 0,
                // ... defaults
            },
        }
    }

    pub fn cc(mut self, cc: u8) -> Self {
        self.control.cc_msb = cc;
        self
    }

    pub fn range(mut self, low: u16, high: u16) -> Self {
        self.control.value_low = low;
        self.control.value_high = high;
        self
    }

    pub fn display(mut self, display_type: DisplayType) -> Self {
        self.control.display_type = display_type;
        self
    }

    pub fn toggle(mut self) -> Self {
        self.control.attributes.attr1 |= ControlAttributes1::TOGGLE;
        self
    }

    pub fn invert(mut self) -> Self {
        self.control.attributes.attr2 |= ControlAttributes2::INVERT;
        self
    }

    pub fn midi_channel(mut self, channel: u8) -> Self {
        self.control.midi_channel = channel;
        self
    }

    pub fn and(mut self) -> TemplateBuilder {
        self.template_builder.controls.push(self.control);
        self.template_builder
    }
}

#[derive(Debug)]
pub enum BuildError {
    EmptyName,
    TooManyControls,
    InvalidControlNumber,
}
```

**Usage example**:
```rust
let template = TemplateBuilder::new("MyController")
    .template_type(TemplateType::Normal)
    .keyboard_channel(0)
    .zone(0, Zone {
        midi_channel: 0,
        routing: PortRouting::USB_PORT3_HIDDEN,
        velocity: 100,
        note_low: 36,
        note_high: 96,
        transpose: 0,
        attributes: ZoneAttributes::default(),
    })
    .encoder(1, "Cutoff")
        .cc(74)
        .range(0, 127)
        .display(DisplayType::FT127)
        .and()
    .pot(1, "Resonance")
        .cc(71)
        .range(0, 127)
        .display(DisplayType::FT127)
        .and()
    .button(1, "Mute")
        .cc(20)
        .toggle()
        .display(DisplayType::FTLED)
        .and()
    .build()?;
```

**Success criteria**:
- Ergonomic API with method chaining
- Sensible defaults for all fields
- Type-safe control configuration
- Clear error messages

---

### Phase 4: Device Integration (1-2 hours)

**Goal**: Methods for reading/writing templates to/from device

**Add to `AutomapDevice` in `device.rs`**:

```rust
impl AutomapDevice {
    /// Request a template from the device
    pub async fn request_template(&mut self, template_num: u8) -> Result<Template, std::io::Error> {
        // Use Data-Block read with Template-Header target
        // Read header first, then controls based on control_count
        let header_data = self.read_template_header(template_num).await?;
        let template = Template::from_bytes(&header_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(template)
    }

    /// Upload a template to the device
    pub async fn upload_template(&mut self, template_num: u8, template: &Template) -> Result<(), std::io::Error> {
        let data = template.to_bytes();

        // Use Cmd 07 (Upload Template) SysEx command
        let mut payload = vec![template_num];
        payload.extend_from_slice(&data);

        self.send_sysex(AutomapSysEx::UploadTemplate {
            data: &payload
        }).await
    }

    /// Read current template from RAM
    pub async fn read_current_template(&mut self) -> Result<Template, std::io::Error> {
        // Use Data-Block read with offset 0, length = template size
        // Template-Header target, control 0
        todo!()
    }

    /// Modify a single control in the current template
    pub async fn write_control(&mut self, control_num: u8, control: &Control) -> Result<(), std::io::Error> {
        let data = control.to_bytes();

        // Use Data-Block write
        // Control-Data target, control number, offset 0, 41 bytes
        todo!()
    }
}
```

**Success criteria**:
- Read template from any slot (0-31)
- Upload template to any slot
- Modify individual controls without rewriting entire template
- Error handling for device communication failures

---

### Phase 5: Example Applications (2-3 hours)

**Goal**: Demonstrate template APIs with practical examples

#### Example 1: Template Info (`examples/template-info.rs`)

```rust
/// Read and display template information from device
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = AutomapDevice::new().await?;

    // Request template 1
    println!("Requesting template 1...");
    let template = device.request_template(1).await?;

    println!("Template: {}", template.header.name);
    println!("Type: {:?}", template.header.template_type);
    println!("Controls: {}", template.controls.len());

    println!("\nEncoders:");
    for (i, control) in template.controls.iter()
        .filter(|c| matches!(c.control_type, ControlType::CC))
        .take(8)
        .enumerate()
    {
        println!("  {}: {} (CC#{})", i+1, control.name, control.cc_msb);
    }

    Ok(())
}
```

#### Example 2: Template Builder (`examples/template-builder.rs`)

```rust
/// Create a custom template and upload to device
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = AutomapDevice::new().await?;

    // Build a simple mixer template
    let template = TemplateBuilder::new("8ChMixer")
        .template_type(TemplateType::Normal)
        // 8 pots for volume
        .pot(1, "Vol 1").cc(7).range(0, 127).display(DisplayType::FT127).and()
        .pot(2, "Vol 2").cc(8).range(0, 127).display(DisplayType::FT127).and()
        .pot(3, "Vol 3").cc(9).range(0, 127).display(DisplayType::FT127).and()
        .pot(4, "Vol 4").cc(10).range(0, 127).display(DisplayType::FT127).and()
        .pot(5, "Vol 5").cc(11).range(0, 127).display(DisplayType::FT127).and()
        .pot(6, "Vol 6").cc(12).range(0, 127).display(DisplayType::FT127).and()
        .pot(7, "Vol 7").cc(13).range(0, 127).display(DisplayType::FT127).and()
        .pot(8, "Vol 8").cc(14).range(0, 127).display(DisplayType::FT127).and()
        // 8 buttons for mute
        .button(1, "Mute1").cc(20).toggle().display(DisplayType::FTLED).and()
        .button(2, "Mute2").cc(21).toggle().display(DisplayType::FTLED).and()
        .button(3, "Mute3").cc(22).toggle().display(DisplayType::FTLED).and()
        .button(4, "Mute4").cc(23).toggle().display(DisplayType::FTLED).and()
        .button(5, "Mute5").cc(24).toggle().display(DisplayType::FTLED).and()
        .button(6, "Mute6").cc(25).toggle().display(DisplayType::FTLED).and()
        .button(7, "Mute7").cc(26).toggle().display(DisplayType::FTLED).and()
        .button(8, "Mute8").cc(27).toggle().display(DisplayType::FTLED).and()
        .build()?;

    println!("Uploading template to slot 10...");
    device.upload_template(10, &template).await?;
    println!("Done!");

    Ok(())
}
```

#### Example 3: Template Backup (`examples/template-backup.rs`)

```rust
/// Backup all templates from device to disk
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = AutomapDevice::new().await?;

    std::fs::create_dir_all("templates")?;

    // Read all 32 templates
    for i in 0..32 {
        println!("Reading template {}...", i);
        match device.request_template(i).await {
            Ok(template) => {
                let filename = format!("templates/template_{:02}_{}.bin",
                                      i, template.header.name.replace(' ', "_"));
                std::fs::write(&filename, template.to_bytes())?;
                println!("  Saved: {}", filename);
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
    }

    println!("\nBackup complete!");
    Ok(())
}
```

**Success criteria**:
- All examples compile and run
- Clear, commented code
- Demonstrate key functionality
- Handle errors gracefully

---

### Phase 6 (Optional): Template Presets (3-4 hours)

**Goal**: Pre-built templates for popular use cases

**Create `src/automap/presets.rs`**:

```rust
pub mod presets {
    use super::*;

    /// Generic 8-channel mixer template
    pub fn mixer_8ch() -> Template {
        TemplateBuilder::new("Mixer8Ch")
            .template_type(TemplateType::Normal)
            // Pots for volume
            .pot(1, "Vol 1").cc(7).and()
            .pot(2, "Vol 2").cc(8).and()
            // ... 8 channels
            // Buttons for mute
            .button(1, "Mute1").cc(20).toggle().and()
            // ... etc
            .build()
            .expect("preset should be valid")
    }

    /// Logic Pro X template with VPots
    pub fn logic_pro_vpot() -> Template {
        TemplateBuilder::new("LogicPro")
            .template_type(TemplateType::LogicController)
            // Encoders mapped to Logic VPots
            .encoder(1, "VPot1").cc(16).display(DisplayType::FTVPOT).and()
            // ... etc
            .build()
            .expect("preset should be valid")
    }

    /// Ableton Live session control
    pub fn ableton_live_session() -> Template {
        TemplateBuilder::new("AbletonSn")
            // Scene launch buttons
            // Track arm buttons
            // Volume/Pan/Send controls
            todo!()
    }

    /// Generic transport + 8 faders template
    pub fn transport_faders() -> Template {
        todo!()
    }
}
```

**Success criteria**:
- At least 3-4 useful presets
- Well-documented MIDI mappings
- Tested with target software (if possible)

---

## Implementation Time Estimate

| Phase | Description | Estimated Time |
|-------|-------------|----------------|
| 1 | Core data structures | 2-3 hours |
| 2 | Parser & serializer | 2-3 hours |
| 3 | Builder API | 2-3 hours |
| 4 | Device integration | 1-2 hours |
| 5 | Example applications | 2-3 hours |
| 6 | Presets (optional) | 3-4 hours |
| **Total** | **Complete system** | **10-14 hours** |

With optional presets: **13-18 hours**

## Success Criteria

### Functional Requirements
- ✅ Parse template from device
- ✅ Create template programmatically
- ✅ Upload template to device
- ✅ Modify individual controls
- ✅ Round-trip binary serialization
- ✅ Save/load templates from disk

### Quality Requirements
- ✅ Type-safe APIs (no raw bytes in user code)
- ✅ Comprehensive error handling
- ✅ Full documentation with examples
- ✅ Tests for parser/serializer
- ✅ Working example applications

### Performance
- Parse/serialize template in < 1ms (not critical, but should be fast)
- No unnecessary allocations

## Testing Strategy

1. **Unit tests** for parser/serializer:
   ```rust
   #[test]
   fn test_control_roundtrip() {
       let control = Control { /* ... */ };
       let bytes = control.to_bytes();
       let parsed = Control::from_bytes(&bytes).unwrap();
       assert_eq!(control, parsed);
   }
   ```

2. **Integration tests** with actual device:
   - Read template 0 (Automap template)
   - Upload simple test template
   - Verify uploaded template can be read back

3. **Example validation**:
   - All examples compile without warnings
   - Examples run successfully (with hardware when available)

## Future Enhancements (Not in Initial Plan)

- **Template editor TUI** - Full-screen terminal UI for editing
- **Template converter** - Import/export to other formats
- **Template diff** - Compare two templates
- **Template merge** - Combine controls from multiple templates
- **MIDI learn mode** - Auto-map controls by listening to MIDI
- **Web UI** - Browser-based template editor

## References

- `docs/TEMPLATE_STRUCTURE.md` - Complete offset documentation
- `docs/PROTOCOL_COVERAGE.md` - Protocol implementation status
- `docs/SLMKII Template Offsets.pdf` - Original template layout
- `docs/SL Control Members etc.pdf` - Control structure details
- `docs/SLMKII MIDI Programmers Reference.pdf` - Core protocol

## Notes

- Template structure is identical for SLMKII, ZeroMKII, and Nocturn Keyboard
- Control count varies: SLMKII has 90 controls, ZeroMKII has additional Cross-fader control
- Templates are stored in Flash memory (32 slots + 1 Automap template)
- Template data can be read/written using Data-Block SysEx protocol (0x68/0x69 commands)
- Actual control size is always 41 bytes (0x29h), as documented
- 14-bit values use 7 bits per byte (see `unpack_u14` / `pack_u14` in `sysex.rs`)

---

**Last Updated**: 2026-02-10
**Ready to Implement**: ✅ Yes - All prerequisites complete
