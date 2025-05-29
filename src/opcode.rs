#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode(u16);

impl Opcode {
    pub fn u16(self) -> u16 {
        self.0
    }

    pub fn x(self) -> usize {
        self.0 as usize >> 8 & 0xF
    }

    fn y(self) -> usize {
        self.0 as usize >> 4 & 0xF
    }

    pub fn xy(self) -> (usize, usize) {
        (self.x(), self.y())
    }

    fn kk(self) -> u8 {
        self.0 as u8
    }

    pub fn xkk(self) -> (usize, u8) {
        (self.x(), self.kk())
    }

    pub fn n(self) -> u8 {
        self.0 as u8 & 0xF
    }

    pub fn nnn(self) -> u16 {
        self.0 & 0xFFF
    }
}

impl From<u16> for Opcode {
    fn from(opcode: u16) -> Self {
        Opcode(opcode)
    }
}

impl std::fmt::UpperHex for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}
