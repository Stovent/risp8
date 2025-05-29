#![allow(non_snake_case)]
#![feature(drain_filter)]

use kanal::{Receiver, Sender, unbounded};

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
    screen: [[bool; 64]; 32],
    keys: [bool; 16],

    /// If None, the ROM is not waiting for a key.
    ///
    /// If Some(> 0xF), a wait key instruction has occured but no new key has been pressed yet.
    ///
    /// If Some(<= 0xF), the awaited key has been pressed and instruction execution will resume on the next loop.
    wait_key: Option<u8>,
    timer: Instant,

    channel_in: Receiver<Risp8Command>,
    channel_out: Sender<Risp8Answer>,
    play: bool,
    execution_method: ExecutionMethod,

    #[cfg(target_arch = "x86_64")]
    caches: Caches,
}

impl Chip8 {
    /// Creates a new Chip8 context.
    ///
    /// `rom` is the path to the ROM to open.
    pub fn new(rom: &str) -> Result<(Self, Sender<Risp8Command>, Receiver<Risp8Answer>), String> {
        let (channel_out, user_in) = unbounded();
        let (user_out, channel_in) = unbounded();

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

            wait_key: None,
            timer: Instant::now(),

            channel_in,
            channel_out,
            play: false,
            execution_method: ExecutionMethod::Interpreter,

            #[cfg(target_arch = "x86_64")]
            caches: Caches::new(),
        };

        core.load_font();
        core.load_rom(rom)?;

        Ok((core, user_out, user_in))
    }

    /// The method to call to start emulation.
    ///
    /// This method is meant to run concurrently with the rest of the program (GUI, ...).
    /// Use the channels to send commands to control the core and receive answers from it.
    pub fn run(&mut self) {
        loop {
            if self.handle_channels() {
                break;
            }

            if self.play {
                match self.execution_method {
                    ExecutionMethod::Interpreter => self.interpreter(),
                    ExecutionMethod::Jit => self.jit(),
                }
            }
        }
    }

    /// Returns true if the emulator has to be stopped.
    fn handle_channels(&mut self) -> bool {
        while !self.channel_in.is_empty() {
            let Ok(cmd) = self.channel_in.recv() else {
                return true;
            };

            match cmd {
                Risp8Command::SetKey(key, pressed) => self.set_key(key, pressed),
                Risp8Command::GetScreen => { let _ = self.channel_out.send(Risp8Answer::Screen(self.screen)); },
                Risp8Command::Play => self.play = true,
                Risp8Command::Pause => self.play = false,
                Risp8Command::SetExecutionMethod(method) => self.execution_method = method,
                Risp8Command::Exit => return true,
            }
        }

        false
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

    /// Sets a key as pressed or unpressed.
    ///
    /// `key` is the key number to set (0 to 9 for keys 0 to 9, and 10 to 15 for keys A to F).
    /// `pressed` = true if pressed, false if released.
    fn set_key(&mut self, key: usize, pressed: bool) {
        if key <= 0xF {
            if self.keys[key] && !pressed { // Key pressed then released.
                match self.wait_key {
                    Some(16..=u8::MAX) => self.wait_key = Some(key as u8),
                    _ => (),
                }
            }
            self.keys[key] = pressed;
        }
    }

    fn wait_key(&mut self, x: usize) {
        match self.wait_key {
            Some(key) => if key <= 0xF {
                // If key is valid, store the key in the given register.
                self.V[x] = self.wait_key.take().unwrap();
            } else {
                // If it is still waiting for a key, decrement PC to make it loop over this instruction.
                self.PC -= 2;
            },
            None => {
                // First execution of the instruction, set it to waiting.
                self.wait_key = Some(255);
                self.PC -= 2;
            },
        }
    }

    fn clear_screen(&mut self) {
        self.screen = [[false; 64]; 32];
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
                self.sound -= 1;
                let _ = self.channel_out.send(Risp8Answer::PlaySound);
            } else {
                let _ = self.channel_out.send(Risp8Answer::StopSound);
            }

            self.timer = Instant::now();
        }
    }
}

trait Address {
    fn address(&self, offset: usize) -> usize;
}

impl<T> Address for T {
    fn address(&self, offset: usize) -> usize {
        self as *const T as usize + offset
    }
}

/// Commands to send to the core.
#[derive(Debug)]
pub enum Risp8Command {
    /// Sets a key as pressed or unpressed.
    ///
    /// `usize` is the key number to set (0 to 9 for keys 0 to 9, and 10 to 15 for keys A to F).
    /// `bool` = true if pressed, false if released.
    SetKey(usize, bool),
    /// Request to get the current state of the screen.
    GetScreen,
    /// Resume emulation.
    Play,
    /// Pause emulation.
    Pause,
    /// Set the execution method.
    SetExecutionMethod(ExecutionMethod),
    /// Request to end the [run](Chip8::run) method.
    Exit,
}

/// Specifies which method to use to execute instructions.
#[derive(Debug)]
pub enum ExecutionMethod {
    Interpreter,
    Jit,
}

/// Answers from the core.
#[derive(Debug)]
pub enum Risp8Answer {
    /// A copy of the screen.
    Screen([[bool; 64]; 32]),
    /// Indicates that the sound should start to be continuously emited.
    ///
    /// This is emitted 60 times per seconds for as long as a sound should be emitted.
    PlaySound,
    /// Indicates that the sound should stop.
    StopSound,
}
