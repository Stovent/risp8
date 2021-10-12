use crate::utils::*;

use dynasmrt::{AssemblyOffset, mmap::ExecutableBuffer};

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
        log(format!("Executing cache at {:#X} (size {}, {:?})", self.pc, self.exec.size(), self.exec.ptr(AssemblyOffset(0)) as *const u8));
        unsafe {
            // breakpoint();
            let ret = std::mem::transmute::<*const u8, fn() -> u32>(self.exec.ptr(AssemblyOffset(0)))();
            log(format!("Cache execution returned with value {:#X}", ret));
            ret
        }
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

    pub fn get_or_create(&mut self, pc: u16, exec: ExecutableBuffer) -> &mut Cache {
        // TODO: remove unsafe when new borrow checker is available.
        unsafe {
            let self1 = (self as *mut Self).as_mut().unwrap();
            if let Some(cache) = self.get(pc) {
                cache
            } else {
                self1.create(pc, exec);
                self1.caches.last_mut().unwrap()
            }
        }
    }

    pub fn create(&mut self, pc: u16, exec: ExecutableBuffer) {
        self.caches.push(Cache::new(pc, exec));
    }
}
