use crate::Chip8;
use crate::opcode::Opcode;
use crate::Address;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, x64::Assembler};

#[derive(Debug)]
pub enum Interrupts {
    UseInterpreter = 1,
    Jump = 2,
    InvalidateCache = 3,
}

impl Interrupts {
    pub fn use_interpreter(addr: u16) -> i64 {
        (Self::UseInterpreter as i64) << 16 | addr as i64
    }

    pub fn jump(addr: u16) -> i64 {
        (Self::Jump as i64) << 16 | addr as i64
    }

    pub fn invalidate(next_pc: u16, range_beg: u16, range_end: u16) -> i64 {
        (range_beg as i64) << 48 | (range_end as i64) << 32 | (Self::InvalidateCache as i64) << 16 | next_pc as i64
    }
}

impl From<u64> for Interrupts {
    fn from(i: u64) -> Self {
        match i {
            1 => Self::UseInterpreter,
            2 => Self::Jump,
            3 => Self::InvalidateCache,
            _ => panic!(),
        }
    }
}

impl Chip8 {
    /// Executes a block of instructions using the JIT compiler.
    pub(super) fn jit(&mut self) {
        if self.jit_caches.get(self.state.PC).is_none() {
            self.compile_block(self.state.PC);
        }

        let ret = self.jit_caches.get(self.state.PC).unwrap().run();
        match Interrupts::from(ret >> 16 & 0xFFFF) {
            Interrupts::UseInterpreter => {
                self.state.PC = ret as u16;
                self.interpreter();
            },
            Interrupts::Jump => self.state.PC = ret as u16,
            Interrupts::InvalidateCache => {
                self.state.PC = ret as u16;
                let beg_addr = (ret >> 48) as u16;
                let end_addr = (ret >> 32) as u16;

                self.jit_caches.invalidate(beg_addr, end_addr);
            },
        }
    }

