use crate::mmu::Mmu;

#[derive(Debug, Default)]
pub struct R16 {
    h: u8, // High Byte
    l: u8, // Low Byte
}

#[allow(dead_code)]
impl R16 {
    fn zero() -> R16 {
        R16::default()
    }

    fn new(h: u8, l: u8) -> R16 {
        R16 {
            h: h,
            l: l
        }
    }

    fn dump(self, h: &mut u8, l: &mut u8) {
        *h = self.h;
        *l = self.l;
    }
}

impl From<u16> for R16 {
    fn from(value: u16) -> Self {
        R16 {
            h: (value >> 8) as u8,
            l: (value) as u8
        }
    }
}

impl Into<u16> for R16 {
    fn into(self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }
}

impl PartialEq for R16 {
    fn eq(&self, other: &Self) -> bool {
        self.h == other.h && self.l == other.l
    }
}

#[test]
fn r16_cast_tests() {
    assert_eq!(0x0000u16, R16::default().into());
    assert_eq!(0x0000u16, R16::zero().into());
    assert_eq!(0x8844u16, R16::new(0x88u8, 0x44u8).into());
    assert_eq!(0x8844u16, R16::from(0x8844u16).into());

    assert_eq!(R16::new(0xdeu8, 0xadu8), R16::from(0xdeadu16));
    assert_eq!(R16::zero(), R16::from(0u16));
}


bitflags! {
    struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Flags::from_bits(value).unwrap()
    }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 {
        self.bits
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Cpu {
    a: u8,    // accumulator
    f: u8,    // flags
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp : u16,    // stack pointer
    pc : u16,    // program counter
    clock: u64,  // accumulated clock counter
    iflag: bool, // interrupt flag
}

const INSTRUCTION_SIZE: [u8; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1,    // 0x00 ~ 0x0F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x10 ~ 0x1F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x20 ~ 0x2F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x30 ~ 0x3F

    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x40 ~ 0x4F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x50 ~ 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x60 ~ 0x6F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x70 ~ 0x7F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x80 ~ 0x8F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x90 ~ 0x9F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xA0 ~ 0xAF
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xB0 ~ 0xBF

    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 2, 3, 3, 2, 1,    // 0xC0 ~ 0xCF
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1,    // 0xD0 ~ 0xDF
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xE0 ~ 0xEF
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xF0 ~ 0xFF
];

const INSTRUCTION_TICKS: [u8; 256] = [
//  x0  x1  x2  x3  x4  x5  x6  x7  x8  x9  xA  xB  xC  xD  xE  xF
    4,  12, 8,  8,  4,  4,  8,  4,  20, 8,  8,  8,  4,  4,  8,  4,    // 0x00 ~ 0x0F
    4,  12, 8,  8,  4,  4,  8,  4,  12, 8,  8,  8,  4,  4,  8,  4,    // 0x10 ~ 0x1F
    8,  12, 8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x20 ~ 0x2F
    8,  12, 8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x30 ~ 0x3F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x40 ~ 0x4F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x50 ~ 0x5F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x60 ~ 0x6F
    8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x70 ~ 0x7F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x80 ~ 0x8F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x90 ~ 0x9F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xA0 ~ 0xAF
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xB0 ~ 0xBF

    8,  12, 12, 16, 12, 16, 8,  16, 8,  16, 12, 8,  12, 24, 8,  16,    // 0xC0 ~ 0xCF
    8,  12, 12, 4,  12, 16, 8,  16, 8,  16, 12, 4,  12, 4,  8,  16,    // 0xD0 ~ 0xDF
    12, 12, 8,  4,  4,  16, 8,  16, 16, 4,  16, 4,  4,  4,  8,  16,    // 0xE0 ~ 0xEF
    12, 12, 8,  4,  4,  16, 8,  16, 12, 8,  16, 4,  4,  4,  8,  16,    // 0xF0 ~ 0xFF
];

#[allow(dead_code)]
impl Cpu {
    fn cycle(&mut self, mmu: &mut dyn Mmu) -> u64 {
        let opcode = mmu.read_byte(self.pc);
        match opcode {
            0x00 => self.opcode00(mmu),
            0x01 => self.opcode01(mmu),
            _ => panic!("Not implemented")
        }

        self.pc += INSTRUCTION_SIZE[opcode as usize] as u16;
        INSTRUCTION_TICKS[opcode as usize].into()
    }

    fn opcode00(&mut self, _mmu: &mut dyn Mmu) {
        // NOP
    }

    fn opcode01(&mut self, _mmu: &mut dyn Mmu) {
        // LD BC, d16
    }
}