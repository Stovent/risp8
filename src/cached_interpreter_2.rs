//! Cached interpreter, idea 2.
//!
//! It implements the trie lookup explained here:
//! https://web.archive.org/web/20210301060701/https://ps1.asuramaru.com/emulator-development/cached-interpreters
//!
//! The way the caches works is [Chip8::interpreter_caches_2]'s size is 224, indexed with (PC - 0x200) >> 4.
//! This returns a pool of 16 caches indexed with (PC - 0x200) & 0xF.
//! Instructions are added to the cache of a pool as long as their address is in this pool, so
//! while PC & 0xF != 0.
//! This makes cache invalidation O(1) on every memory write.

use crate::{
    Chip8,
    opcode::Opcode,
    State,
    cached_interpreter::{
        CachedInstruction,
        InstructionCache,
    },
};

const SUBCACHE_SHIFT: u16 = 4;
pub(super) const SUBCACHE_SIZE: usize = 1 << SUBCACHE_SHIFT as usize;
const SUBCACHE_MASK: u16 = SUBCACHE_SIZE as u16 - 1;

/// Converts the given Chip8 address to its instruction cache index.
#[inline(always)]
pub(super) const fn addr_to_index(addr: u16) -> usize {
    addr as usize - State::INITIAL_PC >> SUBCACHE_SHIFT
}

/// Converts the given Chip8 address to its index in the cache.
#[inline(always)]
const fn index_in_subcache(addr: u16) -> usize {
    addr as usize - State::INITIAL_PC & SUBCACHE_MASK as usize
}

impl Chip8 {
    /// Executes a block of instructions using the cached interpreter variant 2.
    pub fn cached_interpreter_2(&mut self) {
        let pool_index = addr_to_index(self.state.PC);
        let pool = if let Some(pool) = &mut self.interpreter_caches_2[pool_index] {
            pool
        } else {
            let pool = [Chip8::EMPTY_INTERPRETER_CACHES; SUBCACHE_SIZE];
            self.interpreter_caches_2[pool_index] = Some(pool);
            self.interpreter_caches_2[pool_index].as_mut().unwrap()
        };

        let cache_index = index_in_subcache(self.state.PC);
        let cache = if let Some(cache) = &pool[cache_index] {
            cache
        } else {
            let cache = Self::new_cache_block_2(self.state.PC, &self.state.memory);
            pool[cache_index] = Some(cache);
            pool[cache_index].as_ref().unwrap()
        };

        // Execute the cache.
        let mut ret = 0;
        for inst in &cache.instructions {
            // #[cfg(debug_assertions)] println!("cached 2 opcode {:04X} at {:#X}", inst.opcode, self.state.PC);

            self.state.PC += 2;
            let r = (inst.execute)(&mut self.state, inst.opcode);
            if r != 0 {
                ret = r;
                break;
            }
        }

        if ret > 1 {
            // Invalidate caches.
            let beg = addr_to_index((ret >> 16) as u16);
            let end = addr_to_index(ret as u16);
            for addr in beg..=end {
                self.interpreter_caches_2[addr] = None;
            }
        }

        self.handle_timers();
    }

    /// Creates a new cache at the current PC. The state is not modified.
    fn new_cache_block_2(block_pc: u16, memory: &[u8]) -> InstructionCache {
        let mut pc = block_pc;
        let mut instructions = Vec::new();

        'outer: loop {
            let opcode = Opcode((memory[pc as usize] as u16) << 8 | memory[pc as usize + 1] as u16);
            // #[cfg(debug_assertions)] println!("caching 2 opcode {opcode:04X} at {pc:#X}");
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

            if index_in_subcache(pc) == 0 {
                break 'outer;
            }
        }

        if instructions.is_empty() {
            pc -= 2;
            let opcode = (memory[pc as usize] as u16) << 8 | memory[pc as usize + 1] as u16;
            panic!("Unknown opcode {opcode:04X} at {pc:#X}");
        }

        InstructionCache {
            pc: block_pc,
            end_pc: pc,
            instructions,
        }
    }
}
