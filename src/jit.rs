use crate::Chip8;
use crate::utils::*;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, x64::Assembler};

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
    /// Uses the RAX, RCX and RDX (caller-saved) registers.
    ///
    /// EAX contains the return value of the block. RAX, RCX and RDX are used internally by the compiled code.
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
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; add BYTE [rdx], al
                                ; mov rdx, QWORD addrf
                                ; setc BYTE [rdx]
                            );
                        },
                        0x8005 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; sub BYTE [rdx], al
                                ; mov rdx, QWORD addrf
                                ; setnc BYTE [rdx]
                            );
                        },
                        0x8006 => {
                            let x = opcode >> 8 & 0xF;
                            // let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            // let addry = self.V.address(y as isize) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; shr BYTE [rdx], 1
                                ; mov rdx, QWORD addrf
                                ; setc BYTE [rdx]
                            );
                        },
                        0x8007 => {
                            let x = opcode >> 8 & 0xF;
                            let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addry = self.V.address(y as isize) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov ah, BYTE [rdx]
                                ; sub al, ah
                                ; mov BYTE [rdx], al
                                ; mov rdx, QWORD addrf
                                ; setnc BYTE [rdx]
                            );
                        },
                        0x800E => {
                            let x = opcode >> 8 & 0xF;
                            // let y = opcode >> 4 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            // let addry = self.V.address(y as isize) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; shl BYTE [rdx], 1
                                ; mov rdx, QWORD addrf
                                ; setc BYTE [rdx]
                            );
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
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addrdt = (&self.delay).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrdt
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF00A => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addr_last_key = (&self.last_key).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addr_last_key
                                ; mov BYTE [rdx], 255 as _
                                ; lbl:
                                ; cmp BYTE [rdx], 15
                                ; ja <lbl
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF015 => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addrdt = (&self.delay).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrdt
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF018 => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addrsound = (&self.sound).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrsound
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF01E => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addri = (&self.I).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx ax, BYTE [rdx]
                                ; mov rdx, QWORD addri
                                ; add WORD [rdx], ax
                            );
                        },
                        0xF029 => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addri = (&self.I).address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; mov al, BYTE [rdx]
                                ; mov dl, 5
                                ; mul dl
                                ; mov rdx, QWORD addri
                                ; mov WORD [rdx], ax
                            );
                        },
                        0xF033 => {
                            let x = opcode >> 8 & 0xF;
                            let addrx = self.V.address(x as isize) as i64;
                            let addri = (&self.I).address(0) as i64;
                            let addrmem = self.memory.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addri
                                ; movzx rdx, WORD [rdx]
                                ; mov rax, QWORD addrmem
                                ; add rdx, rax
                                ; mov rax, QWORD addrx
                                ; movzx ax, BYTE [rax]
                                ; mov cl, 100
                                ; div cl
                                ; mov BYTE [rdx], al
                                ; movzx ax, ah
                                ; mov cl, 10
                                ; div cl
                                ; mov BYTE [rdx + 1], al
                                ; mov BYTE [rdx + 2], ah
                            );
                        },
                        0xF055 => {
                            let x = opcode >> 8 & 0xF;
                            let addr0 = self.V.address(0) as i64;
                            let addrlast = self.V.address(x as isize) as i64;
                            let addri = (&self.I).address(0) as i64;
                            let addrmem = self.memory.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addri
                                ; movzx rdx, WORD [rdx]
                                ; mov rax, QWORD addrmem
                                ; add rdx, rax
                                ; mov rax, QWORD addr0
                                ; lbl:
                                ; mov cl, BYTE [rax]
                                ; mov BYTE [rdx], cl
                                ; mov rcx, QWORD addrlast
                                ; cmp rax, rcx
                                ; jae >end
                                ; inc rax
                                ; inc rdx
                                ; jmp <lbl
                                ; end:
                            );
                        },
                        0xF065 => {
                            let x = opcode >> 8 & 0xF;
                            let addr0 = self.V.address(0) as i64;
                            let addrlast = self.V.address(x as isize) as i64;
                            let addri = (&self.I).address(0) as i64;
                            let addrmem = self.memory.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addri
                                ; movzx rdx, WORD [rdx]
                                ; mov rax, QWORD addrmem
                                ; add rdx, rax
                                ; mov rax, QWORD addr0
                                ; lbl:
                                ; mov cl, BYTE [rdx]
                                ; mov BYTE [rax], cl
                                ; mov rcx, QWORD addrlast
                                ; cmp rax, rcx
                                ; jae >end
                                ; inc rax
                                ; inc rdx
                                ; jmp <lbl
                                ; end:
                            );
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

        self.caches.add(block_pc, asm.finalize().unwrap());
    }
}
