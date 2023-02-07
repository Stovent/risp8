#![allow(non_snake_case)]

#[cfg(target_arch = "x86_64")]
mod cache;
mod interpreter;
#[cfg(target_arch = "x86_64")]
mod jit;
mod opcode;

#[cfg(target_arch = "x86_64")]
use cache::Caches;

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

/// Chip8 context.
pub struct Chip8 {
    SP: usize,
    PC: u16,
    I: u16,
    stack: [u16; 16],
    V: [u8; 16],
    memory: [u8; 4096],
    delay: u8,
    sound: u8,
    pub screen: [[bool; 64]; 32],
    keys: [bool; 16],

    last_key: u8,
    timer: Instant,

    #[cfg(target_arch = "x86_64")]
    caches: Caches,
}

impl Chip8 {
    /// Creates a new Chip8 context.
    ///
    /// `rom` is the path to the ROM to open.
    pub fn new(rom: &str) -> Result<Chip8, String> {
        let mut core = Chip8 {
            SP: 0,
            PC: 512,
            I: 0,
            stack: [0; 16],
            V: [0; 16],
            memory: [0; 4096],
            delay: 0,
            sound: 0,
            screen: [[false; 64]; 32],
            keys: [false; 16],

            last_key: 255,
            timer: Instant::now(),

            #[cfg(target_arch = "x86_64")]
            caches: Caches::new(),
        };
        core.load_font();
        core.load_rom(rom)?;
        Ok(core)
    }

    fn load_rom(&mut self, filename: &str) -> Result<usize, String> {
        let mut input = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(format!("Could not open ROM: {}", e)),
        };

        match input.read(&mut self.memory[512..4096]) {
            Ok(size) => Ok(size),
            Err(e) => Err(format!("Could not read from ROM: {}", e)),
        }
    }

    fn load_font(&mut self) {
        self.memory[0..80].copy_from_slice(&[
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]);
    }

    fn clear_screen(&mut self) {
        self.screen = [[false; 64]; 32];
    }

    /// Sets a key as pressed or unpressed.
    ///
    /// `key` is the key number to set (0 to 9 for keys 0 to 9, and 10 to 15 for keys A to F).
    /// `pressed` = true if pressed, false if released.
    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if key <= 0xF {
            self.keys[key] = pressed;
            self.last_key = key as u8;
        }
    }

    fn draw(&mut self, x: usize, y: usize, n: u8) {
        for j in 0..n {
            let line = self.memory[(self.I + j as u16) as usize];

            for i in 0..8 {
                if line & (0x80 >> i) != 0 {
                    let y = ((self.V[y] + j) % 32) as usize;
                    let x = ((self.V[x] + i) % 64) as usize;

                    if self.screen[y][x] {
                        self.screen[y][x] = false;
                        self.V[15] = 1;
                    } else {
                        self.screen[y][x] = true;
                        self.V[15] = 0;
                    }
                }
            }
        }
    }

    fn handle_timers(&mut self) {
        if self.timer.elapsed() >= Duration::from_micros(16666) {
            if self.delay > 0 {
                self.delay -= 1;
            }

            if self.sound > 0 {
                // TODO: play sound
                self.sound -= 1;
            }

            self.timer = Instant::now();
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) extern "win64" fn handle_timers(this: &mut Chip8) {
    this.handle_timers();
}

#[cfg(not(target_os = "windows"))]
pub(crate) extern "sysv64" fn handle_timers(this: &mut Chip8) {
    this.handle_timers();
}

trait Address {
    fn address(&self, offset: usize) -> usize;
}

impl<T> Address for T {
    fn address(&self, offset: usize) -> usize {
        self as *const T as usize + offset
    }
}
