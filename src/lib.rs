//! Experimental Chip8 interpreter, cached interpreter and Just-In-Time compiler.

#![feature(const_eval_limit)]
#![feature(const_mut_refs)]
#![feature(const_option)]
#![feature(drain_filter)]
#![const_eval_limit = "0"]

use kanal::{Receiver, Sender, unbounded};

#[cfg(target_arch = "x86_64")]
mod cache;
mod cached_interpreter;
mod cached_interpreter_2;
mod cached_interpreter_3;
mod interpreter;
#[cfg(target_arch = "x86_64")]
mod jit;
mod opcode;

#[cfg(target_arch = "x86_64")]
use cache::Caches;

use cached_interpreter::{InstructionCache, CachedInstruction};

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
struct State {
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
}

impl State {
    fn new() -> Self {
        let mut state = Self {
            SP: 0,
            PC: Chip8::INITIAL_PC,
            I: 0,
            stack: [0; 16],
            V: [0; 16],
            memory: [0; 4096],
            delay: 0,
            sound: 0,
            screen: [[false; 64]; 32],
            keys: [false; 16],

            wait_key: None,
        };
        state.load_font();

        state
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

    fn draw(&mut self, x: usize, y: usize, n: u8) {
        self.V[0xF] = 0;
        let x = self.V[x] as usize % 64;
        let y = self.V[y] as usize % 32;

        for mut j in 0..n as usize {
            let line = self.memory[self.I as usize + j];
            j += y;

            for mut i in 0..8 {
                let mask = 0x80 >> i;
                i += x;
                if line & mask != 0 && i < 64 && j < 32 {
                    if self.screen[j][i] {
                        self.screen[j][i] = false;
                        self.V[0xF] = 1;
                    } else {
                        self.screen[j][i] = true;
                    }
                }
            }
        }
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

    /// Returns true if wait is over, false if it should continue to wait.
    fn wait_key(&mut self, x: usize) -> bool {
        match self.wait_key {
            Some(_key@0..=0xF) => {
                // If key is valid, store the key in the given register.
                self.V[x] = self.wait_key.take().unwrap();
                true
            },
            _ => {
                // Set to waiting.
                self.wait_key = Some(255);
                false
            },
        }
    }
}

/// Chip8 core.
pub struct Chip8 {
    state: State,

    timer: Instant,

    channel_in: Receiver<Risp8Command>,
    channel_out: Sender<Risp8Answer>,
    play: bool,
    execution_method: ExecutionMethod,

    interpreter_caches: Box<[Option<InstructionCache>]>,
    interpreter_caches_2: Box<[Option<[Option<InstructionCache>; cached_interpreter_2::SUBCACHE_SIZE]>]>,
    interpreter_caches_3: Box<[Option<CachedInstruction>]>,

    #[cfg(target_arch = "x86_64")]
    jit_caches: Caches,
}

impl Chip8 {
    const INITIAL_PC: u16 = 0x200; // 512.
    const MEMORY_END: u16 = 0x1000; // 4096.

    const INTERPRETER_CACHES_LEN: usize = (Self::MEMORY_END - Self::INITIAL_PC) as usize;
    const INTERPRETER_CACHES_LEN_2: usize = cached_interpreter_2::addr_to_index(Self::MEMORY_END);

    const EMPTY_INTERPRETER_CACHES: Option<InstructionCache> = None;
    const EMPTY_INTERPRETER_CACHES_2: Option<[Option<InstructionCache>; cached_interpreter_2::SUBCACHE_SIZE]> = None;
    const EMPTY_INTERPRETER_CACHES_3: Option<CachedInstruction> = None;

    /// Creates a new Chip8 context.
    ///
    /// `rom` is the path to the ROM to open.
    pub fn new(rom: &str) -> Result<(Self, Sender<Risp8Command>, Receiver<Risp8Answer>), String> {
        let (channel_out, user_in) = unbounded();
        let (user_out, channel_in) = unbounded();

        let mut core = Chip8 {
            state: State::new(),

            timer: Instant::now(),

            channel_in,
            channel_out,
            play: false,
            execution_method: ExecutionMethod::Interpreter,

            interpreter_caches: vec![Self::EMPTY_INTERPRETER_CACHES; Self::INTERPRETER_CACHES_LEN].into_boxed_slice(),
            interpreter_caches_2: vec![Self::EMPTY_INTERPRETER_CACHES_2; Self::INTERPRETER_CACHES_LEN_2].into_boxed_slice(),
            interpreter_caches_3: vec![Self::EMPTY_INTERPRETER_CACHES_3; Self::INTERPRETER_CACHES_LEN].into_boxed_slice(),

            #[cfg(target_arch = "x86_64")]
            jit_caches: Caches::new(),
        };

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
                self.single_step();
            }
        }
    }

    fn single_step(&mut self) {
        match self.execution_method {
            ExecutionMethod::Interpreter => self.interpreter(),
            ExecutionMethod::CachedInterpreter => self.cached_interpreter(),
            ExecutionMethod::CachedInterpreter2 => self.cached_interpreter_2(),
            ExecutionMethod::CachedInterpreter3 => self.cached_interpreter_3(),
            ExecutionMethod::Jit => self.jit(),
        }
    }

    /// Returns true if the emulator has to be stopped.
    fn handle_channels(&mut self) -> bool {
        while !self.channel_in.is_empty() {
            let Ok(cmd) = self.channel_in.recv() else {
                return true;
            };

            match cmd {
                Risp8Command::SetKey(key, pressed) => self.state.set_key(key, pressed),
                Risp8Command::GetScreen => { let _ = self.channel_out.send(Risp8Answer::Screen(self.state.screen)); },
                Risp8Command::Play => self.play = true,
                Risp8Command::Pause => self.play = false,
                Risp8Command::SingleStep => self.single_step(),
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

        match input.read(&mut self.state.memory[Self::INITIAL_PC as usize..Self::MEMORY_END as usize]) {
            Ok(size) => Ok(size),
            Err(e) => Err(format!("Could not read from ROM: {}", e)),
        }
    }

    fn handle_timers(&mut self) {
        if self.timer.elapsed() >= Duration::from_micros(16666) {
            if self.state.delay > 0 {
                self.state.delay -= 1;
            }

            if self.state.sound > 0 {
                self.state.sound -= 1;
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
    /// Set a key as pressed or unpressed.
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
    /// Run the execution method once.
    SingleStep,
    /// Set the execution method.
    SetExecutionMethod(ExecutionMethod),
    /// Request to end the [run](Chip8::run) method.
    Exit,
}

/// Specifies which method to use to execute instructions.
#[derive(Debug)]
pub enum ExecutionMethod {
    Interpreter,
    CachedInterpreter,
    CachedInterpreter2,
    CachedInterpreter3,
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
