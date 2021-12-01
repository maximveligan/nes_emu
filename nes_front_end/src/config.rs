use glfw::Key;
use nes_emu::controller::Button;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pixel_scale: usize,
    pub ctrl1_layout: ButtonLayout,
    pub ctrl2_layout: ButtonLayout,
    pub overscan: Overscan,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Overscan {
    pub top: u8,
    pub bottom: u8,
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

fn str_to_keycode(input: &str) -> Result<Key, Box<dyn Error>> {
    match input {
        "A" => Ok(Key::A),
        "B" => Ok(Key::B),
        "C" => Ok(Key::C),
        "D" => Ok(Key::D),
        "E" => Ok(Key::E),
        "F" => Ok(Key::F),
        "G" => Ok(Key::G),
        "H" => Ok(Key::H),
        "I" => Ok(Key::I),
        "J" => Ok(Key::J),
        "K" => Ok(Key::K),
        "L" => Ok(Key::L),
        "M" => Ok(Key::M),
        "N" => Ok(Key::N),
        "O" => Ok(Key::O),
        "P" => Ok(Key::P),
        "Q" => Ok(Key::Q),
        "R" => Ok(Key::R),
        "S" => Ok(Key::S),
        "T" => Ok(Key::T),
        "U" => Ok(Key::U),
        "V" => Ok(Key::V),
        "W" => Ok(Key::W),
        "X" => Ok(Key::X),
        "Y" => Ok(Key::Y),
        "Z" => Ok(Key::Z),
        "Left" => Ok(Key::Left),
        "Down" => Ok(Key::Down),
        "Up" => Ok(Key::Up),
        "Right" => Ok(Key::Right),
        "LShift" => Ok(Key::LeftShift),
        "RShift" => Ok(Key::RightShift),
        "Enter" => Ok(Key::Enter),
        k => Err(format!("Unsupported character {}", k).into()),
    }
}

impl ButtonLayout {
    pub fn make_ctrl_map(&self) -> Result<HashMap<Key, Button>, Box<dyn Error>> {
        let mut button_map = HashMap::new();
        button_map.insert(str_to_keycode(&self.left)?, Button::Left);
        button_map.insert(str_to_keycode(&self.right)?, Button::Right);
        button_map.insert(str_to_keycode(&self.down)?, Button::Down);
        button_map.insert(str_to_keycode(&self.up)?, Button::Up);
        button_map.insert(str_to_keycode(&self.a)?, Button::A);
        button_map.insert(str_to_keycode(&self.b)?, Button::B);
        button_map.insert(str_to_keycode(&self.start)?, Button::Start);
        button_map.insert(str_to_keycode(&self.select)?, Button::Select);
        Ok(button_map)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileError(std::io::Error),
    ParseError(toml::de::Error),
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

        let overscan = Overscan { top: 8, bottom: 8 };

        Config {
            pixel_scale: 6,
            ctrl1_layout: layout1,
            ctrl2_layout: layout2,
            overscan,
        }
    }

    pub fn load_config(config_path: String) -> Result<Config, Box<dyn Error>> {
        if Path::new(&config_path).exists() {
            let mut file = File::open(config_path)?;
            let mut config_string = String::new();
            file.read_to_string(&mut config_string)?;
            let config = toml::from_str(&config_string)?;
            println!("Loading config: {:#?}", config);
            Ok(config)
        } else {
            Ok(Config::generate_config())
        }
    }
}
