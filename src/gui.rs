use eframe::egui;
use evdev::KeyCode;
use std::sync::mpsc;
use std::time::Duration;
use crate::bpm::BpmTracker;
use crate::stopwatch::Stopwatch;

pub struct StopwatchApp {
    stopwatch: Stopwatch,
    bpm_tracker: BpmTracker,
    rx: mpsc::Receiver<KeyCode>,
}

impl StopwatchApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rx: mpsc::Receiver<KeyCode>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "my_clock_font".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/digital-7 (mono).ttf"))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "my_clock_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        Self {
            stopwatch: Stopwatch::new(),
            bpm_tracker: BpmTracker::new(),
            rx,
        }
    }
}

impl eframe::App for StopwatchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
   ui.vertical_centered(|ui| {
       ui.add_space(10.0);
       ui.label(format!("mul: {}% , bm: {}",
                        self.bpm_tracker.get_speed_multiplier(),
                        self.bpm_tracker.is_blood_mode()
       ));
       ui.add_space(5.0);
            let last_secs_str = format!("last: {}.{:03}", self.stopwatch.last_seconds,self.stopwatch.last_milliseconds);

            ui.label(egui::RichText::new(last_secs_str)
                        .size(20.0)
                        .strong()
                        .monospace().color(egui::Color32::LIGHT_BLUE));

       ui.add_space(20.0);
            let secs_str = format!("{}.", self.stopwatch.seconds);
            let ms_str = format!("{:03}", self.stopwatch.milliseconds);
            ui.horizontal_top(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 25.0);
                ui.label(egui::RichText::new(secs_str)
                        .size(46.0)
                        .strong()
                        .monospace().color(egui::Color32::GREEN),
                );
                ui.label(egui::RichText::new(ms_str)
                            .size(22.0)
                            .strong()
                            .monospace().color(egui::Color32::GREEN));
            });
        });
        ui.add_space(20.0);
        let (bpm, speed_ms) = self.bpm_tracker.calculate();
        let stats_color = if bpm > 0.0 {
            ui.visuals().text_color()
        } else {
            ui.visuals().text_color().linear_multiply(0.3) // Dim when stopped
        };
        ui.label(
                egui::RichText::new(format!("{:>5.1} BPM  |  {:>4.2} m/s", bpm, speed_ms))
                    .size(20.0)
                    .monospace()
                    .color(egui::Color32::YELLOW),
            );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(key) = self.rx.try_recv() {
            //println!("keycode: {:#?}",&key);
            match key {
                KeyCode::KEY_Q => {
                    self.stopwatch.reset();
                },
                KeyCode::KEY_F => {
                    self.bpm_tracker.tap();
                },
                KeyCode::KEY_F2 => {
                    self.bpm_tracker.cycle_multiplier();
                },
                KeyCode::KEY_F3 => {
                    self.bpm_tracker.toggle_blood_moon();
                }
                _ => {}
            }
        }
        self.stopwatch.tick();
        self.bpm_tracker.tick();
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}