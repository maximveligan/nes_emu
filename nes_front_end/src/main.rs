use anyhow::{Result, anyhow};
use config::{ButtonLayout, Config};
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent, fail_on_errors};
use log::Level;
use nes_emu::{NesEmulator, controller::Button, rom::load_rom};
use sha3::{Digest, Sha3_256};
use std::{
    collections::HashMap, env, error::Error, fs::File, io::Read, path::PathBuf, str, time::Instant,
};

use log::*;

use crate::config::{EmuControl, EmuControlLayout};

pub mod config;
pub mod ogl;

const FPS_TIMER: u128 = 16667;
const NES_SCREEN_WIDTH: u32 = 256;
const NES_SCREEN_HEIGHT: u32 = 240;

struct NesFrontEnd {
    nes: NesEmulator,
    ctrl1: HashMap<Key, Button>,
    ctrl2: HashMap<Key, Button>,
    emu_ctrl: HashMap<Key, EmuControl>,
    glfw: Glfw,
    paused: bool,
    uncapped: bool,
    state_path: PathBuf,
    vsync: bool,
    window: PWindow,
    frame_count: usize,
}

impl NesFrontEnd {
    fn new(path: &str, cfg: &Config) -> Result<(Self, GlfwReceiver<(f64, WindowEvent)>)> {
        let rom_path = PathBuf::from(path);
        let mut raw_rom = File::open(&rom_path)?;
        let mut raw_bytes = Vec::new();
        raw_rom.read_to_end(&mut raw_bytes)?;

        let rom = load_rom(&raw_bytes)?;
        let state_name;

        if let Some(os_stem) = rom_path.file_stem() {
            state_name = os_stem
                .to_str()
                .ok_or(anyhow!("Failed to convert from utf-8"))?;
        } else {
            warn!("Rom file name did not have an extension");
            state_name = path;
        }

        let mut glfw = glfw::init(fail_on_errors!())?;
        let (mut window, events) = glfw
            .create_window(
                NES_SCREEN_WIDTH,
                NES_SCREEN_HEIGHT,
                "Res",
                glfw::WindowMode::Windowed,
            )
            .ok_or(anyhow!("Create window failed"))?;
        window.make_current();
        window.set_key_polling(true);
        window.is_resizable();
        window.set_drag_and_drop_polling(true);

        if cfg.vsync {
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        }

        Ok((
            NesFrontEnd {
                nes: NesEmulator::new(rom),
                ctrl1: ButtonLayout::make_ctrl_map(&cfg.ctrl1_layout)?,
                ctrl2: ButtonLayout::make_ctrl_map(&cfg.ctrl2_layout)?,
                emu_ctrl: EmuControlLayout::make_emu_ctrl_map(&cfg.emu_ctrl_layout)?,
                paused: false,
                uncapped: false,
                state_path: PathBuf::from(state_name),
                vsync: cfg.vsync,
                glfw,
                window,
                frame_count: 0,
            },
            events,
        ))
    }

    fn next_frame(&mut self) -> &[u8] {
        self.nes.next_frame()
    }

    fn frame_info(&self) -> (String, usize) {
        (
            hex::encode(Sha3_256::digest(self.nes.cur_frame())),
            self.frame_count,
        )
    }

    fn set_ctrl_state(&mut self, key: Key, action: Action) {
        let state = action != Action::Release;
        if let Some(button) = self.ctrl1.get(&key) {
            self.nes.mmu.ctrl0.set_button_state(*button, state);
        }

        if let Some(button) = self.ctrl2.get(&key) {
            self.nes.mmu.ctrl1.set_button_state(*button, state);
        }
    }

    fn save_state(&self) -> Result<()> {
        let mut file = File::create(&self.state_path)?;
        self.nes.get_state().save(&mut file)?;
        info!("Saved state: {:?}", &self.state_path);
        Ok(())
    }

    fn load_state(&mut self) -> Result<()> {
        let mut file = File::open(&self.state_path)?;
        let state = nes_emu::state::State::load(&mut file)?;
        self.nes.load_state(state);
        info!("Loaded state: {:?}", &self.state_path);
        Ok(())
    }

    fn speed_swap(&mut self, action: Action) {
        match action {
            Action::Press => {
                self.uncapped = true;
                self.glfw.set_swap_interval(glfw::SwapInterval::None);
            }
            Action::Release => {
                if self.vsync {
                    self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
                }
                self.uncapped = false;
            }
            _ => {
                if log_enabled!(Level::Warn) {
                    warn!("Got unexpected action: release");
                }
            }
        }
    }

    fn emu_ctrl_handler(&mut self, key: Key, action: Action) -> Result<()> {
        if let Some(emu_ctrl) = self.emu_ctrl.get(&key) {
            match emu_ctrl {
                &EmuControl::Pause if action == Action::Press => {
                    self.paused = !self.paused;
                    Ok(())
                }
                &EmuControl::Reset if action == Action::Press => {
                    self.nes.reset();
                    Ok(())
                }
                &EmuControl::SpeedSwap => {
                    self.speed_swap(action);
                    Ok(())
                }
                &EmuControl::SaveState => self.save_state(),
                &EmuControl::LoadState => self.load_state(),
                &EmuControl::Hash => {
                    println!("{:?}", self.frame_info());
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn event_handler(&mut self, events: &GlfwReceiver<(f64, WindowEvent)>) -> Result<bool> {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                WindowEvent::Close => return Ok(false),
                WindowEvent::FileDrop(f) => {
                    if f.len() > 1 {
                        warn!(
                            "Expected one file, got {}. Defaulting to first file, {:?}",
                            f.len(),
                            f[0].to_str()
                        );
                    }
                }
                WindowEvent::Key(k, _, a, _) => {
                    self.set_ctrl_state(k, a);
                    self.emu_ctrl_handler(k, a)?;
                }
                _ => (),
            }
        }
        Ok(true)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut nes_fe, events) = NesFrontEnd::new(
        &env::args()
            .nth(1)
            .ok_or("Did not recieve a valid ROM path")?,
        &Config::load_config("./config.toml".to_string())?,
    )?;

    let mut ts = Instant::now();
    let texture = ogl::init_shaders_and_texture(&mut nes_fe.window);

    while !nes_fe.window.should_close() {
        nes_fe.glfw.poll_events();
        if !nes_fe.event_handler(&events)? {
            break;
        }

        if nes_fe.paused {
            continue;
        }

        if nes_fe.uncapped || (Instant::now() - ts).as_micros() > FPS_TIMER {
            nes_fe.frame_count += 1;
            ogl::update_texture(texture, nes_fe.next_frame());
            ts = Instant::now();
            nes_fe.window.swap_buffers();
        }
    }

    Ok(())
}
