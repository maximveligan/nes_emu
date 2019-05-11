use serde::Serialize;
use std::collections::HashMap;
use serde::Deserialize;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use controller::Button;
use failure::Error;
use sdl2::keyboard::Keycode;

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

fn str_to_keycode(input: &str) -> Result<Keycode, Error> {
    match input {
        "A" => Ok(Keycode::A),
        "B" => Ok(Keycode::B),
        "C" => Ok(Keycode::C),
        "D" => Ok(Keycode::D),
        "E" => Ok(Keycode::E),
        "F" => Ok(Keycode::F),
        "G" => Ok(Keycode::G),
        "H" => Ok(Keycode::H),
        "I" => Ok(Keycode::I),
        "J" => Ok(Keycode::J),
        "K" => Ok(Keycode::K),
        "L" => Ok(Keycode::L),
        "M" => Ok(Keycode::M),
        "N" => Ok(Keycode::N),
        "O" => Ok(Keycode::O),
        "P" => Ok(Keycode::P),
        "Q" => Ok(Keycode::Q),
        "R" => Ok(Keycode::R),
        "S" => Ok(Keycode::S),
        "T" => Ok(Keycode::T),
        "U" => Ok(Keycode::U),
        "V" => Ok(Keycode::V),
        "W" => Ok(Keycode::W),
        "X" => Ok(Keycode::X),
        "Y" => Ok(Keycode::Y),
        "Z" => Ok(Keycode::Z),
        "Left" => Ok(Keycode::Left),
        "Down" => Ok(Keycode::Down),
        "Up" => Ok(Keycode::Up),
        "Right" => Ok(Keycode::Right),
        "LShift" => Ok(Keycode::LShift),
        "RShift" => Ok(Keycode::RShift),
        "Enter" => Ok(Keycode::Return),
        k => Err(format_err!("Unsupported character {}", k)),
    }
}

impl ButtonLayout {
    pub fn make_ctrl_map(&self) -> Result<HashMap<Keycode, Button>, Error> {
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

#[derive(Debug, Fail)]
pub enum ConfigError {
    #[fail(display = "Unable to open config file: {}", _0)]
    FileError(std::io::Error),
    #[fail(display = "Unable to parse config file: {}", _0)]
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
            pixel_scale: 3,
            ctrl1_layout: layout1,
            ctrl2_layout: layout2,
            overscan: overscan,
        }
    }

    pub fn load_config(config_path: String) -> Result<Config, Error> {
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
