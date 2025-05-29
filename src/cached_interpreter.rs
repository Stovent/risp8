//! Cache interpreter.
//!
//! Implemented using ideas from :
//! - https://emudev.org/2021/01/31/cached-interpreter.html
//! - https://web.archive.org/web/20210301060701/https://ps1.asuramaru.com/emulator-development/cached-interpreters

use crate::{Chip8, opcode::Opcode, State};

#[derive(Clone, Copy)]
pub(super) struct CachedInstruction {
    opcode: Opcode,
    /// Returns true if execution must stop.
    execute: fn (&mut State, Opcode) -> u32,
}

#[derive(Clone)]
pub(super) struct InstructionCache {
    pc: u16,
    /// The address of the instruction following the last instruction in this cache [pc, end_pc).
    end_pc: u16,
    instructions: Vec<CachedInstruction>,
}

impl InstructionCache {
    fn push(&mut self, inst: CachedInstruction) {
        self.instructions.push(inst);
    }
}

impl Chip8 {
    pub(super) fn cached_interpreter(&mut self) {
        self.handle_timers();

        let cache_index = (self.state.PC - Self::INITIAL_PC) as usize;
        let cache = if let Some(cache) = &self.interpreter_caches[cache_index] {
            cache
        } else {
            let cache = self.new_cache_block();
            self.interpreter_caches[cache_index] = Some(cache);
            self.interpreter_caches[cache_index].as_ref().unwrap()
        };

        // Execute the cache.
        let mut ret = 0;
        for inst in &cache.instructions {
            // #[cfg(debug_assertions)] println!("cached opcode {:04X} at {:#X}", inst.opcode, self.state.PC);

            self.state.PC += 2;
            let r = (inst.execute)(&mut self.state, inst.opcode);
            if r != 0 {
                ret = r;
                break;
            }
        }

        if ret > 1 {
            self.invalidate_cache((ret >> 16) as u16, ret as u16);
        }
    }

    /// Creates a new cache at the current PC. The state is not modified.
    fn new_cache_block(&self) -> InstructionCache {
        let mut pc = self.state.PC;
        let mut cache = InstructionCache {
            pc,
            end_pc: pc,
            instructions: Vec::new(),
        };

        'outer: loop {
            let opcode = Opcode((self.state.memory[pc as usize] as u16) << 8 | self.state.memory[pc as usize + 1] as u16);
            // #[cfg(debug_assertions)] println!("caching opcode {opcode:04X} at {pc:#X}");
            pc += 2;

            match opcode.0 >> 12 & 0xF {
                0x0 => match opcode.0 {
                    0x00E0 => cache.push(CachedInstruction {opcode, execute: State::execute_00E0 }),
                    0x00EE => { cache.push(CachedInstruction { opcode, execute: State::execute_00EE }); break 'outer; },
                    _ => break 'outer,
                },
                0x1 => { cache.push(CachedInstruction { opcode, execute: State::execute_1nnn }); break 'outer; },
                0x2 => { cache.push(CachedInstruction { opcode, execute: State::execute_2nnn }); break 'outer; },
                0x3 => cache.push(CachedInstruction { opcode, execute: State::execute_3xkk }),
                0x4 => cache.push(CachedInstruction { opcode, execute: State::execute_4xkk }),
                0x5 => cache.push(CachedInstruction { opcode, execute: State::execute_5xy0 }),
                0x6 => cache.push(CachedInstruction { opcode, execute: State::execute_6xkk }),
                0x7 => cache.push(CachedInstruction { opcode, execute: State::execute_7xkk }),
                0x8 => {
                    match opcode.0 & 0xF00F {
                        0x8000 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy0 }),
                        0x8001 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy1 }),
                        0x8002 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy2 }),
                        0x8003 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy3 }),
                        0x8004 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy4 }),
                        0x8005 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy5 }),
                        0x8006 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy6 }),
                        0x8007 => cache.push(CachedInstruction { opcode, execute: State::execute_8xy7 }),
                        0x800E => cache.push(CachedInstruction { opcode, execute: State::execute_8xyE }),
                        _ => break 'outer,
                    }
                },
                0x9 => cache.push(CachedInstruction { opcode, execute: State::execute_9xy0 }),
                0xA => cache.push(CachedInstruction { opcode, execute: State::execute_Annn }),
                0xB => { cache.push(CachedInstruction { opcode, execute: State::execute_Bnnn }); break 'outer; },
                0xC => cache.push(CachedInstruction { opcode, execute: State::execute_Cxkk }),
                0xD => cache.push(CachedInstruction { opcode, execute: State::execute_Dxyn }),
                0xE => match opcode.0 & 0xF0FF {
                    0xE09E => cache.push(CachedInstruction { opcode, execute: State::execute_Ex9E }),
                    0xE0A1 => cache.push(CachedInstruction { opcode, execute: State::execute_ExA1 }),
                    _ => break 'outer,
                },
                0xF => match opcode.0 & 0xF0FF {
                    0xF007 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx07 }),
                    // Wait Key: interrupt the current cache and go to a new cache starting at the wait key instruction.
                    0xF00A => { cache.push(CachedInstruction { opcode, execute: State::execute_Fx0A }); break 'outer },
                    0xF015 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx15 }),
                    0xF018 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx18 }),
                    0xF01E => cache.push(CachedInstruction { opcode, execute: State::execute_Fx1E }),
                    0xF029 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx29 }),
                    0xF033 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx33 }),
                    0xF055 => { cache.push(CachedInstruction { opcode, execute: State::execute_Fx55 }); break 'outer },
                    0xF065 => cache.push(CachedInstruction { opcode, execute: State::execute_Fx65 }),
                    _ => break 'outer,
                },
                _ => break 'outer,
            };
        }

        cache.end_pc = pc;

        if cache.instructions.is_empty() {
            pc -= 2;
            let opcode = (self.state.memory[pc as usize] as u16) << 8 | self.state.memory[pc as usize + 1] as u16;
            panic!("Unknown opcode {opcode:04X} at {pc:#X}");
        }

        cache
    }

    /// Returns true if the current cache has been invalidated.
    fn invalidate_cache(&mut self, beg_addr: u16, end_addr: u16) {
        for i in 0..self.interpreter_caches.len() {
            if let Some(cache) = &self.interpreter_caches[i] {
                if beg_addr >= cache.pc && beg_addr < cache.end_pc ||
                    end_addr >= cache.pc && end_addr < cache.end_pc
                {
                    self.interpreter_caches[i] = None;
                }
            }
        }
    }
}
