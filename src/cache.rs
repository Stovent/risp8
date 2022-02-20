use dynasmrt::{dynasm, DynasmApi, x64::Assembler, AssemblyOffset, mmap::ExecutableBuffer};

pub struct Cache {
    pub pc: u16,
    _called_buf: ExecutableBuffer, // Store it so its memory isn't freed.
    caller_buf: ExecutableBuffer, // Store it so its memory isn't freed.
    caller: extern "win64" fn(&mut u32),
}

impl Cache {
    pub fn new(pc: u16, called_buf: ExecutableBuffer) -> Self {
        let mut caller_buf = Assembler::new().expect("Failed to create new assembler");
        let called = called_buf.ptr(AssemblyOffset(0));

        extern "win64" fn dummy(_: &mut u32) {}
        let mut cache = Self {
            pc,
            _called_buf: called_buf,
            caller_buf: Assembler::new().expect("Dummy assembler").finalize().unwrap(),
            caller: dummy,
        };

        #[cfg(debug_assertions)] println!("New cache at {:#X} (size {}, {:?})", pc, cache._called_buf.size(), called);

        // Saves the caller-saved registers RAX, RCX and RDX.
        // Call the cached code.
        // Move the return value to the `ret` variable.
        // Restore the registers and lend control back to the function.
        dynasm!(caller_buf
            ; .arch x64
            ; push rax
            ; push rcx
            ; push rdx
            ; mov rax, QWORD called as i64
            ; call rax
            ; pop rdx
            ; pop rcx
            ; mov DWORD [rcx], eax
            ; pop rax
            ; ret
        );

        cache.caller_buf = caller_buf.finalize().unwrap();
        unsafe {
            cache.caller = std::mem::transmute::<*const u8, extern "win64" fn(&mut u32)>(cache.caller_buf.ptr(AssemblyOffset(0)))
        }

        cache
    }

    pub fn run(&mut self) -> u32 {
        #[cfg(debug_assertions)] println!("Executing cache at {:#X}", self.pc);
        let mut ret = 0;
        (self.caller)(&mut ret);
        #[cfg(debug_assertions)] println!("Cache execution returned with value {:#X}", ret);
        ret
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
