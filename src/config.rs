use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_reset_str")]
    pub reset_str: String,
    #[serde(default = "default_tap_str")]
    pub tap_str: String,
    #[serde(default = "default_cycle_multiplier_str")]
    pub cycle_multiplier_str: String,
    #[serde(default = "default_blood_moon_str")]
    pub blood_moon_str: String,
    #[serde(skip)]
    pub reset_code: u16,
    #[serde(skip)]
    pub tap_code: u16,
    #[serde(skip)]
    pub cycle_multiplier_code: u16,
    #[serde(skip)]
    pub blood_moon_code: u16,
    #[serde(default)]
    hide_speeds: bool,
    #[serde(default)]
    hide_tap: bool,
}

fn default_reset_str() -> String {
    "1".to_string()
}
fn default_tap_str() -> String {
    "2".to_string()
}
fn default_cycle_multiplier_str() -> String {
    "3".to_string()
}
fn default_blood_moon_str() -> String {
    "4".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let reset_code = parse_key("1");
        let tap_code = parse_key("2");
        let cycle_multiplier_code = parse_key("3");
        let blood_moon_code = parse_key("4");
        Self {
            reset_str: key_code_to_str(reset_code),
            tap_str: key_code_to_str(tap_code),
            cycle_multiplier_str: key_code_to_str(cycle_multiplier_code),
            blood_moon_str: key_code_to_str(blood_moon_code),
            reset_code,
            tap_code,
            cycle_multiplier_code,
            blood_moon_code,
            hide_speeds: false,
            hide_tap: false,
        }
    }
}

impl Config {
    pub fn get_hide_speeds(&self) -> bool {
        self.hide_speeds
    }
    pub fn get_hide_tap(&self) -> bool {
        self.hide_tap
    }
    pub fn toggle_hide_speeds(&mut self) {
        self.hide_speeds = !self.hide_speeds;
        self.save_config();
    }
    pub fn toggle_hide_tap(&mut self) {
        self.hide_tap = !self.hide_tap;
    }
    fn get_config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "phasmoclock") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            Some(config_dir.join("bindings.json"))
        } else {
            None // Fallback if the OS doesn't have a valid home directory
        }
    }
    pub fn load_config() -> Config {
        if let Some(path) = Config::get_config_path() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<Config>(&data) {
                    cfg.reset_code = parse_key(&cfg.reset_str);
                    cfg.tap_code = parse_key(&cfg.tap_str);
                    cfg.cycle_multiplier_code = parse_key(&cfg.cycle_multiplier_str);
                    cfg.blood_moon_code = parse_key(&cfg.blood_moon_str);
                    return cfg;
                }
            }
        }
        Config::default()
    }

    pub fn save_config(&mut self) {
        self.reset_code = parse_key(&self.reset_str);
        self.tap_code = parse_key(&self.tap_str);
        self.cycle_multiplier_code = parse_key(&self.cycle_multiplier_str);
        self.blood_moon_code = parse_key(&self.blood_moon_str);

        if let Some(path) = Config::get_config_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, json);
            }
        }
    }
}

