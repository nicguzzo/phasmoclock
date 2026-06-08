use std::sync::mpsc;
use std::thread;

use crate::{AppKey, ConfigShared};

#[cfg(target_os = "linux")]
pub fn start_input_thread(tx: mpsc::Sender<AppKey>, config: ConfigShared) {
    use evdev::{Device, EventType};
    let device_path = "/dev/input/event3";

    if let Ok(device) = Device::open(device_path) {
        thread::spawn(move || {
            let mut device = device;
            loop {
                if let Ok(events) = device.fetch_events() {
                    for event in events {
                        //println!("event {:#?}", event);
                        if event.event_type() == EventType::KEY {
                            if event.value() == 1 {
                                let incoming_key = event.code();
                                let config = config.lock().unwrap();
                                if incoming_key == config.reset_code {
                                    let _ = tx.send(AppKey::Reset);
                                } else if incoming_key == config.tap_code {
                                    let _ = tx.send(AppKey::Tap);
                                } else if incoming_key == config.cycle_multiplier_code {
                                    let _ = tx.send(AppKey::Mult);
                                } else if incoming_key == config.blood_moon_code {
                                    let _ = tx.send(AppKey::Bm);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

// --------------------------------------------------------
// WINDOWS IMPLEMENTATION (Using rdev or device_query here)
// --------------------------------------------------------
#[cfg(target_os = "windows")]
pub fn start_input_thread(tx: mpsc::Sender<AppKey>, config: ConfigShared) {
    use rdev::{EventType, listen};
    thread::spawn(move || {
        let _ = listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                use crate::config::rdev_to_win_vk;

                let config = config.lock().unwrap();
                let key = rdev_to_win_vk(key);
                //println!("key {}", key);
                if key == config.reset_code {
                    let _ = tx.send(AppKey::Reset);
                } else if key == config.tap_code {
                    let _ = tx.send(AppKey::Tap);
                } else if key == config.cycle_multiplier_code {
                    let _ = tx.send(AppKey::Mult);
                } else if key == config.blood_moon_code {
                    let _ = tx.send(AppKey::Bm);
                }
            }
        });
    });
}
