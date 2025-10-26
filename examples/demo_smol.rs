use std::error::Error;

use automap::{
    AutomapCommand, AutomapDevice, AutomapEvent, AutomapSysEx, LcdClear, LcdLine, LcdOp,
};

fn main() -> Result<(), Box<dyn Error>> {
    smol::block_on(async {
        let mut automap_device = AutomapDevice::new().await?;

        println!("Reading from ZeRO MkII hidden port... (Ctrl+C to stop)");

        // Bring device online
        automap_device
            .send_sysex(AutomapSysEx::OnlineOffline { online: true })
            .await?;

        // Display "Hello" on the LCD
        let msg = AutomapSysEx::LcdText(vec![
            LcdOp::Clear(LcdClear::LeftAll),
            LcdOp::Cursor {
                col: 9,
                line: LcdLine::LeftTop,
            },
            LcdOp::Text(b"Hello"),
            LcdOp::End,
        ]);
        automap_device.send_sysex(msg).await?;

        // Turn off all LEDs
        automap_device
            .send_command(&AutomapCommand::AllLedsOff)
            .await?;

        // Note: Ctrl+C handling with smol requires additional dependencies
        // For simplicity, this example will run until an error occurs
        // In a real application, consider using async-signal or signal-hook
        loop {
            match automap_device.read_events().await {
                Ok(events) => {
                    for event in events {
                        println!("Received event: {:?}", event);
                        // Echo button presses by toggling corresponding LEDs
                        if let AutomapEvent::Button { button, pressed } = event {
                            let cmd = AutomapCommand::ButtonLed {
                                button,
                                on: pressed,
                            };
                            println!("→ Sending command: {:?}", cmd);
                            automap_device.send_command(&cmd).await?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("USB read error: {e}");
                    break;
                }
            }
        }

        // Send offline command before exiting
        println!("Sending offline command...");
        automap_device
            .send_sysex(AutomapSysEx::OnlineOffline { online: false })
            .await?;
        println!("Done.");

        Ok(())
    })
}