// --- LINUX MAPPING (evdev) ---
#[cfg(target_os = "linux")]
pub fn key_code_to_str(key_code: u16) -> String {
    use kbd_evdev::convert::EvdevKeyCodeExt;
    let key_code = evdev::KeyCode(key_code);
    format!("{}", key_code.to_key())
}
#[cfg(target_os = "linux")]
pub fn parse_key(key_str: &str) -> u16 {
    use evdev::KeyCode;
    match key_str {
        "Q" => KeyCode::KEY_Q,
        "W" => KeyCode::KEY_W,
        "E" => KeyCode::KEY_E,
        "R" => KeyCode::KEY_R,
        "T" => KeyCode::KEY_T,
        "Y" => KeyCode::KEY_Y,
        "U" => KeyCode::KEY_U,
        "I" => KeyCode::KEY_I,
        "O" => KeyCode::KEY_O,
        "P" => KeyCode::KEY_P,
        "A" => KeyCode::KEY_A,
        "S" => KeyCode::KEY_S,
        "D" => KeyCode::KEY_D,
        "F" => KeyCode::KEY_F,
        "G" => KeyCode::KEY_G,
        "H" => KeyCode::KEY_H,
        "J" => KeyCode::KEY_J,
        "K" => KeyCode::KEY_K,
        "L" => KeyCode::KEY_L,
        "Z" => KeyCode::KEY_Z,
        "X" => KeyCode::KEY_X,
        "C" => KeyCode::KEY_C,
        "V" => KeyCode::KEY_V,
        "B" => KeyCode::KEY_B,
        "N" => KeyCode::KEY_N,
        "M" => KeyCode::KEY_M,

        "Space" => KeyCode::KEY_SPACE,
        "Escape" => KeyCode::KEY_ESC,
        "Tab" => KeyCode::KEY_TAB,
        "Enter" => KeyCode::KEY_ENTER,
        "Backspace" => KeyCode::KEY_BACKSPACE,
        "Backtick" => KeyCode::KEY_GRAVE,
        "Insert" => KeyCode::KEY_INSERT,
        "Delete" => KeyCode::KEY_DELETE,
        "Home" => KeyCode::KEY_HOME,
        "End" => KeyCode::KEY_END,
        "PageUp" => KeyCode::KEY_PAGEUP,
        "PageDown" => KeyCode::KEY_PAGEUP,
        "ArrowDown" => KeyCode::KEY_DOWN,
        "ArrowLeft" => KeyCode::KEY_LEFT,
        "ArrowRight" => KeyCode::KEY_RIGHT,
        "ArrowUp" => KeyCode::KEY_UP,
        "Comma" => KeyCode::KEY_COMMA,
        "Backslash" => KeyCode::KEY_BACKSLASH,
        "Slash" => KeyCode::KEY_SLASH,
        "Pipe" => KeyCode::KEY_P,
        "OpenBracket" => KeyCode::KEY_LEFTBRACE,
        "CloseBracket" => KeyCode::KEY_RIGHTBRACE,
        "Minus" => KeyCode::KEY_MINUS,
        "Period" => KeyCode::KEY_DOT,
        "Equals" => KeyCode::KEY_EQUAL,
        "Semicolon" => KeyCode::KEY_SEMICOLON,
        //"Quote" => KeyCode::KEY,
        "F1" => KeyCode::KEY_F1,
        "F2" => KeyCode::KEY_F2,
        "F3" => KeyCode::KEY_F3,
        "F4" => KeyCode::KEY_F4,
        "F5" => KeyCode::KEY_F5,
        "F6" => KeyCode::KEY_F6,
        "F7" => KeyCode::KEY_F7,
        "F8" => KeyCode::KEY_F8,
        "F9" => KeyCode::KEY_F9,
        "F10" => KeyCode::KEY_F10,
        "F11" => KeyCode::KEY_F11,
        "F12" => KeyCode::KEY_F12,
        // Numbers
        "Num0" => KeyCode::KEY_0,
        "Num1" => KeyCode::KEY_1,
        "Num2" => KeyCode::KEY_2,
        "Num3" => KeyCode::KEY_3,
        "Num4" => KeyCode::KEY_4,
        "Num5" => KeyCode::KEY_5,
        "Num6" => KeyCode::KEY_6,
        "Num7" => KeyCode::KEY_7,
        "Num8" => KeyCode::KEY_8,
        "Num9" => KeyCode::KEY_9,

        // Fallback for unmapped keys to prevent crashes
        _ => KeyCode::KEY_UNKNOWN,
    }
    .code()
}

// --- WINDOWS MAPPING (rdev) ---
//

