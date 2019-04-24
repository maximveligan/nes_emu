use serde::Serialize;
use std::collections::HashMap;
use serde::Deserialize;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use controller::Button;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pixel_scale: usize,
    pub sprites_per_scanline: u8,
    pub ctrl1_layout: ButtonLayout,
    pub ctrl2_layout: ButtonLayout,
    pub emu_controls: EmulatorControls,
    pub overscan: Overscan,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Overscan {
    pub top: u8,
    pub bottom: u8,
    pub left: u8,
    pub right: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmulatorControls {
    save_state: String,
    load_state: String,
    pause: String,
    reset: String,
}

impl EmulatorControls {
    pub fn make_emuctrl_map(&self) -> HashMap<String, EmuControl> {
        let mut emu_map = HashMap::new();
        emu_map.insert(self.save_state.clone(), EmuControl::SaveState);
        emu_map.insert(self.load_state.clone(), EmuControl::LoadState);
        emu_map.insert(self.pause.clone(), EmuControl::Pause);
        emu_map.insert(self.reset.clone(), EmuControl::Reset);
        emu_map
    }
}

pub enum EmuControl {
    SaveState,
    LoadState,
    Pause,
    Reset,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ButtonLayout {
    left: String,
    up: String,
    down: String,
    right: String,
    a: String,
    b: String,
    start: String,
    select: String,
}

impl ButtonLayout {
    pub fn make_ctrl_map(&self) -> HashMap<String, Button> {
        let mut button_map = HashMap::new();
        button_map.insert(self.left.clone(), Button::Left);
        button_map.insert(self.right.clone(), Button::Right);
        button_map.insert(self.down.clone(), Button::Down);
        button_map.insert(self.up.clone(), Button::Up);
        button_map.insert(self.a.clone(), Button::A);
        button_map.insert(self.b.clone(), Button::B);
        button_map.insert(self.start.clone(), Button::Start);
        button_map.insert(self.select.clone(), Button::Select);
        button_map
    }
}

pub enum ConfigError {
    FileError(std::io::Error),
    ParseError(toml::de::Error),
    UndefinedButton(String),
}

impl Config {
    pub fn generate_config() -> Config {
        let layout1 = ButtonLayout {
            left: "A".to_string(),
            up: "W".to_string(),
            down: "S".to_string(),
            right: "D".to_string(),
            a: "F".to_string(),
            b: "G".to_string(),
            start: "T".to_string(),
            select: "Y".to_string(),
        };

        let layout2 = ButtonLayout {
            left: "Left".to_string(),
            up: "Up".to_string(),
            down: "Down".to_string(),
            right: "Right".to_string(),
            a: "RShift".to_string(),
            b: "Enter".to_string(),
            start: "B".to_string(),
            select: "B".to_string(),
        };

        let overscan = Overscan {
            top: 8,
            bottom: 8,
            left: 0,
            right: 0,
        };

        let emulator_controls = EmulatorControls {
            save_state: "Q".to_string(),
            load_state: "E".to_string(),
            pause: "P".to_string(),
            reset: "R".to_string(),
        };

        Config {
            pixel_scale: 3,
            ctrl1_layout: layout1,
            ctrl2_layout: layout2,
            emu_controls: emulator_controls,
            overscan: overscan,
            sprites_per_scanline: 8,
        }
    }

    pub fn load_config(config_path: String) -> Result<Config, ConfigError> {
        if Path::new(&config_path).exists() {
            match File::open(config_path) {
                Ok(mut file) => {
                    let mut config_string = String::new();
                    match file.read_to_string(&mut config_string) {
                        Ok(_) => match toml::from_str(&config_string) {
                            Ok(config) => {
                                println!("Loading config: {:#?}", config);
                                Ok(config)
                            }
                            Err(err) => Err(ConfigError::ParseError(err)),
                        },
                        Err(err) => Err(ConfigError::FileError(err)),
                    }
                }
                Err(err) => Err(ConfigError::FileError(err)),
            }
        } else {
            Ok(Config::generate_config())
        }
    }
}
