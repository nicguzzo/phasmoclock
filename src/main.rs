#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod bpm;
mod config;
mod gui;
mod stopwatch;

use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, rgba, size,
};

use crate::config::Config;
use gui::StopwatchApp;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

#[derive(Debug)]
pub enum AppKey {
    Reset,
    Tap,
    Mult,
    Bm,
}
type ConfigShared = Arc<Mutex<Config>>;
// --------------------------------------------------------
// LINUX IMPLEMENTATION (evdev)
// --------------------------------------------------------
#[cfg(target_os = "linux")]
fn start_input_thread(tx: mpsc::Sender<AppKey>, config: ConfigShared) {
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
fn start_input_thread(tx: mpsc::Sender<AppKey>, config: ConfigShared) {
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

fn main() {
    let (tx, rx) = mpsc::channel::<AppKey>();
    let config: ConfigShared = Arc::new(Mutex::new(Config::load_config()));
    start_input_thread(tx, config.clone());

    let icon_data = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();

    /*let (width, height) = image.dimensions();
    let egui_icon = eframe::egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let mut options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([210.0, 125.0])
            .with_transparent(true)
            .with_always_on_top()
            .with_decorations(false)
            .with_icon(Arc::new(egui_icon)),
        ..Default::default()
    };

    eframe::run_native(
        "Phasmoclock",
        options,
        Box::new(|cc| Ok(Box::new(StopwatchApp::new(cc, rx, config.clone())))),
    )
    .map_err(|e| Box::new(e) as Box<dyn Error>)*/
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(210.0), px(125.0)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };
        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = StopwatchApp::view(window, cx, rx, config.clone());
                view
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
        //cx.open_window(
        //    ,
        //    |_, cx| cx.new(|_| StopwatchApp::new(cx, rx, config.clone())),
        //)
        //.unwrap();
    });
}
