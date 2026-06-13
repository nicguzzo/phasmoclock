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
    size_input: Entity<gpui_component::input::InputState>,
    opacity_input: Entity<gpui_component::input::InputState>,
    font_select:
        Entity<gpui_component::select::SelectState<gpui_component::select::SearchableVec<String>>>,
    keyboard_select: Option<
        Entity<gpui_component::select::SelectState<gpui_component::select::SearchableVec<String>>>,
    >,
    settings_focus: FocusHandle,
    last_bounds: Option<gpui::Bounds<Pixels>>,
    _subscriptions: Vec<Subscription>,
}

impl StopwatchApp {
    pub fn view(
        _window: &mut Window,
        cx: &mut App,
        rx: mpsc::Receiver<AppKey>,
        config: ConfigShared,
    ) -> Entity<StopwatchApp> {
        cx.new(|cx| StopwatchApp::new(_window, cx, rx, config))
    }
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        rx: mpsc::Receiver<AppKey>,
        config: ConfigShared,
    ) -> Self {
        let stopwatch = cx.new(|_| Stopwatch::new());
        let bpm_tracker = cx.new(|_| BpmTracker::new());

        #[cfg(target_os = "windows")]
        crate::gui::make_window_always_on_top();

        let mut _subscriptions = Vec::new();
        _subscriptions.push(cx.observe(&stopwatch, |_, _, cx| cx.notify()));
        _subscriptions.push(cx.observe(&bpm_tracker, |_, _, cx| cx.notify()));

        let window_handle = window.window_handle();
        let size_input = cx.new(|cx| {
            let mut state = gpui_component::input::InputState::new(window, cx);
            let size = config.lock().unwrap().size;
            state.set_value(format!("{:.2}", size), window, cx);
            state
        });

        let config_clone = config.clone();
        _subscriptions.push(cx.subscribe(
            &size_input,
            move |this, size_input, event: &gpui_component::input::NumberInputEvent, cx| {
                if let gpui_component::input::NumberInputEvent::Step(action) = event {
                    let mut config = config_clone.lock().unwrap();
                    match action {
                        gpui_component::input::StepAction::Increment => {
                            config.size = (config.size + 0.1).min(3.0);
                        }
                        gpui_component::input::StepAction::Decrement => {
                            config.size = (config.size - 0.1).max(0.5);
                        }
                    }
                    config.save_config();
                    let new_size = config.size;

                    let show_settings = this.show_settings;
                    let size_input = size_input.clone();
                    cx.defer(move |cx| {
                        window_handle
                            .update(cx, move |_, window, cx| {
                                let w = if show_settings {
                                    crate::config::WINDOW_WIDTH * 2.0
                                } else {
                                    crate::config::WINDOW_WIDTH
                                };
                                let h = if show_settings {
                                    crate::config::WINDOW_HEIGHT_EXTENDED
                                        .max(crate::config::WINDOW_HEIGHT * new_size)
                                } else {
                                    crate::config::WINDOW_HEIGHT * new_size
                                };
                                window.resize(gpui::size(px(w * new_size), px(h)));
                                size_input.update(cx, |input, cx| {
                                    input.set_value(format!("{:.2}", new_size), window, cx);
                                });
                            })
                            .ok();
                    });
                    cx.notify();
                }
            },
        ));

        let config_clone2 = config.clone();
        _subscriptions.push(cx.subscribe(
            &size_input,
            move |this, size_input, event: &gpui_component::input::InputEvent, cx| {
                if let gpui_component::input::InputEvent::Change = event {
                    let text = size_input.read(cx).value().to_string();
                    if let Ok(val) = text.parse::<f32>() {
                        let mut config = config_clone2.lock().unwrap();
                        let clamped = val.clamp(0.5, 3.0);
                        config.size = clamped;
                        config.save_config();
                        let show_settings = this.show_settings;
                        cx.defer(move |cx| {
                            window_handle
                                .update(cx, move |_, window, _cx| {
                                    let w = if show_settings {
                                        crate::config::WINDOW_WIDTH
                                            + crate::config::WINDOW_WIDTH_EXTENDED
                                    } else {
                                        crate::config::WINDOW_WIDTH
                                    };
                                    let h = if show_settings {
                                        crate::config::WINDOW_HEIGHT_EXTENDED
                                            .max(crate::config::WINDOW_HEIGHT * clamped)
                                    } else {
                                        crate::config::WINDOW_HEIGHT * clamped
                                    };
                                    window.resize(gpui::size(px(w * clamped), px(h)));
                                })
                                .ok();
                        });
                    }
                    cx.notify();
                }
            },
        ));

        let opacity_input = cx.new(|cx| {
            let mut state = gpui_component::input::InputState::new(window, cx);
            let opacity = config.lock().unwrap().opacity;
            state.set_value(format!("{:.2}", opacity), window, cx);
            state
        });

        let config_clone_op = config.clone();
        let window_handle_op = window_handle.clone();
        _subscriptions.push(cx.subscribe(
            &opacity_input,
            move |_this, opacity_input, event: &gpui_component::input::NumberInputEvent, cx| {
                if let gpui_component::input::NumberInputEvent::Step(action) = event {
                    let mut config = config_clone_op.lock().unwrap();
                    match action {
                        gpui_component::input::StepAction::Increment => {
                            config.opacity = (config.opacity + 0.1).min(1.0);
                        }
                        gpui_component::input::StepAction::Decrement => {
                            config.opacity = (config.opacity - 0.1).max(0.0);
                        }
                    }
                    config.save_config();
                    let new_opacity = config.opacity;
                    let opacity_input = opacity_input.clone();
                    cx.defer(move |cx| {
                        window_handle_op
                            .update(cx, move |_, window, cx| {
                                opacity_input.update(cx, |input, cx| {
                                    input.set_value(format!("{:.2}", new_opacity), window, cx);
                                });
                            })
                            .ok();
                    });
                    cx.notify();
                }
            },
        ));

        let config_clone_op2 = config.clone();
        _subscriptions.push(cx.subscribe(
            &opacity_input,
            move |_this, opacity_input, event: &gpui_component::input::InputEvent, cx| {
                if let gpui_component::input::InputEvent::Change = event {
                    let text = opacity_input.read(cx).value().to_string();
                    if let Ok(val) = text.parse::<f32>() {
                        let mut config = config_clone_op2.lock().unwrap();
                        let clamped = val.clamp(0.0, 1.0);
                        config.opacity = clamped;
                        config.save_config();
                    }
                    cx.notify();
                }
            },
        ));

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

        let mut fonts = cx.text_system().all_font_names();
        fonts.sort();
        let default_font = "Digital-7 Mono".to_string();
        if let Some(pos) = fonts.iter().position(|f| f == &default_font) {
            fonts.remove(pos);
        }
        fonts.insert(0, default_font.clone());
        let fonts_vec = gpui_component::select::SearchableVec::new(fonts.clone());

        let selected_font = config.lock().unwrap().font.clone();
        let selected_index = fonts
            .iter()
            .position(|f| f == &selected_font)
            .map(|row| gpui_component::IndexPath::default().row(row));

        let font_select = cx.new(|cx| {
            gpui_component::select::SelectState::new(fonts_vec, selected_index, window, cx)
                .searchable(true)
        });

        let config_clone3 = config.clone();
        _subscriptions.push(cx.subscribe(
            &font_select,
            move |_this,
                  _select,
                  event: &gpui_component::select::SelectEvent<
                gpui_component::select::SearchableVec<String>,
            >,
                  cx| {
                if let gpui_component::select::SelectEvent::Confirm(Some(value)) = event {
                    let mut config = config_clone3.lock().unwrap();
                    config.font = value.clone();
                    config.save_config();
                    cx.notify();
                }
            },
        ));

        #[cfg(target_os = "linux")]
        let keyboard_select = Some(cx.new(|cx| {
            let mut devices = vec!["Auto-detect".to_string()];
            let current_device = config.lock().unwrap().keyboard_device.clone();

            use evdev::{EventType, enumerate};
            for (path, device) in enumerate() {
                if device.supported_events().contains(EventType::KEY) {
                    let path_str = path.to_string_lossy().to_string();
                    let name = device.name().unwrap_or("Unknown").to_string();
                    let label = format!("{} ({})", name, path_str);
                    devices.push(label);
                }
            }

            let selected_index = devices
                .iter()
                .position(|f| {
                    if current_device.is_empty() {
                        f == "Auto-detect"
                    } else {
                        f.ends_with(&format!("({})", current_device))
                    }
                })
                .map(|row| gpui_component::IndexPath::default().row(row));

            let devices_vec = gpui_component::select::SearchableVec::new(devices);
            gpui_component::select::SelectState::new(devices_vec, selected_index, window, cx)
                .searchable(true)
        }));

        #[cfg(not(target_os = "linux"))]
        let keyboard_select = None;

        #[cfg(target_os = "linux")]
        if let Some(keyboard_select) = &keyboard_select {
            let config_clone_kbd = config.clone();
            _subscriptions.push(cx.subscribe(
                keyboard_select,
                move |_this,
                      _select,
                      event: &gpui_component::select::SelectEvent<
                    gpui_component::select::SearchableVec<String>,
                >,
                      cx| {
                    if let gpui_component::select::SelectEvent::Confirm(Some(value)) = event {
                        let mut config = config_clone_kbd.lock().unwrap();
                        if value == "Auto-detect" {
                            config.keyboard_device = "".to_string();
                        } else if let Some(start) = value.rfind('(') {
                            if let Some(end) = value.rfind(')') {
                                let path = &value[start + 1..end];
                                config.keyboard_device = path.to_string();
                            }
                        }
                        config.save_config();
                        cx.notify();
                    }
                },
            ));
        }

        let settings_focus = cx.focus_handle();

        Self {
            stopwatch,
            bpm_tracker,
            binding_state: None,
            show_settings: false,
            config,
            size_input,
            opacity_input,
            font_select,
            keyboard_select,
            settings_focus,
            last_bounds: None,
            _subscriptions,
        }
    }
}

