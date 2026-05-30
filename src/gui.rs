use crate::bpm::BpmTracker;
use crate::stopwatch::Stopwatch;
use crate::{AppKey, ConfigShared, config};
use eframe::egui::{self, RichText};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum BindingAction {
    Reset,
    Tap,
    CycleMultipliers,
    BloodMoon,
}

pub struct StopwatchApp {
    stopwatch: Stopwatch,
    bpm_tracker: BpmTracker,
    rx: mpsc::Receiver<AppKey>,
    binding_state: Option<BindingAction>,
    show_settings: bool,
    config: ConfigShared,
}

impl StopwatchApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: mpsc::Receiver<AppKey>,
        config: ConfigShared,
    ) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "clock_font".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/digital-7 (mono).ttf"
            ))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "clock_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        Self {
            stopwatch: Stopwatch::new(),
            bpm_tracker: BpmTracker::new(),
            rx,
            binding_state: None,
            show_settings: false,
            config,
        }
    }
}

impl eframe::App for StopwatchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let has_focus = ui.ctx().input(|i| i.focused);
        let mut config = self.config.lock().unwrap();
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if has_focus && ui.button("🗙").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.allocate_ui(egui::vec2(30.0, 20.0), |ui| {
                    if has_focus && ui.button("👁").clicked() {
                        config.toggle_hide_speeds();
                    }
                });
                if !config.get_hide_speeds() {
                    let blood_moon_color = if self.bpm_tracker.is_blood_mode() {
                        egui::Color32::RED
                    } else {
                        egui::Color32::GRAY
                    };
                    if ui
                        .button(RichText::new("🌙").color(blood_moon_color))
                        .clicked()
                    {
                        self.bpm_tracker.toggle_blood_moon();
                    }

                    let bpm_str = format!("{}%", self.bpm_tracker.get_speed_multiplier());
                    if ui.button(bpm_str).clicked() {
                        self.bpm_tracker.cycle_multiplier();
                    }
                    if ui.button("⚙").clicked() {
                        self.show_settings = !self.show_settings;
                        if !self.show_settings {
                            self.binding_state = None;
                        }
                    }
                } else {
                    ui.label(format!(" "));
                }
            });
            if self.show_settings {
                // This creates a dedicated overlay window that refuses to close until you tell it to

                egui::Window::new("Keybind Configuration")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Click an action to rebind it:");
                            if ui.button("🗙").clicked() {
                                self.show_settings = false;
                                self.binding_state = None;
                            }
                        });
                        ui.add_space(2.0);
                        {
                            //let config = self.config.lock().unwrap();
                            let actions = [
                                ("Reset Stopwatch", BindingAction::Reset, &config.reset_str),
                                ("Tap Speed", BindingAction::Tap, &config.tap_str),
                                (
                                    "Cycle Speed",
                                    BindingAction::CycleMultipliers,
                                    &config.cycle_multiplier_str,
                                ),
                                (
                                    "Blood Moon",
                                    BindingAction::BloodMoon,
                                    &config.blood_moon_str,
                                ),
                            ];

                            for (label, action, current_key) in actions {
                                ui.horizontal(|ui| {
                                    ui.add_sized([120.0, 20.0], egui::Label::new(label));

                                    let button_text = if self.binding_state == Some(action) {
                                        "Press any key...".to_string()
                                    } else {
                                        format!("[ {} ]", current_key)
                                    };

                                    if ui.button(button_text).clicked() {
                                        self.binding_state = Some(action);
                                    }
                                });
                            }
                        }
                    });
            }
            ui.add_space(5.0);
            let last_secs_str = format!(
                "last: {}.{:02}",
                self.stopwatch.last_seconds, self.stopwatch.last_milliseconds
            );

            ui.label(
                egui::RichText::new(last_secs_str)
                    .size(20.0)
                    .strong()
                    .monospace()
                    .color(egui::Color32::LIGHT_BLUE),
            );

            ui.add_space(10.0);
            let secs_str = format!("{}.", self.stopwatch.seconds);
            let ms_str = format!("{:02}", self.stopwatch.milliseconds / 10);
            let color = if self.stopwatch.seconds >= 60 && self.stopwatch.seconds < 90 {
                egui::Color32::RED
            } else if self.stopwatch.seconds >= 90 && self.stopwatch.seconds < 180 {
                egui::Color32::CYAN
            } else {
                egui::Color32::GREEN
            };
            ui.horizontal_top(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 25.0);
                ui.label(
                    egui::RichText::new(secs_str)
                        .size(46.0)
                        .strong()
                        .monospace()
                        .color(color),
                );
                ui.label(
                    egui::RichText::new(ms_str)
                        .size(22.0)
                        .strong()
                        .monospace()
                        .color(color),
                );
            });
            ui.add_space(15.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.allocate_ui(egui::vec2(30.0, 20.0), |ui| {
                    if has_focus && ui.button("👁").clicked() {
                        config.toggle_hide_tap();
                    }
                });
                if !config.get_hide_tap() {
                    let (bpm, speed_ms) = self.bpm_tracker.calculate();
                    ui.label(
                        egui::RichText::new(format!("{:>5.1} BPM | {:>4.2} m/s", bpm, speed_ms))
                            .size(20.0)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );
                }
            });
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.pointer.primary_pressed()) && !ctx.egui_wants_pointer_input() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));

        if let Some(action) = self.binding_state {
            let mut captured_key = None;

            // Look through egui's event queue for a physical key press
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
                        captured_key = Some(key.clone());
                    }
                }
            });

            // If a key was pressed, save it, write the JSON, and exit binding mode
            if let Some(key) = captured_key {
                let key_name = format!("{:?}", key); // Converts egui::Key::Q to "Q"
                let mut config = self.config.lock().unwrap();
                match action {
                    BindingAction::Reset => config.reset_str = key_name,
                    BindingAction::Tap => config.tap_str = key_name,
                    BindingAction::CycleMultipliers => config.cycle_multiplier_str = key_name,
                    BindingAction::BloodMoon => config.blood_moon_str = key_name,
                }

                config.save_config();
                self.binding_state = None; // Reset state
            }
        } else {
            while let Ok(key) = self.rx.try_recv() {
                //println!("keycode: {:#?}",&key);
                match key {
                    AppKey::Reset => {
                        self.stopwatch.reset();
                    }
                    AppKey::Tap => {
                        self.bpm_tracker.tap();
                    }
                    AppKey::Mult => {
                        self.bpm_tracker.cycle_multiplier();
                    }
                    AppKey::Bm => {
                        self.bpm_tracker.toggle_blood_moon();
                    }
                }
            }
        }
        self.stopwatch.tick();
        self.bpm_tracker.tick();
        ctx.request_repaint_after(Duration::from_millis(50));
    }
    //pub fn parse_key(key: Key) -> u16 {}
}
