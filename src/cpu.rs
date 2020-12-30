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