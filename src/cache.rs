use crate::utils::*;

use dynasmrt::{dynasm, DynasmApi, x64::Assembler, AssemblyOffset, mmap::ExecutableBuffer};

pub struct Cache {
    pub pc: u16,
    pub exec: ExecutableBuffer,
}

impl Cache {
    pub fn new(pc: u16, exec: ExecutableBuffer) -> Self {
        log(format!("New cache at {:#X}", pc));
        Self {
            pc,
            exec,
        }
    }

    pub fn run(&mut self) -> u32 {
        // log(format!("Executing cache at {:#X} (size {}, {:?})", self.pc, self.exec.size(), self.exec.ptr(AssemblyOffset(0)) as *const u8));
        let mut caller = Assembler::new().expect("Failed to create new assembler");
        let func = self.exec.ptr(AssemblyOffset(0));
        let mut ret = 0;

        // Saves the caller-saved registers RAX and RDX. RAX/EAX is used for the return value. RDX is used by the compiled code.
        // Call the cached code.
        // Move the return value to the `ret` variable.
        // Restore the registers and lend control back to the function.
        dynasm!(caller
            ; .arch x64
            ; push rdx
            ; push rax
            ; mov rax, QWORD func as _
            ; call rax
            ; mov rdx, QWORD &mut ret as *mut u32 as _
            ; mov [rdx], eax
            ; pop rax
            ; pop rdx
            ; ret
        );

        unsafe {
            std::mem::transmute::<*const u8, fn() -> u32>(caller.finalize().unwrap().ptr(AssemblyOffset(0)))();
        }

        // log(format!("Cache execution returned with value {:#X}", ret));
        ret
    }
}

pub struct Caches {
    caches: Vec<Cache>,
}

impl Caches {
    pub fn new() -> Self {
        Self {
            caches: Vec::<Cache>::new(),
        }
    }

    pub fn get(&mut self, pc: u16) -> Option<&mut Cache> {
        self.caches.iter_mut().find(|el| el.pc == pc)
    }

    pub fn create(&mut self, pc: u16, exec: ExecutableBuffer) {
        self.caches.push(Cache::new(pc, exec));
    }
}
