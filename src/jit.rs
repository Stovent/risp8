use crate::{Chip8, timer};
use crate::opcode::Opcode;
use crate::Address;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, x64::Assembler};

#[derive(Debug)]
pub enum Interrupts {
    UseInterpreter = 1,
    Jump = 2,
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
            2 => Self::Jump,
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
            Interrupts::Jump => self.PC = ret as u16,
        }
    }

    // TODO: for self-modifying code, store the memory range used by each cache, and when writing to this location,
    // invalidate the cache with an interrupt, perform the assignment in interpreter, then recompile the block.
    /// Uses the RAX, RCX and RDX (caller-saved) registers.
    ///
    /// EAX contains the return value of the block. RAX, RCX and RDX are used internally by the compiled code.
    fn compile_block(&mut self, mut pc: u16) {
        let block_pc = pc;
        let mut asm = Assembler::new().expect("Failed to create new assembler");

        let timer = timer as *const ();
        let this = self as *mut Chip8;
        dynasm!(asm
            ; .arch x64
            ; mov rax, QWORD timer as i64
            ; mov rcx, QWORD this as i64
            ; call rax
        );

        'outer: loop {
            let opcode = Opcode::from((self.memory[pc as usize] as u16) << 8 | self.memory[pc as usize + 1] as u16);

            #[cfg(debug_assertions)]
            println!("Compiling opcode {:#04X} at {:#X}", opcode, pc);

            match opcode.u16() >> 12 & 0xF {
                0x0 => match opcode.u16() {
                    0x00E0 => {
                        let addr_screen = self.screen.address(0) as i64;
                        dynasm!(asm
                            ; .arch x64
                            ; mov rdx, QWORD addr_screen
                            ; mov rax, rdx
                            ; add rax, 64 * 32
                            ; lbl:
                            ; mov QWORD [rdx], 0
                            ; add rdx, 8
                            ; cmp rdx, rax
                            ; jb <lbl
                        );
                    },
                    0x00EE => {
                        let sp = self.SP.address(0);
                        let stack = self.stack.address(0);
                        dynasm!(asm
                            ; .arch x64
                            ; mov rdx, QWORD sp as i64
                            ; cmp QWORD [rdx], 0
                            ; ja >lbl
                            ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                            ; ret
                            ; lbl:
                            ; dec QWORD [rdx]
                            ; mov rax, QWORD [rdx]
                            ; shl rax, 1
                            ; mov rcx, QWORD stack as i64
                            ; add rcx, rax
                            ; mov eax, DWORD Interrupts::make(Interrupts::Jump, 0) as i32
                            ; mov ax, WORD [rcx]
                        );
                        break 'outer;
                    },
                    _ => panic!("Unknown opcode {:04X}", opcode),
                },
                0x1 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, opcode.nnn()) as i32
                    );
                    break 'outer;
                },
                0x2 => {
                    let sp = self.SP.address(0);
                    let stack = self.stack.address(0);
                    let nnn = opcode.nnn();
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD sp as i64
                        ; mov rax, QWORD [rdx]
                        ; cmp rax, 15
                        ; jb >lbl
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, pc) as i32
                        ; ret
                        ; lbl:
                        ; shl rax, 1
                        ; mov rcx, QWORD stack as i64
                        ; add rcx, rax
                        ; mov WORD [rcx], (pc + 2) as i16
                        ; inc QWORD [rdx]
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, 0) as i32
                        ; mov ax, WORD nnn as i16
                    );
                    break 'outer;
                },
                0x3 => {
                    let (x, kk) = opcode.xkk();
                    let addrx = self.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addrx
                        ; mov al, BYTE [rdx]
                        ; cmp al, kk as i8
                        ; jne >lbl
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                        ; ret
                        ; lbl:
                    );
                },
                0x4 => {
                    let (x, kk) = opcode.xkk();
                    let addrx = self.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addrx
                        ; mov al, BYTE [rdx]
                        ; cmp al, kk as i8
                        ; je >lbl
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                        ; ret
                        ; lbl:
                    );
                },
                0x5 => {
                    let (x, y) = opcode.xy();
                    let addrx = self.V.address(x) as i64;
                    let addry = self.V.address(y) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addry
                        ; mov al, BYTE [rdx]
                        ; mov rdx, QWORD addrx
                        ; cmp BYTE [rdx], al
                        ; jne >lbl
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                        ; ret
                        ; lbl:
                    );
                },
                0x6 => {
                    let (x, kk) = opcode.xkk();
                    let addr = self.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov BYTE [rdx], kk as i8
                    );
                },
                0x7 => {
                    let (x, kk) = opcode.xkk();
                    let addr = self.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; add BYTE [rdx], kk as i8
                    );
                },
                0x8 => {
                    match opcode.u16() & 0xF00F {
                        0x8000 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0x8001 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; or BYTE [rdx], al
                            );
                        },
                        0x8002 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; and BYTE [rdx], al
                            );
                        },
                        0x8003 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addry
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; xor BYTE [rdx], al
                            );
                        },
                        0x8004 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
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
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
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
                            let x = opcode.x();
                            // let y = opcode.y();
                            let addrx = self.V.address(x) as i64;
                            // let addry = self.V.address(y) as i64;
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
                            let (x, y) = opcode.xy();
                            let addrx = self.V.address(x) as i64;
                            let addry = self.V.address(y) as i64;
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
                            let x = opcode.x();
                            // let y = opcode.y();
                            let addrx = self.V.address(x) as i64;
                            // let addry = self.V.address(y) as i64;
                            let addrf = self.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; shl BYTE [rdx], 1
                                ; mov rdx, QWORD addrf
                                ; setc BYTE [rdx]
                            );
                        },
                        _ => panic!("Unknown opcode {:04X}", opcode),
                    }
                },
                0x9 => {
                    let (x, y) = opcode.xy();
                    let addrx = self.V.address(x) as i64;
                    let addry = self.V.address(y) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addry
                        ; mov al, BYTE [rdx]
                        ; mov rdx, QWORD addrx
                        ; cmp BYTE [rdx], al
                        ; je >lbl
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                        ; ret
                        ; lbl:
                    );
                },
                0xA => {
                    let nnn = opcode.nnn();
                    let addr = self.I.address(0) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov WORD [rdx], nnn as i16
                    );
                },
                0xB => {
                    let nnn = opcode.nnn();
                    let addr0 = self.V.address(0) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::Jump, nnn) as i32
                        ; mov rdx, QWORD addr0
                        ; movzx edx, BYTE [rdx]
                        ; add eax, edx
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
                    match opcode.u16() & 0xF0FF {
                        0xE09E => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addr_keys = self.keys.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx rax, BYTE [rdx]
                                ; mov rdx, QWORD addr_keys
                                ; add rdx, rax
                                ; mov al, BYTE [rdx]
                                ; cmp al, 0
                                ; je >lbl
                                ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                                ; ret
                                ; lbl:
                            );
                        },
                        0xE0A1 => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addr_keys = self.keys.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx rax, BYTE [rdx]
                                ; mov rdx, QWORD addr_keys
                                ; add rdx, rax
                                ; mov al, BYTE [rdx]
                                ; cmp al, 0
                                ; jne >lbl
                                ; mov eax, DWORD Interrupts::make(Interrupts::Jump, pc + 4) as i32
                                ; ret
                                ; lbl:
                            );
                        },
                        _ => panic!("Unknown opcode {:04X}", opcode),
                    }
                },
                0xF => {
                    match opcode.u16() & 0xF0FF {
                        0xF007 => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addrdt = self.delay.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrdt
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF00A => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addr_last_key = self.last_key.address(0) as i64;
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
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addrdt = self.delay.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrdt
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF018 => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addrsound = self.sound.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrsound
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF01E => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addri = self.I.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx ax, BYTE [rdx]
                                ; mov rdx, QWORD addri
                                ; add WORD [rdx], ax
                            );
                        },
                        0xF029 => {
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addri = self.I.address(0) as i64;
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
                            let x = opcode.x();
                            let addrx = self.V.address(x) as i64;
                            let addri = self.I.address(0) as i64;
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
                            let x = opcode.x();
                            let addr0 = self.V.address(0) as i64;
                            let addrlast = self.V.address(x) as i64;
                            let addri = self.I.address(0) as i64;
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
                            let x = opcode.x();
                            let addr0 = self.V.address(0) as i64;
                            let addrlast = self.V.address(x) as i64;
                            let addri = self.I.address(0) as i64;
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
                        _ => panic!("Unknown opcode {:04X}", opcode),
                    }
                },
                _ => panic!("Unknown opcode {:04X}", opcode),
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
