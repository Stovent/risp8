use crate::utils::*;
use crate::x86::*;

use memmap::MmapMut;

pub struct Cache {
    pc: u16,
    code: Vec<u8>,
}

impl Cache {
    pub fn new(pc: u16) -> Self {
        log(format!("New cache at {:#X}", pc));
        Self {
            pc,
            code: Vec::<u8>::new(),
        }
    }

    pub fn run(&mut self) -> u32 {
        log(format!("Executing cache at {:#X} (size {}, {:?})", self.pc, self.code.len(), &self.code[0] as *const u8));
        unsafe {
            let mut code = MmapMut::map_anon(self.code.len()).expect("Failed to map cache.");
            std::ptr::copy(self.code.as_ptr(), code.as_mut_ptr(), self.code.len());
            let code = code.make_exec().expect("Failed to make executable buffer");
            // breakpoint();
            let ret = std::mem::transmute::<*const u8, fn() -> u32>(code.as_ptr())();
            log(format!("Cache execution returned with value {:#X}", ret));
            ret
        }
    }
}

impl ICache for Cache {
    fn push_8(&mut self, d: u8) {
        self.code.push(d);
    }

    fn push_32(&mut self, d: u32) {
        self.push_8(d as u8);
        self.push_8((d >> 8) as u8);
        self.push_8((d >> 16) as u8);
        self.push_8((d >> 24) as u8);
    }
}

impl X86Emitter<Cache> for Cache {}

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

    pub fn get_or_create(&mut self, pc: u16) -> &mut Cache {
        // TODO: remove unsafe when new borrow checker is available.
        unsafe {
            let self1 = (self as *mut Self).as_mut().unwrap();
            if let Some(cache) = self.get(pc) {
                cache
            } else {
                self1.create(pc);
                self1.caches.last_mut().unwrap()
            }
        }
    }

    pub fn create(&mut self, pc: u16) {
        self.caches.push(Cache::new(pc));
    }
}