impl Render for StopwatchApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_bounds = window.bounds();
        let mut bounds_changed = false;
        if let Some(last) = self.last_bounds {
            if last.origin.x != current_bounds.origin.x || last.origin.y != current_bounds.origin.y
            {
                bounds_changed = true;
            }
        }
        self.last_bounds = Some(current_bounds);

        if bounds_changed {
            let mut config = self.config.lock().unwrap();
            config.window_x = Some(current_bounds.origin.x.into());
            config.window_y = Some(current_bounds.origin.y.into());
            config.save_config();
        }

        let bpm_tracker_entity = self.bpm_tracker.clone();
        let bpm_tracker_entity2 = self.bpm_tracker.clone();
        let stopwatch = self.stopwatch.read(cx);
        let bpm_tracker = self.bpm_tracker.read(cx);
        let blood_moon_color = if bpm_tracker.is_blood_mode() {
            rgb(0xff0000)
        } else {
            rgb(0x666666)
        };
        let (size, font_family_name, opacity) = {
            let config = self.config.lock().unwrap();
            (config.size, config.font.clone(), config.opacity)
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

        let color = if stopwatch.seconds >= 60 && stopwatch.seconds < 90 {
            rgb(0xff0000)
        } else if stopwatch.seconds >= 90 && stopwatch.seconds < 180 {
            rgb(0x00ffff)
        } else {
            rgb(0x00ff00)
        };

        div()
            .size_full()
            .w_full()
            .h_full()
            .flex()
            .flex_row()
            .m_0()
            .p_0()
            .child(
                div()
                    .w(px(crate::config::WINDOW_WIDTH * size))
                    .h_full()
                    .flex()
                    .flex_col()
                    .bg(gpui::black().alpha(opacity))
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
                                    .w(px(128.0 * size))
                                    .h(px(32.0 * size))
                                    .child(
                                        div()
                                            .font_family(font_family_name.clone())
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
                                    .size(px(32.0 * size))
                                    .label("⚙")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.show_settings = !this.show_settings;
                                        if !this.show_settings {
                                            this.binding_state = None;
                                            let config = this.config.lock().unwrap();
                                            _window.resize(gpui::size(
                                                px(crate::config::WINDOW_WIDTH * config.size),
                                                px(crate::config::WINDOW_HEIGHT * config.size),
                                            ));
                                        } else {
                                            this.settings_focus.focus(_window);
                                            let config = this.config.lock().unwrap();
                                            let h = crate::config::WINDOW_HEIGHT_EXTENDED
                                                .max(crate::config::WINDOW_HEIGHT * config.size);
                                            _window.resize(gpui::size(
                                                px((crate::config::WINDOW_WIDTH * config.size
                                                    + crate::config::WINDOW_WIDTH_EXTENDED)),
                                                px(h),
                                            ));
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(div().w_full().when(!self.show_settings, |this| {
                                this.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    |_event, window, _app| {
                                        //println!("on_mouse_down");
                                        #[cfg(target_os = "linux")]
                                        window.start_window_move();
                                        #[cfg(target_os = "windows")]
                                        crate::gui::start_window_drag_native();
                                        #[cfg(not(target_os = "windows"))]
                                        window.start_window_move();
                                    },
                                )
                            }))
                            .child(
                                Button::new("close")
                                    .size(px(32.0 * size))
                                    .label("🗙")
                                    .on_click(|_event, _window, app| {
                                        app.quit();
                                    }),
                            ),
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
                            .font_family(font_family_name.clone())
                            .child(
                                div()
                                    .m_0()
                                    .p_0()
                                    .pt(px(3.0 * size))
                                    .line_height(gpui::relative(0.8))
                                    .text_size(px(40.0 * size))
                                    .text_color(rgb(0x00ffff))
                                    .child(last_secs_str),
                            )
                            .child(
                                div()
                                    .m_0()
                                    .p_0()
                                    .pt(px(3.0 * size))
                                    .line_height(gpui::relative(0.8))
                                    .text_size(px(100.0 * size))
                                    .text_color(color)
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
                            .when(!self.show_settings, |this| {
                                this.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    |_event, window, _app| {
                                        #[cfg(target_os = "linux")]
                                        window.start_window_move();
                                        #[cfg(target_os = "windows")]
                                        crate::gui::start_window_drag_native();
                                        #[cfg(not(target_os = "windows"))]
                                        window.start_window_move();
                                    },
                                )
                            }),
                    ),
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

                let size_controls = div()
                    .flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(div().text_color(rgb(0xffffff)).child("Window Size"))
                    .child(
                        div()
                            .w(px(140.))
                            .child(gpui_component::input::NumberInput::new(&self.size_input)),
                    );

                let font_controls = div()
                    .flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(div().text_color(rgb(0xffffff)).child("Font"))
                    .child(
                        div()
                            .w(px(140.))
                            .child(gpui_component::select::Select::new(&self.font_select)),
                    );

                let opacity_controls = div()
                    .flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(div().text_color(rgb(0xffffff)).child("Opacity"))
                    .child(
                        div()
                            .w(px(140.))
                            .child(gpui_component::input::NumberInput::new(&self.opacity_input)),
                    );

                actions_div = actions_div
                    .child(size_controls)
                    .child(opacity_controls)
                    .child(font_controls);

                #[cfg(target_os = "linux")]
                if let Some(keyboard_select) = &self.keyboard_select {
                    let keyboard_controls = div()
                        .flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_color(rgb(0xffffff))
                                .child("Keyboard (Requires Restart)"),
                        )
                        .child(
                            div()
                                .w(px(400.))
                                .child(gpui_component::select::Select::new(keyboard_select)),
                        );
                    actions_div = actions_div.child(div().h(px(10.))).child(keyboard_controls);
                }

                Some(
                    div()
                        .w(px(crate::config::WINDOW_WIDTH
                            + crate::config::WINDOW_WIDTH_EXTENDED * size))
                        .h_full()
                        .bg(rgb(0x222222))
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .track_focus(&self.settings_focus)
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                            if let Some(action) = this.binding_state {
                                let key_name = event.keystroke.key.clone();
                                let mut config = this.config.lock().unwrap();
                                match action {
                                    BindingAction::Reset => config.reset_str = key_name,
                                    BindingAction::Tap => config.tap_str = key_name,
                                    BindingAction::CycleMultipliers => {
                                        config.cycle_multiplier_str = key_name
                                    }
                                    BindingAction::BloodMoon => config.blood_moon_str = key_name,
                                }
                                config.save_config();
                                this.binding_state = None;
                                cx.notify();
                            }
                        }))
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
                                        let config = this.config.lock().unwrap();
                                        _w.resize(gpui::size(
                                            px(crate::config::WINDOW_WIDTH * config.size),
                                            px(crate::config::WINDOW_HEIGHT * config.size),
                                        ));
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
                )
            } else {
                None
            })
    }

    //pub fn parse_key(key: Key) -> u16 {}
}

