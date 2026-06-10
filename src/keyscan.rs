use std::sync::mpsc;
use std::thread;

use crate::{AppKey, ConfigShared};

#[cfg(target_os = "linux")]
fn auto_detect_keyboard() -> Option<String> {
    use evdev::{EventType, KeyCode, enumerate};
    for (path, device) in enumerate() {
        if device.supported_events().contains(EventType::KEY) {
            if let Some(keys) = device.supported_keys() {
                if keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_SPACE) {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn start_input_thread(tx: mpsc::Sender<AppKey>, config: ConfigShared) {
    use evdev::{Device, EventType};
    if let Some(k) = auto_detect_keyboard() {
        println!("autodetected: {}", k);
    }
    let device_path = {
        let conf = config.lock().unwrap();
        let path = conf.keyboard_device.clone();
        if path.is_empty() {
            auto_detect_keyboard().unwrap_or_else(|| "/dev/input/event0".to_string())
        } else {
            path
        }
    };

    println!("Using keyboard device: {}", device_path);

    if let Ok(device) = Device::open(&device_path) {
        thread::spawn(move || {
            let mut device = device;
            loop {
                if let Ok(events) = device.fetch_events() {
                    for event in events {
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
                } else {
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        });
    } else {
        println!("Failed to open keyboard device: {}", device_path);
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
