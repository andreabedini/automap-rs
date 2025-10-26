use nusb::io::EndpointWrite;
use nusb::transfer::{Bulk, In, Out};
use nusb::{self, io::EndpointRead};

// Conditional imports for async traits based on selected runtime
#[cfg(feature = "tokio")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(feature = "smol")]
use futures_lite::{AsyncReadExt, AsyncWriteExt};

use std::error::Error;

use crate::automap::command::AutomapCommand;
use crate::automap::event::AutomapEvent;
use crate::midi::{split_midi_messages, usbmidi_pack, usbmidi_unpack};

use super::sysex::AutomapSysEx;

const VID: u16 = 0x1235;
const PID: u16 = 0x000c;

// ZeRO MkII vendor interface (from your lsusb -v dump)
const IFACE: u8 = 2;

const EP_OUT: u8 = 0x06; // host -> device
const EP_IN: u8 = 0x86; // device -> host

// const USB_PKT: usize = 4; // USB-MIDI event packet size
pub const USB_BUF: usize = 64; // endpoint wMaxPacketSize = 32 bytes => multiple of 4 ok

pub struct AutomapDevice {
    reader: EndpointRead<Bulk>,
    writer: EndpointWrite<Bulk>,
}

impl AutomapDevice {
    pub async fn new() -> Result<AutomapDevice, Box<dyn Error>> {
        let device_info = nusb::list_devices()
            .await?
            .find(|dev| dev.vendor_id() == VID && dev.product_id() == PID)
            .expect("device not found");

        let device = device_info.open().await?;
        let interface = device.claim_interface(IFACE).await?;

        let reader = interface.endpoint::<Bulk, In>(EP_IN)?.reader(64);
        let writer = interface.endpoint::<Bulk, Out>(EP_OUT)?.writer(64);

        Ok(AutomapDevice { reader, writer })
    }

    /// Sends a SysEx message to the device.
    ///
    /// The message is automatically encoded to bytes and packed into USB-MIDI packets.
    ///
    /// # Arguments
    ///
    /// * `msg` - The SysEx message to send
    ///
    /// # Errors
    ///
    /// Returns an error if the USB write fails.
    pub async fn send_sysex(&mut self, msg: AutomapSysEx<'_>) -> Result<(), std::io::Error> {
        self.writer
            .write_all(&usbmidi_pack(&msg.to_bytes()))
            .await?;
        self.writer.flush().await
    }

    /// Sends a command to the device.
    ///
    /// Commands are typically for controlling LEDs and encoder rings.
    /// The command is automatically encoded and packed into USB-MIDI packets.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to send
    ///
    /// # Errors
    ///
    /// Returns an error if the USB write fails.
    pub async fn send_command(&mut self, cmd: &AutomapCommand) -> Result<(), std::io::Error> {
        self.writer
            .write_all(&usbmidi_pack(&cmd.to_bytes()))
            .await?;
        self.writer.flush().await
    }

    /// Reads events from the device.
    ///
    /// This method reads USB-MIDI packets from the device, unpacks them into
    /// raw MIDI bytes, and decodes them into `AutomapEvent` instances.
    ///
    /// # Returns
    ///
    /// A vector of successfully decoded events. Invalid or unrecognized MIDI
    /// messages are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if the USB read fails.
    pub async fn read_events(&mut self) -> Result<Vec<AutomapEvent>, std::io::Error> {
        let mut buf = vec![0u8; USB_BUF];
        let mut events = Vec::new();

        match self.reader.read(&mut buf).await {
            Ok(n) if n >= 4 => {
                let n4 = n - (n % 4);
                let raw = usbmidi_unpack(&buf[..n4]);
                for msg in split_midi_messages(&raw) {
                    if let Ok(event) = AutomapEvent::decode_event(&msg) {
                        events.push(event);
                    }
                }
            }
            Ok(_) => {} // Short read, no complete packets
            Err(e) => return Err(e),
        }

        Ok(events)
    }
}