#[cfg(target_os = "windows")]
pub fn make_window_always_on_top() {
    std::thread::spawn(|| {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::System::Threading::GetCurrentProcessId;
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, HWND_TOPMOST, IsWindowVisible, SWP_NOMOVE,
            SWP_NOSIZE, SetWindowPos,
        };

        unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let mut process_id = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut _));
                if process_id == GetCurrentProcessId() && IsWindowVisible(hwnd).as_bool() {
                    let out_hwnd = &mut *(lparam.0 as *mut Option<HWND>);
                    *out_hwnd = Some(hwnd);
                    return BOOL(0);
                }
                BOOL(1)
            }
        }

        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let mut found_hwnd: Option<HWND> = None;
            unsafe {
                let _ = EnumWindows(
                    Some(enum_window),
                    LPARAM(&mut found_hwnd as *mut _ as isize),
                );
                if let Some(hwnd) = found_hwnd {
                    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                    break;
                }
            }
        }
    });
}

#[cfg(target_os = "windows")]
pub fn start_window_drag_native() {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, HTCAPTION, IsWindowVisible, PostMessageW,
        WM_NCLBUTTONDOWN,
    };

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let mut process_id = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut _));
            if process_id == GetCurrentProcessId() && IsWindowVisible(hwnd).as_bool() {
                let out_hwnd = &mut *(lparam.0 as *mut Option<HWND>);
                *out_hwnd = Some(hwnd);
                return BOOL(0);
            }
            BOOL(1)
        }
    }

    let mut found_hwnd: Option<HWND> = None;
    unsafe {
        let _ = EnumWindows(
            Some(enum_window),
            LPARAM(&mut found_hwnd as *mut _ as isize),
        );
        if let Some(hwnd) = found_hwnd {
            let _ = ReleaseCapture();
            let _ = PostMessageW(
                hwnd,
                WM_NCLBUTTONDOWN,
                WPARAM(HTCAPTION as usize),
                LPARAM(0),
            );
        }
    }
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
