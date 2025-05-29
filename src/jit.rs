use crate::Chip8;
use crate::utils::*;

use dynasmrt::{dynasm, DynasmApi, x64::Assembler};

#[derive(Debug)]
pub enum Interrupts {
    UseInterpreter = 1,
}

impl Interrupts {
    pub fn make(int: Interrupts, arg: u16) -> u32 {
        (int as u32) << 16 | arg as u32
    }
}

impl From<u32> for Interrupts {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::UseInterpreter,
            _ => panic!(),
        }
    }
}

impl Chip8 {
    /// Executes a block of instructions using the JIT compiler.
    pub fn jit(&mut self) {
        if self.caches.get(self.PC).is_none() {
            self.compile_block(self.PC);
        }

        let ret = self.caches.get(self.PC).unwrap().run();
        match Interrupts::from(ret >> 16) {
            Interrupts::UseInterpreter => {
                self.PC = ret as u16;
                self.interpreter();
            },
        }
    }

    // For self-modifying code, store the memory range used by each cache, and when writing to
    // this location, invalidate the cache with an interrupt, perform the assignment in interpreter, then recompile the block.
    /// Uses the RDX register.
    fn compile_block(&mut self, mut pc: u16) {
        let block_pc = pc;
        let mut asm = Assembler::new().expect("Failed to create new assembler");

        'outer: loop {
            let opcode: u16 = (self.memory[pc as usize] as u16) << 8 | self.memory[pc as usize + 1] as u16;

            log(format!("Compiling opcode {:#04X} at {:#X}", opcode, pc));

            match (opcode >> 12) & 0xF {
                0x0 => {
                    match opcode {
                        0x00E0 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0x00EE => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        _ => panic!("Unknown opcode {}", opcode),
                    }
                },
                0x1 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0x2 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0x3 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0x4 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0x5 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0x6 => {
                    let x = opcode >> 8 & 0xF;
                    let kk = opcode as i8;
                    let addr = self.V.address(x as isize) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov BYTE [rdx], kk
                    );
                },
                0x7 => {
                    let x = opcode >> 8 & 0xF;
                    let kk = opcode as i8;
                    let addr = self.V.address(x as isize) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; add BYTE [rdx], kk
                    );
                },
                0x8 => {
                    match opcode & 0xF00F {
                        0x8000 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0x8001 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; or BYTE [rdx], al
                            );
                        },
                        0x8002 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; and BYTE [rdx], al
                            );
                        },
                        0x8003 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; xor BYTE [rdx], al
                            );
                        },
                        0x8004 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0x8005 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0x8006 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0x8007 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0x800E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        _ => panic!("Unknown opcode {}", opcode),
                    }
                },
                0x9 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0xA => {
                    let nnn = opcode as i16 & 0x0FFF;
                    let addr = (&self.I).address(0) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov WORD [rdx], nnn
                    );
                },
                0xB => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0xC => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0xD => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                    );
                    break 'outer;
                },
                0xE => {
                    match opcode & 0xF0FF {
                        0xE09E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xE0A1 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        _ => panic!("Unknown opcode {}", opcode),
                    }
                },
                0xF => {
                    match opcode & 0xF0FF {
                        0xF007 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF00A => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF015 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF018 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF01E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF029 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF033 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF055 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        0xF065 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            );
                            break 'outer;
                        },
                        _ => panic!("Unknown opcode {}", opcode),
                    }
                },
                _ => panic!("Unknown opcode {}", opcode),
            };

            pc += 2;
        }

        dynasm!(asm
            ; .arch x64
            ; ret
        );

        self.caches.create(block_pc, asm.finalize().unwrap());
    }
}
