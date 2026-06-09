#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod bpm;
mod config;
mod gui;
mod keyscan;
mod stopwatch;

use crate::config::Config;
use crate::keyscan::start_input_thread;
use anyhow::anyhow;
use gpui::*;
use gpui::{
    App, Application, AssetSource, AsyncApp, Bounds, SharedString, WindowBounds, WindowOptions, px,
    size,
};
use gpui_component::{IconName, Root, v_flex};
//use gpui_component_assets::Assets;
use gui::StopwatchApp;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::sync::{Arc, Mutex, mpsc};

/// An asset source that loads assets from the `./assets` folder.
#[derive(RustEmbed)]
#[folder = "./assets"]
//#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.ttf"]
pub struct Assets;

#[derive(Debug)]
pub enum AppKey {
    Reset,
    Tap,
    Mult,
    Bm,
}
type ConfigShared = Arc<Mutex<Config>>;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}
struct CompositeAssetSource<A1, A2> {
    primary: A1,
    secondary: A2,
}
impl<A1, A2> AssetSource for CompositeAssetSource<A1, A2>
where
    A1: AssetSource,
    A2: AssetSource,
{
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        //println!("load combined assets: {:#?}", path);

        if let Ok(p) = self.primary.load(path) {
            if let Some(data) = p {
                return Ok(Some(data));
            } else {
                return self.secondary.load(path);
            }
        } else {
            return self.secondary.load(path);
        }
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut o = vec![];
        let mut s = self.secondary.list(path);
        let mut p = self.primary.list(path);
        if let Ok(l1) = s.as_mut() {
            o.append(l1);
            if let Ok(l2) = p.as_mut() {
                o.append(l2);
            }
        }
        println!("assets: {:#?}", o);
        Ok(o)
    }
}

/*pub struct FontAssets;

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
}*/

fn main() {
    let (tx, rx) = mpsc::channel::<AppKey>();
    let config: ConfigShared = Arc::new(Mutex::new(Config::load_config()));
    start_input_thread(tx, config.clone());

    let icon_data = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();

    let gpui_assets = gpui_component_assets::Assets;
    let combined_assets = CompositeAssetSource {
        primary: Assets,
        secondary: gpui_assets,
    };
    //let app = Application::new().with_assets(FontAssets);
    let app = Application::new().with_assets(combined_assets);
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

        // Force dark theme
        use gpui_component::theme::{Theme, ThemeMode};
        Theme::change(ThemeMode::Dark, None, cx);

        let (w, h, window_x, window_y) = {
            let config = config.lock().unwrap();
            (
                crate::config::WINDOW_WIDTH * config.size,
                crate::config::WINDOW_HEIGHT * config.size,
                config.window_x,
                config.window_y,
            )
        };

        let bounds = if let (Some(x), Some(y)) = (window_x, window_y) {
            Bounds::new(gpui::Point { x: px(x), y: px(y) }, size(px(w), px(h)))
        } else {
            Bounds::centered(None, size(px(w), px(h)), cx)
        };
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            titlebar: None,
            is_resizable: true,
            ..Default::default()
        };
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                cx.open_window(window_options, |window, cx| {
                    let app = StopwatchApp::view(window, cx, rx, config.clone());
                    cx.new(|cx| gpui_component::Root::new(app, window, cx))
                })?;
                Ok::<_, anyhow::Error>(())
            }
        })
        .detach();
    });
}
