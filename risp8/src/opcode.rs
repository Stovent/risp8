#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Opcode(pub u16);

impl Opcode {
    #[inline(always)]
    pub const fn x(self) -> usize {
        self.0 as usize >> 8 & 0xF
    }

    #[inline(always)]
    const fn y(self) -> usize {
        self.0 as usize >> 4 & 0xF
    }

    #[inline(always)]
    pub const fn xy(self) -> (usize, usize) {
        (self.x(), self.y())
    }

    #[inline(always)]
    const fn kk(self) -> u8 {
        self.0 as u8
    }

    #[inline(always)]
    pub const fn xkk(self) -> (usize, u8) {
        (self.x(), self.kk())
    }

    #[inline(always)]
    pub const fn n(self) -> u8 {
        self.0 as u8 & 0xF
    }

    #[inline(always)]
    pub const fn nnn(self) -> u16 {
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
