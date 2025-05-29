#![allow(dead_code)]
#![allow(non_snake_case)]

mod cache;
mod interpreter;
mod jit;
mod utils;
mod x86;

use cache::Caches;

use std::fs::File;
use std::io::Read;

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
    screen: [bool; 2048],
    keys: [bool; 16],

    last_key: u8,
    clock_delay: f64,
    dec_timer_ms: f64,

    caches: Caches,
}

impl Chip8 {
    /// Creates a new Chip8 context.
    ///
    /// `rom` is the path to the ROM to open.
    /// `freq` is the speed of emulation, in instructions per seconds (500 is a good default value).
    pub fn new(rom: &str, freq: usize) -> Result<Chip8, String> {
        let mut core = Chip8 {
            SP: 0,
            PC: 512,
            I: 0,
            stack: [0; 16],
            V: [0; 16],
            memory: [0; 4096],
            delay: 0,
            sound: 0,
            screen: [false; 2048],
            keys: [false; 16],

            last_key: 255,
            clock_delay: 1.0 / freq as f64,
            dec_timer_ms: 0.0,

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
            Ok(size) => {
                println!("Successfully opened ROM \"{}\" (size: {} bytes)", filename, size);
                Ok(size)
            }
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
        self.screen = [false; 2048];
    }

    /// Sets a key as pressed or unpressed.
    ///
    /// `key` is the key number to set (0 to 9 for keys 0 to 9, and 10 to 15 for keys A to F).
    /// `pressed` = true if pressed, false if released.
    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if key > 15 {
            return;
        }
        self.keys[key] = pressed;
        self.last_key = key as u8;
    }

    fn draw(&mut self, x: usize, y: usize, n: u8) {
        for j in 0..n {
            let line: u8 = self.memory[(self.I + j as u16) as usize];

            for i in 0..8u8 {
                if line & (0x80 >> i) != 0 {
                    let index: usize = ((self.V[y] + j) as usize * 64 + (self.V[x] + i) as usize) % 2048;

                    if self.screen[index] {
                        self.screen[index] = false;
                        self.V[15] = 1;
                    } else {
                        self.screen[index] = true;
                        self.V[15] = 0;
                    }
                }
            }
        }
    }
}
