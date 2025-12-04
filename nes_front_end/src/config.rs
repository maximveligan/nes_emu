use anyhow::{Result, anyhow};
use glfw::Key;
use log::*;
use nes_emu::controller::Button;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub enum GpuBackend {
    OpenGL,
    Vulkan,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub ctrl1_layout: ButtonLayout,
    pub ctrl2_layout: ButtonLayout,
    pub emu_ctrl_layout: EmuControlLayout,
    pub overscan: Overscan,
    pub gpu_backend: GpuBackend,
    pub vsync: bool,
}

pub enum EmuControl {
    Pause,
    Reset,
    SpeedSwap,
    SaveState,
    LoadState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmuControlLayout {
    pause: String,
    reset: String,
    speed_swap: String,
    stop: String,
    save_state: String,
    load_state: String,
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

fn str_to_keycode(input: &str) -> Result<Key> {
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
        "Space" => Ok(Key::Space),
        "Escape" => Ok(Key::Escape),
        k => Err(anyhow!("Unsupported character {}", k)),
    }
}

impl ButtonLayout {
    pub fn make_ctrl_map(&self) -> Result<HashMap<Key, Button>> {
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

impl EmuControlLayout {
    pub fn make_emu_ctrl_map(&self) -> Result<HashMap<Key, EmuControl>> {
        let mut emu_ctrl_map = HashMap::new();
        emu_ctrl_map.insert(str_to_keycode(&self.pause)?, EmuControl::Pause);
        emu_ctrl_map.insert(str_to_keycode(&self.reset)?, EmuControl::Reset);
        emu_ctrl_map.insert(str_to_keycode(&self.speed_swap)?, EmuControl::SpeedSwap);
        emu_ctrl_map.insert(str_to_keycode(&self.save_state)?, EmuControl::SaveState);
        emu_ctrl_map.insert(str_to_keycode(&self.load_state)?, EmuControl::LoadState);
        Ok(emu_ctrl_map)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileError(std::io::Error),
    ParseError(toml::de::Error),
}

impl Config {
    pub fn generate_config() -> Config {
        let ctrl1_layout = ButtonLayout {
            left: "A".to_string(),
            up: "W".to_string(),
            down: "S".to_string(),
            right: "D".to_string(),
            a: "F".to_string(),
            b: "G".to_string(),
            start: "T".to_string(),
            select: "Y".to_string(),
        };

        let ctrl2_layout = ButtonLayout {
            left: "Left".to_string(),
            up: "Up".to_string(),
            down: "Down".to_string(),
            right: "Right".to_string(),
            a: "RShift".to_string(),
            b: "Enter".to_string(),
            start: "B".to_string(),
            select: "N".to_string(),
        };

        let emu_ctrl_layout = EmuControlLayout {
            speed_swap: "Space".to_string(),
            stop: "Escape".to_string(),
            pause: "P".to_string(),
            reset: "R".to_string(),
            save_state: "E".to_string(),
            load_state: "Q".to_string(),
        };

        let overscan = Overscan { top: 8, bottom: 8 };

        Config {
            ctrl1_layout,
            ctrl2_layout,
            overscan,
            emu_ctrl_layout,
            gpu_backend: GpuBackend::OpenGL,
            vsync: true,
        }
    }

    pub fn load_config(config_path: String) -> Result<Config> {
        if Path::new(&config_path).exists() {
            let mut file = File::open(config_path)?;
            let mut config_string = String::new();
            file.read_to_string(&mut config_string)?;
            let config = toml::from_str(&config_string)?;
            if log_enabled!(Level::Debug) {
                debug!("Loading config: {:#?}", config);
            }
            Ok(config)
        } else {
            if log_enabled!(Level::Warn) {
                warn!("Did not find a Config.toml for  the controls! Generating defaults");
            }
            Ok(Config::generate_config())
        }
    }
}
