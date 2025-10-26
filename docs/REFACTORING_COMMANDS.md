# Refactoring Commands: Separating Domain Model from Wire Format

## Problem

Current `AutomapCommand` types expose wire format details in their semantic API:
- `EncoderRingValue { pos_0_to_11: u8 }` - exposes 0-11 encoding range
- `RowLhBitmap { bits: u8 }` - exposes raw bit manipulation
- `RowRhBitmap { bits: u8 }` - exposes raw bit manipulation

This violates separation of concerns and makes the API less type-safe and harder to use correctly.

## Solution

Separate domain model (what the command *means*) from encoding (how it's *transmitted*).

## Proposed Changes

### 1. EncoderRingValue - Use Semantic Type

**Before**:
```rust
EncoderRingValue {
    encoder: Encoder,
    pos_0_to_11: u8  // Wire format leaking into API
}
```

**After**:
```rust
EncoderRingValue {
    encoder: Encoder,
    position: EncoderPosition  // Semantic type
}

/// Encoder ring LED position (0-11 on the physical ring)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncoderPosition(u8);

impl EncoderPosition {
    /// Create a position (0-11)
    pub const fn new(pos: u8) -> Option<Self> {
        if pos <= 11 {
            Some(Self(pos))
        } else {
            None
        }
    }

    /// Create position, clamping to valid range
    pub const fn new_clamped(pos: u8) -> Self {
        if pos <= 11 {
            Self(pos)
        } else {
            Self(11)
        }
    }

    /// Get the raw position value (0-11)
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Minimum position (fully counter-clockwise)
    pub const MIN: Self = Self(0);

    /// Maximum position (fully clockwise)
    pub const MAX: Self = Self(11);

    /// Center position
    pub const CENTER: Self = Self(6);
}

// Encoding stays in encode_into()
impl AutomapCommand {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        match self {
            AutomapCommand::EncoderRingValue { encoder, position } => {
                out.extend_from_slice(&[
                    AUTOMAP_CC_STATUS,
                    (encoder as u8) - 0x08,
                    position.get(),  // Convert to wire format here
                ]);
            }
            // ...
        }
    }
}
```

**Benefits**:
- Type-safe: can't create invalid positions
- Self-documenting: `EncoderPosition::CENTER` is clearer than `6`
- Encoding logic stays in encoding layer

### 2. RowLhBitmap - Use Semantic Collection

**Before**:
```rust
RowLhBitmap {
    bits: u8  // Raw bits: RS5(bit4), RS4(bit3), RS3(bit2), RS2(bit1), RS1(bit0)
}
```

**After - Option A (Bool Array)**:
```rust
RowLhBitmap {
    rs1: bool,
    rs2: bool,
    rs3: bool,
    rs4: bool,
    rs5: bool,
}

impl RowLhBitmap {
    /// Create from row-select states
    pub const fn new(rs1: bool, rs2: bool, rs3: bool, rs4: bool, rs5: bool) -> Self {
        Self { rs1, rs2, rs3, rs4, rs5 }
    }

    /// All off
    pub const fn all_off() -> Self {
        Self::new(false, false, false, false, false)
    }

    /// All on
    pub const fn all_on() -> Self {
        Self::new(true, true, true, true, true)
    }

    /// Set specific row-select
    pub fn with_rs1(mut self, on: bool) -> Self {
        self.rs1 = on;
        self
    }
    // ... similar for rs2-rs5
}

// Encoding
impl AutomapCommand {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        match self {
            AutomapCommand::RowLhBitmap { rs1, rs2, rs3, rs4, rs5 } => {
                let bits = (rs1 as u8)
                    | ((rs2 as u8) << 1)
                    | ((rs3 as u8) << 2)
                    | ((rs4 as u8) << 3)
                    | ((rs5 as u8) << 4);
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x60, bits & 0x7F]);
            }
            // ...
        }
    }
}
```

**After - Option B (Bitflags)**:
```rust
bitflags::bitflags! {
    /// Left-hand row-select LED states
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RowSelectLhSet: u8 {
        const RS1 = 0b00001;
        const RS2 = 0b00010;
        const RS3 = 0b00100;
        const RS4 = 0b01000;
        const RS5 = 0b10000;
    }
}

RowLhBitmap {
    rows: RowSelectLhSet
}

// Usage:
let cmd = AutomapCommand::RowLhBitmap {
    rows: RowSelectLhSet::RS1 | RowSelectLhSet::RS3 | RowSelectLhSet::RS5
};

// Encoding
impl AutomapCommand {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        match self {
            AutomapCommand::RowLhBitmap { rows } => {
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x60, rows.bits() & 0x7F]);
            }
            // ...
        }
    }
}
```

**After - Option C (Array + Builder)**:
```rust
/// Left-hand row-select states (5 row-selects)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowSelectLhSet([bool; 5]);

impl RowSelectLhSet {
    pub const fn new(states: [bool; 5]) -> Self {
        Self(states)
    }

    pub const fn all_off() -> Self {
        Self([false; 5])
    }

    pub const fn all_on() -> Self {
        Self([true; 5])
    }

    /// Set state of specific row-select (1-based indexing: RS1-RS5)
    pub fn set(&mut self, rs_number: u8, on: bool) {
        if rs_number >= 1 && rs_number <= 5 {
            self.0[(rs_number - 1) as usize] = on;
        }
    }

    /// Get state of specific row-select
    pub fn get(&self, rs_number: u8) -> bool {
        if rs_number >= 1 && rs_number <= 5 {
            self.0[(rs_number - 1) as usize]
        } else {
            false
        }
    }

    /// Iterate over (row_select_number, is_on) pairs
    pub fn iter(&self) -> impl Iterator<Item = (u8, bool)> + '_ {
        (1..=5).map(|n| (n, self.get(n)))
    }
}

RowLhBitmap {
    rows: RowSelectLhSet
}

// Encoding
impl AutomapCommand {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        match self {
            AutomapCommand::RowLhBitmap { rows } => {
                let bits = rows.0.iter()
                    .enumerate()
                    .fold(0u8, |acc, (i, &on)| acc | ((on as u8) << i));
                out.extend_from_slice(&[AUTOMAP_CC_STATUS, 0x60, bits & 0x7F]);
            }
            // ...
        }
    }
}
```

**Recommendation**: **Option B (Bitflags)** - Best balance of ergonomics and type safety
- Natural set operations (union, intersection)
- Type-safe (can't set invalid bits)
- Familiar pattern in Rust
- Clear semantics: `RowSelectLhSet::RS1 | RowSelectLhSet::RS3`

### 3. RowRhBitmap - Same Treatment

**Before**:
```rust
RowRhBitmap {
    bits: u8  // RS8(bit2), RS7(bit1), RS6(bit0), REC(bit3)
}
```

**After**:
```rust
bitflags::bitflags! {
    /// Right-hand row-select LED states
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RowSelectRhSet: u8 {
        const RS6 = 0b0001;
        const RS7 = 0b0010;
        const RS8 = 0b0100;
        const REC = 0b1000;  // Record LED
    }
}

RowRhBitmap {
    rows: RowSelectRhSet
}
```

### 4. EchoRequest - Keep as-is (or add type)

**Current**:
```rust
EchoRequest { value: u8 }  // Echo back arbitrary value for testing
```

**Options**:
1. **Keep as-is** - It's genuinely an opaque echo value
2. **Add newtype** for clarity:
   ```rust
   /// Opaque value for echo testing
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct EchoValue(pub u8);

   EchoRequest { value: EchoValue }
   ```

**Recommendation**: Keep as-is - the semantic meaning truly is "arbitrary byte to echo"

## Implementation Steps

1. **Add new types to `cc.rs`**:
   ```rust
   // src/automap/cc.rs

   pub struct EncoderPosition(u8);
   // ... implementation

   bitflags::bitflags! {
       pub struct RowSelectLhSet: u8 { ... }
   }

   bitflags::bitflags! {
       pub struct RowSelectRhSet: u8 { ... }
   }
   ```

2. **Update `command.rs`**:
   - Change enum variant types
   - Update encoding in `encode_into()`
   - Keep tests, update assertions

3. **Update tests**:
   ```rust
   #[test]
   fn test_encoder_ring_value() {
       let cmd = AutomapCommand::EncoderRingValue {
           encoder: Encoder::Encoder1,
           position: EncoderPosition::new(5).unwrap(),
       };
       assert_eq!(cmd.to_bytes(), vec![0xBF, 0x70, 0x05]);
   }

   #[test]
   fn test_row_bitmap() {
       let cmd = AutomapCommand::RowLhBitmap {
           rows: RowSelectLhSet::RS2 | RowSelectLhSet::RS5,
       };
       assert_eq!(cmd.to_bytes(), vec![0xBF, 0x60, 0x12]);
   }
   ```

4. **Update usage in `demo.rs`** (minimal changes needed)

## Benefits

### Type Safety
```rust
// Before: Easy to make mistakes
let cmd = AutomapCommand::EncoderRingValue {
    encoder: Encoder::Encoder1,
    pos_0_to_11: 15  // Oops! Out of range, clamped at runtime
};

// After: Compile-time safety
let cmd = AutomapCommand::EncoderRingValue {
    encoder: Encoder::Encoder1,
    position: EncoderPosition::new(15)?  // Returns None, must handle
};
```

### Clarity
```rust
// Before: What does 0x12 mean?
let cmd = AutomapCommand::RowLhBitmap { bits: 0x12 };

// After: Crystal clear
let cmd = AutomapCommand::RowLhBitmap {
    rows: RowSelectLhSet::RS2 | RowSelectLhSet::RS5
};
```

### Maintainability
```rust
// Encoding logic centralized in one place
impl AutomapCommand {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        // All wire format knowledge here
        // Domain types stay clean
    }
}
```

## Migration Path

1. Add new types alongside old ones (non-breaking)
2. Add deprecation warnings to old variants
3. Update examples and documentation
4. Remove deprecated variants in next major version

Or: Do breaking change immediately (project is pre-1.0)

## Summary

| Type | Issue | Solution | Benefit |
|------|-------|----------|---------|
| `EncoderRingValue` | `pos_0_to_11: u8` | `position: EncoderPosition` | Type-safe position |
| `RowLhBitmap` | `bits: u8` | `rows: RowSelectLhSet` | Clear set semantics |
| `RowRhBitmap` | `bits: u8` | `rows: RowSelectRhSet` | Clear set semantics |
| `EchoRequest` | `value: u8` | Keep as-is | Already semantic |

**Result**: Clean domain model with encoding isolated to `encode_into()`
