mod stopwatch;
mod gui;
mod bpm;

use evdev::{Device, EventType, KeyCode};
use std::error::Error;
use std::sync::mpsc;
use std::thread;
//use std::io::{self, Write};
//use std::time::{Duration, Instant};


use gui::StopwatchApp;

fn main() -> Result<(), Box<dyn Error>> {
    let device_path="/dev/input/event3";
    let device = Device::open(device_path)?;
    let (tx, rx) = mpsc::channel::<KeyCode>();

    // Spawn background thread to handle blocking evdev reads
    thread::spawn(move || {
        let mut device = device;
        loop {
            if let Ok(events) = device.fetch_events() {
                for event in events {
                    //println!("event {:#?}",event);
                    if event.event_type() == EventType::KEY {
                        if event.value() == 1 {
                            let _ = tx.send(KeyCode(event.code()));
                            //println!("event {:#?}",event);
                        }
                    }
                }
            }
        }
    });

    
    let device = Device::open(device_path)?;
    println!("Listening to device: {}", device.name().unwrap_or("Unknown Device"));

    /*

    let mut stopwatch=Stopwatch::new();
    let tick_rate = Duration::from_millis(40);
    loop {
        let loop_start = Instant::now();
        while let Ok(key) = rx.try_recv() {
            match key {
                
                KeyCode::KEY_Q => {
                    stopwatch.reset();
                }
                
                _ => {}
            }
        }
        stopwatch.tick();

        stopwatch.print();

        let elapsed = loop_start.elapsed();
        if elapsed < tick_rate {
            thread::sleep(tick_rate - elapsed);
        }
    }*/

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([200.0, 110.0])
            .with_always_on_top()
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "Stopwatch Overlay",
        options,
        Box::new(|cc| Ok(Box::new(StopwatchApp::new(cc, rx)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn Error>)


}
