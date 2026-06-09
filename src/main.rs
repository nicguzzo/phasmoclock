#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod bpm;
mod config;
mod gui;
mod keyscan;
mod stopwatch;

use gpui::{App, Application, AssetSource, AsyncApp, SharedString, WindowOptions};

use crate::config::Config;
use crate::keyscan::start_input_thread;
use gui::StopwatchApp;
use std::borrow::Cow;
use std::sync::{Arc, Mutex, mpsc};

#[derive(Debug)]
pub enum AppKey {
    Reset,
    Tap,
    Mult,
    Bm,
}
type ConfigShared = Arc<Mutex<Config>>;

pub struct FontAssets;

impl AssetSource for FontAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match path {
            "fonts/digital-7_mono.ttf" => {
                let bytes = include_bytes!("../assets/fonts/digital-7_mono.ttf");
                Ok(Some(Cow::Borrowed(bytes)))
            }
            _ => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec!["fonts/digital-7_mono.ttf".into()])
    }
}

fn main() {
    let (tx, rx) = mpsc::channel::<AppKey>();
    let config: ConfigShared = Arc::new(Mutex::new(Config::load_config()));
    start_input_thread(tx, config.clone());

    let icon_data = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();

    let app = Application::new().with_assets(FontAssets);
    app.run(move |cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![
                cx.asset_source()
                    .load("fonts/digital-7_mono.ttf")
                    .unwrap()
                    .unwrap(),
            ])
            .expect("Failed to load custom font");
        gpui_component::init(cx);

        //let bounds = Bounds::centered(None, size(px(210.0), px(125.0)), cx);
        let window_options = WindowOptions {
            //window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                cx.open_window(window_options, |window, cx| {
                    StopwatchApp::view(window, cx, rx, config.clone())
                })?;
                Ok::<_, anyhow::Error>(())
            }
        })
        .detach();
    });
}
