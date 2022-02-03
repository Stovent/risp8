use crate::utils::Address;

use dynasmrt::{dynasm, DynasmApi, x64::Assembler, AssemblyOffset, mmap::ExecutableBuffer};

pub struct Cache {
    pub pc: u16,
    called_buf: ExecutableBuffer,
    caller_buf: ExecutableBuffer,
    caller: fn(),
}

static mut RET: u32 = 0;

impl Cache {
    pub fn new(pc: u16, called_buf: ExecutableBuffer) -> Self {
        let mut caller_buf = Assembler::new().expect("Failed to create new assembler");
        let called = called_buf.ptr(AssemblyOffset(0));

        fn dummy() {}
        let mut cache = Self {
            pc,
            called_buf,
            caller_buf: Assembler::new().expect("Dummy assembler").finalize().unwrap(),
            caller: dummy,
        };

        #[cfg(debug_assertions)] println!("New cache at {:#X} (size {}, {:?})", pc, cache.called_buf.size(), called);

        // Saves the caller-saved registers RAX and RDX.
        // Call the cached code.
        // Move the return value to the `ret` variable.
        // Restore the registers and lend control back to the function.
        unsafe {
            dynasm!(caller_buf
                ; .arch x64
                ; push rdx
                ; push rcx
                ; push rax
                ; mov rax, QWORD called as i64
                ; call rax
                ; mov rdx, QWORD RET.address(0) as i64
                ; mov DWORD [rdx], eax
                ; pop rax
                ; pop rcx
                ; pop rdx
                ; ret
            );
        }

        cache.caller_buf = caller_buf.finalize().unwrap();
        unsafe {
            cache.caller = std::mem::transmute::<*const u8, fn()>(cache.caller_buf.ptr(AssemblyOffset(0)))
        }

        cache
    }

    pub fn run(&mut self) -> u32 {
        #[cfg(debug_assertions)] println!("Executing cache at {:#X}", self.pc);
        (self.caller)();
        #[cfg(debug_assertions)] println!("Cache execution returned with value {:#X}", unsafe { RET });
        unsafe { RET }
    }
}

pub struct Caches {
    caches: Vec<Cache>,
}

impl Caches {
    pub fn new() -> Self {
        Self {
            caches: Vec::new(),
        }
    }

    pub fn add(&mut self, pc: u16, exec: ExecutableBuffer) {
        self.caches.push(Cache::new(pc, exec));
    }

    pub fn get(&mut self, pc: u16) -> Option<&mut Cache> {
        self.caches.iter_mut().find(|el| el.pc == pc)
    }
}