    /// Uses the RAX, RCX and RDX (caller-saved) registers.
    ///
    /// RAX contains the return value of the block. RAX, RCX and RDX are used internally by the compiled code.
    fn compile_block(&mut self, addr: u16) {
        let block_pc = addr;
        let mut asm = Assembler::new().expect("Failed to create new assembler");

        let timer = handle_timers as *const ();
        let this = self as *mut Chip8;

        #[cfg(target_os = "windows")]
        dynasm!(asm
            ; .arch x64
            ; mov rax, QWORD timer as i64
            ; mov rcx, QWORD this as i64
            ; call rax
        );

        #[cfg(not(target_os = "windows"))]
        dynasm!(asm
            ; .arch x64
            ; mov rax, QWORD timer as i64
            ; push rdi
            ; mov rdi, QWORD this as i64
            ; call rax
            ; pop rdi
        );

        let mut next_pc = addr;
        'outer: loop {
            let current_pc = next_pc;
            let opcode = Opcode::from((self.state.memory[current_pc as usize] as u16) << 8 | self.state.memory[current_pc as usize + 1] as u16);
            next_pc += 2;

            // #[cfg(debug_assertions)] println!("Compiling opcode {opcode:#04X} at {current_pc:#X}");

            match opcode.0 >> 12 & 0xF {
                0x0 => match opcode.0 {
                    0x00E0 => {
                        let addr_screen = self.state.screen.address(0) as i64;
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
                        let sp = self.state.SP.address(0);
                        let stack = self.state.stack.address(0);
                        dynasm!(asm
                            ; .arch x64
                            ; mov rdx, QWORD sp as i64
                            ; cmp QWORD [rdx], 0
                            ; ja >lbl
                            ; mov rax, QWORD Interrupts::use_interpreter(current_pc)
                            ; ret
                            ; lbl:
                            ; dec QWORD [rdx]
                            ; mov rax, QWORD [rdx]
                            ; shl rax, 1
                            ; mov rcx, QWORD stack as i64
                            ; add rcx, rax
                            ; mov rax, QWORD Interrupts::jump(0)
                            ; mov ax, WORD [rcx]
                        );
                        break 'outer;
                    },
                    _ => panic!("Unknown opcode {opcode:04X}"),
                },
                0x1 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov rax, QWORD Interrupts::jump(opcode.nnn())
                    );
                    break 'outer;
                },
                0x2 => {
                    let sp = self.state.SP.address(0);
                    let stack = self.state.stack.address(0);
                    let nnn = opcode.nnn();
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD sp as i64
                        ; mov rax, QWORD [rdx]
                        ; cmp rax, 15
                        ; jb >lbl
                        ; mov rax, QWORD Interrupts::use_interpreter(current_pc)
                        ; ret
                        ; lbl:
                        ; shl rax, 1
                        ; mov rcx, QWORD stack as i64
                        ; add rcx, rax
                        ; mov WORD [rcx], (next_pc) as i16
                        ; inc QWORD [rdx]
                        ; mov rax, QWORD Interrupts::jump(0)
                        ; mov ax, WORD nnn as i16
                    );
                    break 'outer;
                },
                0x3 => {
                    let (x, kk) = opcode.xkk();
                    let addrx = self.state.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addrx
                        ; mov al, BYTE [rdx]
                        ; cmp al, kk as i8
                        ; jne >lbl
                        ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                        ; ret
                        ; lbl:
                    );
                },
                0x4 => {
                    let (x, kk) = opcode.xkk();
                    let addrx = self.state.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addrx
                        ; mov al, BYTE [rdx]
                        ; cmp al, kk as i8
                        ; je >lbl
                        ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                        ; ret
                        ; lbl:
                    );
                },
                0x5 => {
                    let (x, y) = opcode.xy();
                    let addrx = self.state.V.address(x) as i64;
                    let addry = self.state.V.address(y) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addry
                        ; mov al, BYTE [rdx]
                        ; mov rdx, QWORD addrx
                        ; cmp BYTE [rdx], al
                        ; jne >lbl
                        ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                        ; ret
                        ; lbl:
                    );
                },
                0x6 => {
                    let (x, kk) = opcode.xkk();
                    let addr = self.state.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov BYTE [rdx], kk as i8
                    );
                },
                0x7 => {
                    let (x, kk) = opcode.xkk();
                    let addr = self.state.V.address(x) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; add BYTE [rdx], kk as i8
                    );
                },
                0x8 => {
                    match opcode.0 & 0xF00F {
                        0x8000 => {
                            let (x, y) = opcode.xy();
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
                            let addrf = self.state.V.address(0xF) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
                            let addrf = self.state.V.address(0xF) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            // let addry = self.state.V.address(y) as i64;
                            let addrf = self.state.V.address(0xF) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addry = self.state.V.address(y) as i64;
                            let addrf = self.state.V.address(0xF) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            // let addry = self.state.V.address(y) as i64;
                            let addrf = self.state.V.address(0xF) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; shl BYTE [rdx], 1
                                ; mov rdx, QWORD addrf
                                ; setc BYTE [rdx]
                            );
                        },
                        _ => panic!("Unknown opcode {opcode:04X}"),
                    }
                },
                0x9 => {
                    let (x, y) = opcode.xy();
                    let addrx = self.state.V.address(x) as i64;
                    let addry = self.state.V.address(y) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addry
                        ; mov al, BYTE [rdx]
                        ; mov rdx, QWORD addrx
                        ; cmp BYTE [rdx], al
                        ; je >lbl
                        ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                        ; ret
                        ; lbl:
                    );
                },
                0xA => {
                    let nnn = opcode.nnn();
                    let addr = self.state.I.address(0) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rdx, QWORD addr
                        ; mov WORD [rdx], nnn as i16
                    );
                },
                0xB => {
                    let nnn = opcode.nnn();
                    let addr0 = self.state.V.address(0) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rax, QWORD Interrupts::jump(nnn)
                        ; mov rdx, QWORD addr0
                        ; movzx dx, BYTE [rdx]
                        ; add ax, dx
                    );
                    break 'outer;
                },
                0xC => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov rax, QWORD Interrupts::use_interpreter(current_pc)
                    );
                    break 'outer;
                },
                0xD => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov rax, QWORD Interrupts::use_interpreter(current_pc)
                    );
                    break 'outer;
                },
                0xE => {
                    match opcode.0 & 0xF0FF {
                        0xE09E => {
                            let x = opcode.x();
                            let addrx = self.state.V.address(x) as i64;
                            let addr_keys = self.state.keys.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx rax, BYTE [rdx]
                                ; mov rdx, QWORD addr_keys
                                ; add rdx, rax
                                ; mov al, BYTE [rdx]
                                ; cmp al, 0
                                ; je >lbl
                                ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                                ; ret
                                ; lbl:
                            );
                        },
                        0xE0A1 => {
                            let x = opcode.x();
                            let addrx = self.state.V.address(x) as i64;
                            let addr_keys = self.state.keys.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrx
                                ; movzx rax, BYTE [rdx]
                                ; mov rdx, QWORD addr_keys
                                ; add rdx, rax
                                ; mov al, BYTE [rdx]
                                ; cmp al, 0
                                ; jne >lbl
                                ; mov rax, QWORD Interrupts::jump(current_pc + 4)
                                ; ret
                                ; lbl:
                            );
                        },
                        _ => panic!("Unknown opcode {opcode:04X}"),
                    }
                },
                0xF => {
                    match opcode.0 & 0xF0FF {
                        0xF007 => {
                            let x = opcode.x();
                            let addrx = self.state.V.address(x) as i64;
                            let addrdt = self.state.delay.address(0) as i64;
                            dynasm!(asm
                                ; .arch x64
                                ; mov rdx, QWORD addrdt
                                ; mov al, BYTE [rdx]
                                ; mov rdx, QWORD addrx
                                ; mov BYTE [rdx], al
                            );
                        },
                        0xF00A => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov rax, QWORD Interrupts::use_interpreter(current_pc)
                            );
                            break 'outer;
                        },
                        0xF015 => {
                            let x = opcode.x();
                            let addrx = self.state.V.address(x) as i64;
                            let addrdt = self.state.delay.address(0) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addrsound = self.state.sound.address(0) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addri = self.state.I.address(0) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addri = self.state.I.address(0) as i64;
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
                            let addrx = self.state.V.address(x) as i64;
                            let addri = self.state.I.address(0) as i64;
                            let addrmem = self.state.memory.address(0) as i64;
                            let int_invalidate = Interrupts::invalidate as *const ();

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

                            #[cfg(target_os = "windows")]
                            dynasm!(asm
                                ; .arch x64
                                ; mov rcx, QWORD current_pc as i64 // Load current PC in rcx.
                                ; add rcx, 2 // Add 2 for next PC.
                                ; mov rdx, QWORD addri
                                ; movzx rdx, WORD [rdx] // Load begin address I in rdx.
                                ; mov r8, rdx
                                ; add r8, 2 // Load end address in r8.
                                ; mov rax, QWORD int_invalidate as i64
                                ; call rax
                            );

                            #[cfg(not(target_os = "windows"))]
                            dynasm!(asm
                                ; .arch x64
                                ; push rdi
                                ; push rsi
                                ; mov rdi, QWORD current_pc as i64 // Load current PC in rdi.
                                ; add rdi, 2 // Add 2 for next PC.
                                ; mov rsi, QWORD addri
                                ; movzx rsi, WORD [rsi] // Load begin address I in rsi.
                                ; mov rdx, rsi
                                ; add rdx, 2 // Load end address in rdx.
                                ; mov rax, QWORD int_invalidate as i64
                                ; call rax
                                ; pop rsi
                                ; pop rdi
                            );
                            break 'outer;
                        },
                        0xF055 => {
                            let x = opcode.x();
                            let addr0 = self.state.V.address(0) as i64;
                            let addrlast = self.state.V.address(x) as i64;
                            let addri = self.state.I.address(0) as i64;
                            let addrmem = self.state.memory.address(0) as i64;
                            let int_invalidate = Interrupts::invalidate as *const ();

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

                            #[cfg(target_os = "windows")]
                            dynasm!(asm
                                ; .arch x64
                                ; mov rcx, QWORD current_pc as i64 // Load current PC in rcx.
                                ; add rcx, 2 // Add 2 for next PC.
                                ; mov rdx, QWORD addri
                                ; movzx rdx, WORD [rdx] // Load begin address I in rdx.
                                ; mov r8, rdx
                                ; add r8, x as i32 // Load end address in r8.
                                ; mov rax, QWORD int_invalidate as i64
                                ; call rax
                            );

                            #[cfg(not(target_os = "windows"))]
                            dynasm!(asm
                                ; .arch x64
                                ; push rdi
                                ; push rsi
                                ; mov rdi, QWORD current_pc as i64 // Load current PC in rdi.
                                ; add rdi, 2 // Add 2 for next PC.
                                ; mov rsi, QWORD addri
                                ; movzx rsi, WORD [rsi] // Load begin address I in rsi.
                                ; mov rdx, rsi
                                ; add rdx, x as i32 // Load end address in rdx.
                                ; mov rax, QWORD int_invalidate as i64
                                ; call rax
                                ; pop rsi
                                ; pop rdi
                            );
                            break 'outer;
                        },
                        0xF065 => {
                            let x = opcode.x();
                            let addr0 = self.state.V.address(0) as i64;
                            let addrlast = self.state.V.address(x) as i64;
                            let addri = self.state.I.address(0) as i64;
                            let addrmem = self.state.memory.address(0) as i64;
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
                        _ => panic!("Unknown opcode {opcode:04X}"),
                    }
                },
                _ => panic!("Unknown opcode {opcode:04X}"),
            };
        }

        dynasm!(asm
            ; .arch x64
            ; ret
        );

        self.jit_caches.add(block_pc, next_pc, asm.finalize().unwrap());
    }
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
extern "win64" fn handle_timers(this: &mut Chip8) {
    this.handle_timers();
}

#[cfg(all(not(target_os = "windows"), target_arch = "x86_64"))]
extern "sysv64" fn handle_timers(this: &mut Chip8) {
    this.handle_timers();
}
