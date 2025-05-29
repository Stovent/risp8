//! Experimental Chip8 interpreter, cached interpreter and Just-In-Time compiler.

pub use kanal::{Receiver, Sender};
use kanal::unbounded;

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

use std::fs::read;
use std::io::Error;
use std::time::{Duration, Instant};

/// The underlying type that represents the Chip8 screen.
pub type Screen = [[bool; State::SCREEN_WIDTH]; State::SCREEN_HEIGHT];
/// The default value of the screen.
pub const DEFAULT_SCREEN: Screen = [[false; State::SCREEN_WIDTH]; State::SCREEN_HEIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaitKey {
    NotWaiting,
    Waiting,
    Key(u8),
}

/// State of the chip8 virtual machine.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct State {
    SP: usize,
    PC: u16,
    I: u16,
    stack: [u16; 16],
    V: [u8; 16],
    memory: [u8; Self::MEMORY_SIZE],
    delay: u8,
    sound: u8,
    screen: Screen,
    keys: [bool; 16],

    wait_key: WaitKey,
}

impl State {
    pub const SCREEN_WIDTH: usize = 64;
    pub const SCREEN_HEIGHT: usize = 32;
    const INITIAL_PC: usize = 0x200; // 512.
    const MEMORY_SIZE: usize = 0x1000; // 4096.
    pub const MAX_PROGRAM_LEN: usize = Self::MEMORY_SIZE - Self::INITIAL_PC;

    /// Returns a new chip-8 state with the given program loaded.
    pub fn new(program: &[u8]) -> Self {
        Self {
            SP: 0,
            PC: Self::INITIAL_PC as u16,
            I: 0,
            stack: [0; 16],
            V: [0; 16],
            memory: Self::new_memory(program),
            delay: 0,
            sound: 0,
            screen: DEFAULT_SCREEN,
            keys: [false; 16],

            wait_key: WaitKey::NotWaiting,
        }
    }

    /// Returns the memory in its initial state with the given program loaded.
    ///
    /// `program` must not be greater than State::MAX_PROGRAM_LEN bytes.
    fn new_memory(program: &[u8]) -> [u8; State::MEMORY_SIZE] {
        assert!(program.len() <= Self::MAX_PROGRAM_LEN, "Input program ({} bytes) exceeds memory size ({} bytes)", program.len(), Self::MAX_PROGRAM_LEN);

        let mut memory = [0; State::MEMORY_SIZE];

        // Load font.
        memory[0..80].copy_from_slice(&[
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

        // Load program.
        let end = Self::INITIAL_PC + program.len();
        memory[Self::INITIAL_PC..end].copy_from_slice(program);

        memory
    }

    const fn clear_screen(&mut self) {
        self.screen = DEFAULT_SCREEN;
    }

    fn draw(&mut self, x: usize, y: usize, n: u8) {
        self.V[0xF] = 0;
        let x = self.V[x] as usize % State::SCREEN_WIDTH;
        let y = self.V[y] as usize % State::SCREEN_HEIGHT;

        for mut j in 0..n as usize {
            let line = self.memory[self.I as usize + j];
            j += y;

            for mut i in 0..8 {
                let mask = 0x80 >> i;
                i += x;
                if line & mask != 0 && i < State::SCREEN_WIDTH && j < State::SCREEN_HEIGHT {
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
    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if key <= 0xF {
            if self.wait_key == WaitKey::Waiting && self.keys[key] && !pressed { // Wait key triggers when released.
                self.wait_key = WaitKey::Key(key as u8);
            }
            self.keys[key] = pressed;
        }
    }

    /// Returns true if wait is over, false if it should continue to wait.
    fn wait_key(&mut self, x: usize) -> bool {
        match self.wait_key {
            WaitKey::NotWaiting => {
                self.wait_key = WaitKey::Waiting;
                false
            },
            WaitKey::Waiting => false,
            WaitKey::Key(k) => {
                self.V[x] = k;
                self.wait_key = WaitKey::NotWaiting;
                true
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
    const INTERPRETER_CACHES_LEN: usize = State::MAX_PROGRAM_LEN;
    const INTERPRETER_CACHES_LEN_2: usize = cached_interpreter_2::addr_to_index(State::MEMORY_SIZE as u16);

    const EMPTY_INTERPRETER_CACHES: Option<InstructionCache> = None;
    const EMPTY_INTERPRETER_CACHES_2: Option<[Option<InstructionCache>; cached_interpreter_2::SUBCACHE_SIZE]> = None;
    const EMPTY_INTERPRETER_CACHES_3: Option<CachedInstruction> = None;

    /// Creates a new Chip8 context.
    ///
    /// `rom` is the path to the ROM to open.
    pub fn new(rom: &str) -> Result<(Self, Sender<Risp8Command>, Receiver<Risp8Answer>), Error> {
        let (channel_out, user_in) = unbounded();
        let (user_out, channel_in) = unbounded();

        let core = Self {
            state: Self::new_state(rom)?,

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

        Ok((core, user_out, user_in))
    }

    fn new_state(filename: &str) -> Result<State, Error> {
        let program = read(filename)?;

        Ok(State::new(&program))
    }

    /// Starts emulation in an infinite loop.
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

    /// Returns true if the emulator has to be stopped (when the channel is closed or error).
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

/// Trait to get the address of a variable.
trait Address {
    /// Returns the address of `self`, possibly offsetted by the given number of bytes.
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
    Screen(Screen),
    /// Indicates that the sound should start to be continuously emited.
    ///
    /// This is emitted 60 times per seconds for as long as a sound should be emitted.
    PlaySound,
    /// Indicates that the sound should stop.
    StopSound,
}
