//! Cached interpreter, idea 3.
//!
//! This cached interpreter is the simplest one: it maintains a look-up table [Chip8::interpreter_caches_3] indexed
//! using the instuction's address `(self.state.PC - Self::INITIAL_PC) as usize >> 1` and executing what's there if
//! present, or decode and cache the instruction to be executed.
//!
//! The advantages are a O(1) instruction look-up and cache invalidation, as self-modifying code only invalidates
//! the modified instructions and does not delete what shouldn't be, and no dynamic allocation nor bump allocator
//! except for the initial array.
//!
//! In risp8 this method is slightly less efficient that the interpreter because reading in the array and checking if the
//! instruction is already cached takes longer than the LUT decoding of the interpreter.
//!
//! The purpose of this method is to be a POC to implement it in other architectures where instruction decoding may be
//! slower than this array check.

use crate::{
    Chip8,
    opcode::Opcode,
    State,
    cached_interpreter::{
        addr_to_index,
        CachedInstruction,
    },
};

impl Chip8 {
    pub(super) fn cached_interpreter_3(&mut self) {
        self.handle_timers();

        let cache_index = addr_to_index(self.state.PC);
        let pc = self.state.PC;
        self.state.PC += 2;

        let ret = if let Some(inst) = self.interpreter_caches_3[cache_index] {
            // #[cfg(debug_assertions)] println!("cached 3 opcode {:04X} at {pc:#X}", inst.opcode);
            (inst.execute)(&mut self.state, inst.opcode)
        } else {
            let opcode = Opcode((self.state.memory[pc as usize] as u16) << 8 | self.state.memory[pc as usize + 1] as u16);
            let execute = State::ILUT[opcode.0 as usize];
            self.interpreter_caches_3[cache_index] = Some(CachedInstruction {
                opcode,
                execute,
            });

            // #[cfg(debug_assertions)] println!("caching 3 opcode {opcode:04X} at {pc:#X}");
            (execute)(&mut self.state, opcode)
        };

        if ret > 1 {
            // Invalidate caches.
            let beg = addr_to_index((ret >> 16) as u16);
            let end = addr_to_index(ret as u16);
            for addr in beg..=end {
                self.interpreter_caches_3[addr as usize] = None;
            }
        }
    }
}
