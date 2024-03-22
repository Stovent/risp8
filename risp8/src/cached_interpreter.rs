//! Cached interpreter.
//!
//! Implemented using ideas from :
//! - <https://emudev.org/2021/01/31/cached-interpreter.html>
//! - <https://web.archive.org/web/20210301060701/https://ps1.asuramaru.com/emulator-development/cached-interpreters>
//!
//! This is the basic cached interpreter: each entry in [Chip8::interpreter_caches] is the cached instructions starting
//! at this PC, index with PC - 0x200 because theoretically there is no code execution below 0x200.
//!
//! When cache needs to be invalidated I have to search though every entries, which can be very slow if a lot of self-
//! modifying code is executed.
//! See cached_interpreter_2 for a O(1) cache invalidation method.

use crate::{Chip8, opcode::Opcode, State};

#[derive(Clone, Copy)]
pub(super) struct CachedInstruction {
    pub opcode: Opcode,
    /// Returns non-zero if execution must stop.
    pub execute: fn(&mut State, Opcode) -> u32,
}

#[derive(Clone)]
pub(super) struct InstructionCache {
    pub pc: u16,
    /// The address of the instruction following the last instruction in this cache [pc, end_pc).
    pub end_pc: u16,
    pub instructions: Vec<CachedInstruction>,
}

/// Converts the given Chip8 address to its instruction cache index.
#[inline(always)]
pub const fn addr_to_index(addr: u16) -> usize {
    addr as usize - State::INITIAL_PC
}

impl Chip8 {
    /// Executes a block of instructions using the cached interpreter.
    pub fn cached_interpreter(&mut self) {
        let cache_index = addr_to_index(self.state.PC);
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

        self.handle_timers();
    }

    /// Creates a new cache at the current PC. The state is not modified.
    fn new_cache_block(&self) -> InstructionCache {
        let block_pc = self.state.PC;
        let mut pc = self.state.PC;
        let mut instructions = Vec::new();

        'outer: loop {
            let opcode = Opcode((self.state.memory[pc as usize] as u16) << 8 | self.state.memory[pc as usize + 1] as u16);
            // #[cfg(debug_assertions)] println!("caching opcode {opcode:04X} at {pc:#X}");
            pc += 2;

            match opcode.0 >> 12 & 0xF {
                0x0 => match opcode.0 {
                    0x00E0 => instructions.push(CachedInstruction {opcode, execute: State::execute_00E0 }),
                    0x00EE => { instructions.push(CachedInstruction { opcode, execute: State::execute_00EE }); break 'outer; },
                    _ => break 'outer,
                },
                0x1 => { instructions.push(CachedInstruction { opcode, execute: State::execute_1nnn }); break 'outer; },
                0x2 => { instructions.push(CachedInstruction { opcode, execute: State::execute_2nnn }); break 'outer; },
                0x3 => instructions.push(CachedInstruction { opcode, execute: State::execute_3xkk }),
                0x4 => instructions.push(CachedInstruction { opcode, execute: State::execute_4xkk }),
                0x5 => instructions.push(CachedInstruction { opcode, execute: State::execute_5xy0 }),
                0x6 => instructions.push(CachedInstruction { opcode, execute: State::execute_6xkk }),
                0x7 => instructions.push(CachedInstruction { opcode, execute: State::execute_7xkk }),
                0x8 => {
                    match opcode.0 & 0xF00F {
                        0x8000 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy0 }),
                        0x8001 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy1 }),
                        0x8002 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy2 }),
                        0x8003 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy3 }),
                        0x8004 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy4 }),
                        0x8005 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy5 }),
                        0x8006 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy6 }),
                        0x8007 => instructions.push(CachedInstruction { opcode, execute: State::execute_8xy7 }),
                        0x800E => instructions.push(CachedInstruction { opcode, execute: State::execute_8xyE }),
                        _ => break 'outer,
                    }
                },
                0x9 => instructions.push(CachedInstruction { opcode, execute: State::execute_9xy0 }),
                0xA => instructions.push(CachedInstruction { opcode, execute: State::execute_Annn }),
                0xB => { instructions.push(CachedInstruction { opcode, execute: State::execute_Bnnn }); break 'outer; },
                0xC => instructions.push(CachedInstruction { opcode, execute: State::execute_Cxkk }),
                0xD => instructions.push(CachedInstruction { opcode, execute: State::execute_Dxyn }),
                0xE => match opcode.0 & 0xF0FF {
                    0xE09E => instructions.push(CachedInstruction { opcode, execute: State::execute_Ex9E }),
                    0xE0A1 => instructions.push(CachedInstruction { opcode, execute: State::execute_ExA1 }),
                    _ => break 'outer,
                },
                0xF => match opcode.0 & 0xF0FF {
                    0xF007 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx07 }),
                    // Wait Key: interrupt the current cache and go to a new cache starting at the wait key instruction.
                    0xF00A => { instructions.push(CachedInstruction { opcode, execute: State::execute_Fx0A }); break 'outer },
                    0xF015 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx15 }),
                    0xF018 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx18 }),
                    0xF01E => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx1E }),
                    0xF029 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx29 }),
                    0xF033 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx33 }),
                    0xF055 => { instructions.push(CachedInstruction { opcode, execute: State::execute_Fx55 }); break 'outer },
                    0xF065 => instructions.push(CachedInstruction { opcode, execute: State::execute_Fx65 }),
                    _ => break 'outer,
                },
                _ => break 'outer,
            };
        }

        if instructions.is_empty() {
            pc -= 2;
            let opcode = (self.state.memory[pc as usize] as u16) << 8 | self.state.memory[pc as usize + 1] as u16;
            panic!("Unknown opcode {opcode:04X} at {pc:#X}");
        }

        InstructionCache {
            pc: block_pc,
            end_pc: pc,
            instructions,
        }
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
