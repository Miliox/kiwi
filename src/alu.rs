bitflags! {
    pub struct Flags: u8 {
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


#[derive(Debug, Default)]
pub struct R16 {
    pub h: u8, // High Byte
    pub l: u8, // Low Byte
}

#[allow(dead_code)]
impl R16 {
    pub fn zero() -> R16 {
        R16::default()
    }

    pub fn new(h: u8, l: u8) -> R16 {
        R16 {
            h: h,
            l: l
        }
    }

    pub fn dump(self, h: &mut u8, l: &mut u8) {
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

pub fn decrement_8bit_register(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    if *r & 0xf != 0 {
        flags = flags | Flags::H;
    }
    if *r == 0x01 {
        flags = flags | Flags::Z;
    }
    flags = flags | Flags::N;
    *f = flags.into();
    *r = r.wrapping_add(1);  
}

pub fn decrement_16bit_register(h: &mut u8, l: &mut u8) {
    if *l == 0 {
        *h = h.wrapping_sub(1);
    }
    *l = l.wrapping_sub(1);
}

pub fn increment_8bit_register(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    if *r == 0xff {
        flags = flags | Flags::Z;
    }
    flags = flags - Flags::N;
    if *r & 0xf == 0xf {
        flags = flags | Flags::H;
    }
    *f = flags.into();
    *r = r.wrapping_add(1);
}

pub fn increment_16bit_register(h: &mut u8, l: &mut u8) {
    *l = l.wrapping_add(1);
    if *l == 0 {
        *h = h.wrapping_add(1);
    }
}

pub fn rotate_left_and_store_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags = flags - Flags::Z;
    flags = flags - Flags::N;
    flags = flags - Flags::H;
    if *r & 0x80 != 0 {
        flags = flags | Flags::C;
    } else {
        flags = flags - Flags::C;
    }
    *f = flags.into();
    *r = r.rotate_left(1);
}

pub fn rotate_left_through_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    let carry = flags.contains(Flags::C);

    flags = flags - Flags::Z;
    flags = flags - Flags::N;
    flags = flags - Flags::H;
    if *r & 0x80 != 0 {
        flags = flags | Flags::C;
    } else {
        flags = flags - Flags::C;
    }
    *f = flags.into();

    *r = r.rotate_left(1);
    if carry {
        *r = *r | 0b0000_0001;
    } else {
        *r = *r & 0b1111_1110;
    }
}

pub fn rotate_right_and_store_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags = flags - Flags::Z;
    flags = flags - Flags::N;
    flags = flags - Flags::H;
    if *r & 0x01 != 0 {
        flags = flags | Flags::C;
    } else {
        flags = flags - Flags::C;
    }
    *f = flags.into();
    *r = r.rotate_right(1);
}

pub fn rotate_right_through_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    let carry = flags.contains(Flags::C);

    flags = flags - Flags::Z;
    flags = flags - Flags::N;
    flags = flags - Flags::H;
    if *r & 0x01 != 0 {
        flags = flags | Flags::C;
    } else {
        flags = flags - Flags::C;
    }
    *f = flags.into();

    *r = r.rotate_left(1);
    if carry {
        *r = *r | 0b1000_0000;
    } else {
        *r = *r & 0b0111_1111;
    }
}

pub fn decimal_adjust(r: &mut u8, f: &mut u8) {
    // https://ehaskins.com/2018-01-30%20Z80%20DAA/
    let mut correction = 0;

    let mut flags = Flags::from(*f);
    if flags.contains(Flags::H) || (!flags.contains(Flags::N) && (*r & 0xf) > 9) {
      correction |= 0x6;
    }

    if flags.contains(Flags::C) || (!flags.contains(Flags::N) && *r > 0x99) {
      correction |= 0x60;
      flags.insert(Flags::C);
    }
  
    if flags.contains(Flags::N) {
        *r = r.wrapping_sub(correction);
    } else {
        *r = r.wrapping_add(correction);
    }

    if *r == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    flags.remove(Flags::H);

    *f = flags.into();
}

pub fn complement(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.insert(Flags::N | Flags::H);
    *f = flags.into();

    *r = !*r;
}

pub fn add_8bit_registers(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);

    if ((*r1 & 0xf) + (r2 & 0xf)) & 0x10 != 0 {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }

    if ((*r1 as u16) + (r2 as u16)) & 0x100 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }

    *r1 = r1.wrapping_add(r2);

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}

pub fn add_8bit_registers_with_carry(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);

    let carry = if flags.contains(Flags::C) { 1 } else { 0 };

    if ((*r1 & 0xf) + (r2 & 0xf) + carry) & 0x10 != 0 {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }

    if (*r1 as u16 + r2 as u16 + carry as u16) & 0x100 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }

    *r1 = r1.wrapping_add(r2);
    *r1 = r1.wrapping_add(carry);

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}

pub fn add_16bit_registers(h1: &mut u8, l1: &mut u8, h2: u8, l2: u8, f: &mut u8) {
    let r1 = R16::new(*h1, *l1);
    let r2 = R16::new(h2, l2);
    let v1: u16 = r1.into();
    let v2: u16 = r2.into();

    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);
    if (*l1 as u16 + l2 as u16) & 0x100u16 != 0u16 {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }
    if (v1 as u32 + v2 as u32) & 0x10000u32 != 0u32 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
    *f = flags.into();

    let total = v1.wrapping_add(v2);
    R16::from(total).dump(h1, l1);
}

pub fn sub_8bit_registers(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.insert(Flags::N);

    if (*r1 & 0xf) > (r2 & 0xf) {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }

    if *r1 > r2 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }

    *r1 = r1.wrapping_sub(r2);
    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    *f = flags.into();
}

pub fn sub_8bit_registers_with_carry(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.insert(Flags::N);

    let carry = if flags.contains(Flags::C) { 1 } else { 0 };

    if (*r1 & 0xf0).wrapping_sub(r2 & 0xf0).wrapping_sub(carry) & 0x08 != 0 {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }

    if *r1 > r2 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }

    *r1 = r1.wrapping_sub(r2);
    *r1 = r1.wrapping_sub(carry);

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    *f = flags.into();
}

pub fn and_8bit_registers(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);
    flags.insert(Flags::H);
    flags.remove(Flags::C);

    *r1 &= r2;

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}

pub fn or_8bit_registers(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);
    flags.remove(Flags::H);
    flags.remove(Flags::C);

    *r1 |= r2;

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}

pub fn xor_8bit_registers(r1: &mut u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);
    flags.remove(Flags::H);
    flags.remove(Flags::C);

    *r1 ^= r2;

    if *r1 == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}

pub fn compare_8bit_registers(r1: u8, r2: u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.insert(Flags::N);

    if (r1 & 0xf) > (r2 & 0xf) {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }

    if r1 > r2 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }

    if r1.wrapping_sub(r2) == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }

    *f = flags.into();
}