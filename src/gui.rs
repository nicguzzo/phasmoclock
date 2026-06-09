use crate::bpm::BpmTracker;
use crate::stopwatch::Stopwatch;
use crate::{AppKey, ConfigShared};

use gpui::colors::Colors;
use gpui::{
    AbsoluteLength, App, AsyncApp, Context, Div, DragMoveEvent, Entity, FocusHandle, KeyDownEvent,
    Pixels, Render, Subscription, WeakEntity, Window, div, prelude::*, px, rgb, rgba, size,
};
use gpui_component::ColorName::Red;
use gpui_component::{button::*, *};
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
    stopwatch: Entity<Stopwatch>,
    bpm_tracker: Entity<BpmTracker>,
    binding_state: Option<BindingAction>,
    show_settings: bool,
    config: ConfigShared,
    settings_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl StopwatchApp {
    pub fn view(
        _window: &mut Window,
        cx: &mut App,
        rx: mpsc::Receiver<AppKey>,
        config: ConfigShared,
    ) -> Entity<StopwatchApp> {
        cx.new(|cx| StopwatchApp::new(cx, rx, config))
    }
    pub fn new(cx: &mut Context<Self>, rx: mpsc::Receiver<AppKey>, config: ConfigShared) -> Self {
        let stopwatch = cx.new(|_| Stopwatch::new());
        let bpm_tracker = cx.new(|_| BpmTracker::new());

        let mut _subscriptions = Vec::new();
        _subscriptions.push(cx.observe(&stopwatch, |_, _, cx| cx.notify()));
        _subscriptions.push(cx.observe(&bpm_tracker, |_, _, cx| cx.notify()));

        let stopwatch_clone = stopwatch.clone();
        let bpm_tracker_clone = bpm_tracker.clone();

        cx.spawn(|this: WeakEntity<StopwatchApp>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    let _ = cx
                        .background_executor()
                        .timer(Duration::from_millis(50))
                        .await;

                    let result = this.update(&mut cx, |_, cx| {
                        stopwatch_clone.update(cx, |stopwatch: &mut Stopwatch, cx| {
                            while let Ok(key) = rx.try_recv() {
                                match key {
                                    AppKey::Reset => {
                                        stopwatch.reset();
                                    }
                                    AppKey::Tap => {
                                        bpm_tracker_clone.update(
                                            cx,
                                            |bpm_tracker: &mut BpmTracker, cx| {
                                                bpm_tracker.tap();
                                                cx.notify();
                                            },
                                        );
                                    }
                                    AppKey::Mult => {
                                        bpm_tracker_clone.update(
                                            cx,
                                            |bpm_tracker: &mut BpmTracker, cx| {
                                                bpm_tracker.cycle_multiplier();
                                                cx.notify();
                                            },
                                        );
                                    }
                                    AppKey::Bm => {
                                        bpm_tracker_clone.update(
                                            cx,
                                            |bpm_tracker: &mut BpmTracker, cx| {
                                                bpm_tracker.toggle_blood_moon();
                                                cx.notify();
                                            },
                                        );
                                    }
                                }
                            }
                            stopwatch.tick();
                            cx.notify();
                        });

                        bpm_tracker_clone.update(cx, |bpm_tracker: &mut BpmTracker, cx| {
                            bpm_tracker.tick();
                            cx.notify();
                        });
                    });

                    if result.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        let settings_focus = cx.focus_handle();

        Self {
            stopwatch,
            bpm_tracker,
            binding_state: None,
            show_settings: false,
            config,
            settings_focus,
            _subscriptions,
        }
    }
}

fn gpui_key_to_str(k: &str) -> String {
    match k {
        "space" => "Space".to_string(),
        "escape" => "Escape".to_string(),
        "enter" => "Enter".to_string(),
        "backspace" => "Backspace".to_string(),
        "tab" => "Tab".to_string(),
        "up" => "ArrowUp".to_string(),
        "down" => "ArrowDown".to_string(),
        "left" => "ArrowLeft".to_string(),
        "right" => "ArrowRight".to_string(),
        _ => {
            if k.len() == 1 {
                k.to_uppercase()
            } else {
                let mut c = k.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        }
    }
}

impl Render for StopwatchApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bpm_tracker_entity = self.bpm_tracker.clone();
        let bpm_tracker_entity2 = self.bpm_tracker.clone();
        let stopwatch = self.stopwatch.read(cx);
        let bpm_tracker = self.bpm_tracker.read(cx);
        let blood_moon_color = if bpm_tracker.is_blood_mode() {
            rgb(0xff0000)
        } else {
            rgb(0x666666)
        };
        let size = {
            let config = self.config.lock().unwrap();
            config.size
        };
        let speed_multiplier = format!("{:03}%", bpm_tracker.get_speed_multiplier());

        let secs_str = format!(
            "{}.{:02}",
            stopwatch.seconds as u64,
            stopwatch.milliseconds / 10
        );
        let last_secs_str = format!(
            "last: {}.{:02}",
            stopwatch.last_seconds,
            stopwatch.last_milliseconds / 10
        );
        let bpm_str = format!("Speed {:>4.2} m/s", bpm_tracker.speed_ms);

        let btn1 = ButtonCustomVariant::new(cx)
            .color(rgb(0x111111).into())
            .foreground(rgb(0xffffff).into())
            .border(rgb(0x0).into())
            .shadow(false)
            .hover(rgb(0x222222).into())
            .active(rgb(0xffffff).into());

        div()
            .size_full()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .m_0()
            .p_0()
            //.bg(rgb(0xff0000))
            //.bg(rgba(0x00000000))
            //.justify_center()
            .child(
                div()
                    .p_2()
                    .flex()
                    .w_full()
                    .justify_end()
                    .gap_1()
                    .child(
                        Button::new("speed_multiplier")
                            //.custom(btn1)
                            .primary()
                            .w(px(64.0 * size))
                            .h(px(32.0 * size))
                            .child(
                                div()
                                    .font_family("Digital-7 Mono")
                                    .text_size(px(32.0 * size))
                                    .child(speed_multiplier),
                            )
                            .on_click(move |_event, _window, cx| {
                                bpm_tracker_entity.update(cx, |bpm_tracker, cx| {
                                    bpm_tracker.cycle_multiplier();
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        Button::new("blood_moon")
                            .primary()
                            .size(px(32.0 * size))
                            .icon(Icon::new(IconName::Moon).text_color(blood_moon_color))
                            .on_click(move |_event, _window, cx| {
                                bpm_tracker_entity2.update(cx, |bpm_tracker, cx| {
                                    bpm_tracker.toggle_blood_moon();
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        Button::new("settings")
                            .primary()
                            .size(px(32.0 * size))
                            .label("⚙")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_settings = !this.show_settings;
                                if !this.show_settings {
                                    this.binding_state = None;
                                } else {
                                    this.settings_focus.focus(_window);
                                }
                                cx.notify();
                            })),
                    )
                    .child(div().w_full().on_mouse_down(
                        gpui::MouseButton::Left,
                        |_event, window, _app| {
                            //println!("on_mouse_down");
                            window.start_window_move();
                        },
                    ))
                    .child(Button::new("close").primary().label("🗙").on_click(
                        |_event, _window, app| {
                            app.quit();
                        },
                    )),
            )
            .child(
                div()
                    .m_0()
                    .p_0()
                    .gap_0()
                    //.w_full()
                    .size_full()
                    .flex()
                    .flex_col()
                    .justify_start()
                    .items_center()
                    .font_family("Digital-7 Mono")
                    .child(
                        div()
                            .m_0()
                            .p_0()
                            .pt_3()
                            .line_height(gpui::relative(0.8))
                            .text_size(px(40.0 * size))
                            .text_color(rgb(0x00ffff))
                            .child(last_secs_str),
                    )
                    .child(
                        div()
                            .m_0()
                            .p_0()
                            .pt_3()
                            .line_height(gpui::relative(0.8))
                            .text_size(px(100.0 * size))
                            .text_color(rgb(0x00ff00))
                            .child(secs_str),
                    )
                    .child(
                        div()
                            .m_0()
                            .p_0()
                            .text_size(px(40.0 * size))
                            .text_color(rgb(0xffff00))
                            .child(bpm_str),
                    )
                    .on_mouse_down(gpui::MouseButton::Left, |_event, window, _app| {
                        //println!("on_mouse_down");
                        window.start_window_move();
                    }),
            )
            .children(if self.show_settings {
                let config = self.config.lock().unwrap();
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
                let mut actions_div = div().flex().flex_col().gap_2();
                for (label_str, action, current_key) in actions {
                    let is_active = self.binding_state == Some(action);
                    let button_text = if is_active {
                        "Press any key...".to_string()
                    } else {
                        format!("[ {} ]", current_key)
                    };
                    actions_div = actions_div.child(
                        div()
                            .flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(div().text_color(rgb(0xffffff)).child(label_str))
                            .child(Button::new(label_str).label(button_text).on_click(
                                cx.listener(move |this, _e, _w, cx| {
                                    this.binding_state = Some(action);
                                    this.settings_focus.focus(_w);
                                    cx.notify();
                                }),
                            )),
                    );
                }

                Some(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(rgba(0x000000cc))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(320.))
                                .bg(rgb(0x222222))
                                .p_4()
                                .rounded_lg()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .track_focus(&self.settings_focus)
                                .on_key_down(cx.listener(
                                    |this, event: &KeyDownEvent, _window, cx| {
                                        if let Some(action) = this.binding_state {
                                            let key_name = gpui_key_to_str(&event.keystroke.key);
                                            let mut config = this.config.lock().unwrap();
                                            match action {
                                                BindingAction::Reset => config.reset_str = key_name,
                                                BindingAction::Tap => config.tap_str = key_name,
                                                BindingAction::CycleMultipliers => {
                                                    config.cycle_multiplier_str = key_name
                                                }
                                                BindingAction::BloodMoon => {
                                                    config.blood_moon_str = key_name
                                                }
                                            }
                                            config.save_config();
                                            this.binding_state = None;
                                            cx.notify();
                                        }
                                    },
                                ))
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .text_color(rgb(0xffffff))
                                                .child("Keybind Configuration"),
                                        )
                                        .child(Button::new("close_settings").label("🗙").on_click(
                                            cx.listener(|this, _e, _w, cx| {
                                                this.show_settings = false;
                                                this.binding_state = None;
                                                cx.notify();
                                            }),
                                        )),
                                )
                                .child(
                                    div()
                                        .text_color(rgb(0xaaaaaa))
                                        .text_size(px(14.))
                                        .child("Click an action to rebind it:"),
                                )
                                .child(actions_div),
                        ),
                )
            } else {
                None
            })
    }

    //pub fn parse_key(key: Key) -> u16 {}
}

/*
impl eframe::App for StopwatchApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let has_focus = ui.ctx().input(|i| i.focused);

        let mut config = self.config.lock().unwrap();
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(5.0);
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
                        } else {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                egui::vec2(210.0, 150.0),
                            ));
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
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                    egui::vec2(210.0, 125.0),
                                ));
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
                self.stopwatch.last_seconds,
                self.stopwatch.last_milliseconds / 10
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
                ui.add_space(5.0);
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
            ctx.input(|i| {
                let config = self.config.lock().unwrap();
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
                        let key = key.name();
                        if key == config.reset_str {
                            self.stopwatch.reset();
                        } else if key == config.tap_str {
                            self.bpm_tracker.tap();
                        } else if key == config.cycle_multiplier_str {
                            self.bpm_tracker.cycle_multiplier();
                        } else if key == config.blood_moon_str {
                            self.bpm_tracker.toggle_blood_moon();
                        }
                    }
                }
            });
        }
        self.stopwatch.tick();
        self.bpm_tracker.tick();
        ctx.request_repaint_after(Duration::from_millis(50));
    }
    //pub fn parse_key(key: Key) -> u16 {}
}
*/
