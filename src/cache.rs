use dynasmrt::{dynasm, DynasmApi, x64::Assembler, AssemblyOffset, mmap::ExecutableBuffer};

pub struct Cache {
    pc: u16,
    /// The address of the instruction following the last instruction in this cache [pc, end_pc).
    end_pc: u16,
    _called_buf: ExecutableBuffer, // Store it so its memory isn't freed.
    _caller_buf: ExecutableBuffer, // Store it so its memory isn't freed.
    caller: extern "win64" fn(&mut u64),
}

impl Cache {
    pub fn new(pc: u16, end_pc: u16, called_buf: ExecutableBuffer) -> Self {
        let mut caller_asm = Assembler::new().expect("Failed to create new assembler");
        let called = called_buf.ptr(AssemblyOffset(0));

        // #[cfg(debug_assertions)] println!("New cache at {pc:#X} (end {end_pc:#X}, {called:?})");

        dynasm!(caller_asm
            ; .arch x64
            ; push rax // Saves the caller-saved registers RAX, RCX and RDX.
            ; push rcx
            ; push rdx
            ; mov rax, QWORD called as i64 // Call the cached code.
            ; call rax
            ; pop rdx// Restore the registers and lend control back to the function.
            ; pop rcx
            ; mov QWORD [rcx], rax // Move the return value to the `ret` variable.
            ; pop rax
            ; ret
        );

        let _caller_buf = caller_asm.finalize().unwrap();
        let caller = unsafe {
            std::mem::transmute::<*const u8, extern "win64" fn(&mut u64)>(_caller_buf.ptr(AssemblyOffset(0)))
        };

        Self {
            pc,
            end_pc,
            _called_buf: called_buf,
            _caller_buf,
            caller,
        }
    }

    pub fn run(&self) -> u64 {
        // #[cfg(debug_assertions)] println!("Executing cache at {:#X}", self.pc);
        let mut ret = 0;
        (self.caller)(&mut ret);
        // #[cfg(debug_assertions)] println!("Cache execution returned with value {:#X}", ret);
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

    pub fn add(&mut self, pc: u16, end_pc: u16, exec: ExecutableBuffer) {
        self.caches.push(Cache::new(pc, end_pc, exec));
    }

    pub fn get(&self, pc: u16) -> Option<&Cache> {
        self.caches.iter().find(|cache| cache.pc == pc)
    }

    /// Deletes all the caches that contain the given address range. end_addr inclusive.
    pub fn invalidate(&mut self, beg_addr: u16, end_addr: u16) {
        let _: Vec<_> = self.caches.extract_if(|cache| {
            beg_addr >= cache.pc && beg_addr < cache.end_pc ||
            end_addr >= cache.pc && end_addr < cache.end_pc
        }).collect();
    }
}
