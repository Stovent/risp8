use crate::utils::*;

#[derive(Debug)]
pub enum X86Reg {
    EAX = 0,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
}

pub trait ICache {
    fn push_8(&mut self, _: u8);
    /// Little-endian
    fn push_32(&mut self, _: u32);
}

pub trait X86Emitter<T: ICache> : ICache {
    fn add_mem_imm8(&mut self, addr: u32, imm: u8) {
        log(format!("add [{:#X}], {}", addr, imm));
        self.push_8(0x80);
        self.push_8(0x05);
        self.push_32(addr);
        self.push_8(imm);
    }

    fn mov_mem_imm8(&mut self, addr: u32, imm: u8) {
        log(format!("mov [{:#X}], {}", addr, imm));
        self.push_8(0xC6);
        self.push_8(0x05);
        self.push_32(addr);
        self.push_8(imm);
    }

    fn mov_reg_imm32(&mut self, reg: X86Reg, value: u32) {
        log(format!("mov {:?}, {:#X}", reg, value));
        self.push_8(0xB8 + reg as u8);
        self.push_32(value);
    }

    fn mov_mem_eax(&mut self, addr: u32) {
        log(format!("mov [{:#X}], eax", addr));
        self.push_8(0xA3);
        self.push_32(addr);
    }

    fn mov_eax_mem(&mut self, addr: u32) {
        log(format!("mov eax, [{:#X}]", addr));
        self.push_8(0xA1);
        self.push_32(addr);
    }

    fn ret(&mut self, value: u32) {
        self.mov_reg_imm32(X86Reg::EAX, value);
        log_str("ret");
        self.push_8(0xC3);
    }
}
