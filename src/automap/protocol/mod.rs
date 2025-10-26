#![allow(dead_code)]

use cc::Button;
use event::AutomapEvent;
use sysex::{AutomapSysEx, LcdClear, LcdLine, LcdOp, PROTO_VER_BETA, PROTO_VER_MAIN};

pub mod cc;
pub mod command;
pub mod event;
pub mod sysex;
pub mod template;