#[cfg(target_os = "windows")]
pub fn rdev_to_win_vk(key: rdev::Key) -> u16 {
    use rdev::Key;

    match key {
        // --- Letters (VK_A through VK_Z are 0x41 - 0x5A) ---
        Key::KeyA => 0x41,
        Key::KeyB => 0x42,
        Key::KeyC => 0x43,
        Key::KeyD => 0x44,
        Key::KeyE => 0x45,
        Key::KeyF => 0x46,
        Key::KeyG => 0x47,
        Key::KeyH => 0x48,
        Key::KeyI => 0x49,
        Key::KeyJ => 0x4A,
        Key::KeyK => 0x4B,
        Key::KeyL => 0x4C,
        Key::KeyM => 0x4D,
        Key::KeyN => 0x4E,
        Key::KeyO => 0x4F,
        Key::KeyP => 0x50,
        Key::KeyQ => 0x51,
        Key::KeyR => 0x52,
        Key::KeyS => 0x53,
        Key::KeyT => 0x54,
        Key::KeyU => 0x55,
        Key::KeyV => 0x56,
        Key::KeyW => 0x57,
        Key::KeyX => 0x58,
        Key::KeyY => 0x59,
        Key::KeyZ => 0x5A,

        // --- Numbers (VK_0 through VK_9 are 0x30 - 0x39) ---
        Key::Num0 => 0x30,
        Key::Num1 => 0x31,
        Key::Num2 => 0x32,
        Key::Num3 => 0x33,
        Key::Num4 => 0x34,
        Key::Num5 => 0x35,
        Key::Num6 => 0x36,
        Key::Num7 => 0x37,
        Key::Num8 => 0x38,
        Key::Num9 => 0x39,

        // --- Special Keys ---
        Key::Space => 0x20,     // VK_SPACE
        Key::Escape => 0x1B,    // VK_ESCAPE
        Key::Return => 0x0D,    // VK_RETURN (Enter)
        Key::Backspace => 0x08, // VK_BACK
        Key::Tab => 0x09,       // VK_TAB
        Key::BackQuote => 0xC0, // VK_OEM_3 (The ` or ~ key)

        // Fallback for unmapped keys (0x00 means unassigned)
        _ => 0x00,
    }
}
#[cfg(target_os = "windows")]
pub fn parse_key(key_str: &str) -> u16 {
    use rdev::Key;
    match key_str {
        "Q" => rdev_to_win_vk(Key::KeyQ),
        "W" => rdev_to_win_vk(Key::KeyW),
        "E" => rdev_to_win_vk(Key::KeyE),
        "R" => rdev_to_win_vk(Key::KeyR),
        "T" => rdev_to_win_vk(Key::KeyT),
        "Y" => rdev_to_win_vk(Key::KeyY),
        "U" => rdev_to_win_vk(Key::KeyU),
        "I" => rdev_to_win_vk(Key::KeyI),
        "O" => rdev_to_win_vk(Key::KeyO),
        "P" => rdev_to_win_vk(Key::KeyP),
        "A" => rdev_to_win_vk(Key::KeyA),
        "S" => rdev_to_win_vk(Key::KeyS),
        "D" => rdev_to_win_vk(Key::KeyD),
        "F" => rdev_to_win_vk(Key::KeyF),
        "G" => rdev_to_win_vk(Key::KeyG),
        "H" => rdev_to_win_vk(Key::KeyH),
        "J" => rdev_to_win_vk(Key::KeyJ),
        "K" => rdev_to_win_vk(Key::KeyK),
        "L" => rdev_to_win_vk(Key::KeyL),
        "Z" => rdev_to_win_vk(Key::KeyZ),
        "X" => rdev_to_win_vk(Key::KeyX),
        "C" => rdev_to_win_vk(Key::KeyC),
        "V" => rdev_to_win_vk(Key::KeyV),
        "B" => rdev_to_win_vk(Key::KeyB),
        "N" => rdev_to_win_vk(Key::KeyN),
        "M" => rdev_to_win_vk(Key::KeyM),

        // Special Keys
        "Space" => rdev_to_win_vk(Key::Space),
        "Escape" => rdev_to_win_vk(Key::Escape),
        "Enter" => rdev_to_win_vk(Key::Return),
        "Backspace" => rdev_to_win_vk(Key::Backspace),
        "Backtick" => rdev_to_win_vk(Key::BackQuote),
        "Tab" => rdev_to_win_vk(Key::Tab),
        "F1" => rdev_to_win_vk(Key::F1),
        "F2" => rdev_to_win_vk(Key::F2),
        "F3" => rdev_to_win_vk(Key::F3),
        "F4" => rdev_to_win_vk(Key::F4),
        "F5" => rdev_to_win_vk(Key::F5),
        "F6" => rdev_to_win_vk(Key::F6),
        "F7" => rdev_to_win_vk(Key::F7),
        "F8" => rdev_to_win_vk(Key::F8),
        "F9" => rdev_to_win_vk(Key::F9),
        "F10" => rdev_to_win_vk(Key::F10),
        "F11" => rdev_to_win_vk(Key::F11),
        "F12" => rdev_to_win_vk(Key::F12),

        // Numbers
        "Num0" => rdev_to_win_vk(Key::Num0),
        "Num1" => rdev_to_win_vk(Key::Num1),
        "Num2" => rdev_to_win_vk(Key::Num2),
        "Num3" => rdev_to_win_vk(Key::Num3),
        "Num4" => rdev_to_win_vk(Key::Num4),
        "Num5" => rdev_to_win_vk(Key::Num5),
        "Num6" => rdev_to_win_vk(Key::Num6),
        "Num7" => rdev_to_win_vk(Key::Num7),
        "Num8" => rdev_to_win_vk(Key::Num8),
        "Num9" => rdev_to_win_vk(Key::Num9),

        _ => rdev_to_win_vk(Key::Unknown(0)),
    }
}
#[cfg(target_os = "windows")]
pub fn key_code_to_str(vk_code: u16) -> String {
    let code = match vk_code {
        // --- Letters (0x41 - 0x5A) ---
        0x41 => "A",
        0x42 => "B",
        0x43 => "C",
        0x44 => "D",
        0x45 => "E",
        0x46 => "F",
        0x47 => "G",
        0x48 => "H",
        0x49 => "I",
        0x4A => "J",
        0x4B => "K",
        0x4C => "L",
        0x4D => "M",
        0x4E => "N",
        0x4F => "O",
        0x50 => "P",
        0x51 => "Q",
        0x52 => "R",
        0x53 => "S",
        0x54 => "T",
        0x55 => "U",
        0x56 => "V",
        0x57 => "W",
        0x58 => "X",
        0x59 => "Y",
        0x5A => "Z",

        // --- Numbers (0x30 - 0x39) ---
        0x30 => "Num0",
        0x31 => "Num1",
        0x32 => "Num2",
        0x33 => "Num3",
        0x34 => "Num4",
        0x35 => "Num5",
        0x36 => "Num6",
        0x37 => "Num7",
        0x38 => "Num8",
        0x39 => "Num9",

        // --- Function Keys (0x70 - 0x7B) ---
        0x70 => "F1",
        0x71 => "F2",
        0x72 => "F3",
        0x73 => "F4",
        0x74 => "F5",
        0x75 => "F6",
        0x76 => "F7",
        0x77 => "F8",
        0x78 => "F9",
        0x79 => "F10",
        0x7A => "F11",
        0x7B => "F12",

        // --- Special Keys ---
        0x20 => "Space",     // VK_SPACE
        0x1B => "Escape",    // VK_ESCAPE
        0x0D => "Enter",     // VK_RETURN
        0x08 => "Backspace", // VK_BACK
        0x09 => "Tab",       // VK_TAB
        0xC0 => "Backtick",  // VK_OEM_3 (The ` or ~ key)

        // Fallback
        _ => "Unknown",
    };
    String::from(code)
}
