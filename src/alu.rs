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
    let aux = *r & 0x1f;
    if aux & 0x10 != aux.wrapping_sub(1) & 0x10 {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }
    if *r == 0x01 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    flags.insert(Flags::N);
    *f = flags.into();
    *r = r.wrapping_sub(1);
}

#[test]
fn decrement_8bit_register_test() {
    let mut r1: u8 = 0xff;
    let mut r2: u8 = 0x10;
    let mut r3: u8 = 0x00;
    let mut r4: u8 = 0x01;

    let mut f1: u8 = 0;
    let mut f2: u8 = 0;
    let mut f3: u8 = 0;
    let mut f4: u8 = 0;

    decrement_8bit_register(&mut r1, &mut f1);
    decrement_8bit_register(&mut r2, &mut f2);
    decrement_8bit_register(&mut r3, &mut f3);
    decrement_8bit_register(&mut r4, &mut f4);

    assert_eq!(0xfeu8, r1);
    assert_eq!(0x0fu8, r2);
    assert_eq!(0xffu8, r3);
    assert_eq!(0x00u8, r4);

    assert_eq!(Flags::N.bits(), f1);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f2);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f3);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f3);
    assert_eq!(Flags::Z.bits() | Flags::N.bits(), f4);
}

pub fn decrement_16bit_register(h: &mut u8, l: &mut u8) {
    if *l == 0 {
        *h = h.wrapping_sub(1);
    }
    *l = l.wrapping_sub(1);
}

#[test]
fn decrement_16bit_register_test() {
    let mut r1_h: u8 = 0x00;
    let mut r1_l: u8 = 0x00;

    let mut r2_h: u8 = 0x01;
    let mut r2_l: u8 = 0x00;

    let mut r3_h: u8 = 0xff;
    let mut r3_l: u8 = 0xff;

    decrement_16bit_register(&mut r1_h, &mut r1_l);
    decrement_16bit_register(&mut r2_h, &mut r2_l);
    decrement_16bit_register(&mut r3_h, &mut r3_l);

    assert_eq!(r1_h, 0xffu8);
    assert_eq!(r1_l, 0xffu8);

    assert_eq!(r2_h, 0x00u8);
    assert_eq!(r2_l, 0xffu8);

    assert_eq!(r3_h, 0xffu8);
    assert_eq!(r3_l, 0xfeu8);
}

pub fn increment_8bit_register(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::N);
    if *r == 0xff {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    if *r & 0xf == 0xf {
        flags.insert(Flags::H);
    } else {
        flags.remove(Flags::H);
    }
    *f = flags.into();
    *r = r.wrapping_add(1);
}

#[test]
fn increment_8bit_register_tests() {
    let mut r1: u8 = 0x00;
    let mut r2: u8 = 0x0f;
    let mut r3: u8 = 0xff;

    let mut f1: u8 = 0;
    let mut f2: u8 = 0;
    let mut f3: u8 = 0;

    increment_8bit_register(&mut r1, &mut f1);
    increment_8bit_register(&mut r2, &mut f2);
    increment_8bit_register(&mut r3, &mut f3);

    assert_eq!(0x01u8, r1);
    assert_eq!(0x10u8, r2);
    assert_eq!(0x00u8, r3);

    assert_eq!(0x00, f1);
    assert_eq!(Flags::H.bits(), f2);
    assert_eq!(Flags::H.bits() | Flags::Z.bits(), f3);
}

pub fn increment_16bit_register(h: &mut u8, l: &mut u8) {
    *l = l.wrapping_add(1);
    if *l == 0 {
        *h = h.wrapping_add(1);
    }
}

#[test]
fn increment_16bit_register_test() {
    let mut r1_h: u8 = 0x00;
    let mut r1_l: u8 = 0x00;

    let mut r2_h: u8 = 0x00;
    let mut r2_l: u8 = 0xff;

    let mut r3_h: u8 = 0xff;
    let mut r3_l: u8 = 0xff;

    increment_16bit_register(&mut r1_h, &mut r1_l);
    increment_16bit_register(&mut r2_h, &mut r2_l);
    increment_16bit_register(&mut r3_h, &mut r3_l);

    assert_eq!(r1_h, 0x00u8);
    assert_eq!(r1_l, 0x01u8);

    assert_eq!(r2_h, 0x01u8);
    assert_eq!(r2_l, 0x00u8);

    assert_eq!(r3_h, 0x00u8);
    assert_eq!(r3_l, 0x00u8);
}

pub fn rotate_left_and_store_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::Z | Flags::N | Flags::H);
    if *r & 0x80 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
    *f = flags.into();
    *r = r.rotate_left(1);
}

#[test]
fn rotate_left_and_store_carry_test() {
    let mut r = 0b1010_1010;
    let mut f = 0;

    for _ in 0..4  {
        rotate_left_and_store_carry(&mut r, &mut f);
        assert_eq!(0b0101_0101, r);
        assert_eq!(Flags::C.bits(), f);

        rotate_left_and_store_carry(&mut r, &mut f);
        assert_eq!(0b1010_1010, r);
        assert_eq!(0, f);
    }

    r = 0b1111_0000;
    f = 0;

    rotate_left_and_store_carry(&mut r, &mut f);
    assert_eq!(0b1110_0001, r);
    assert_eq!(Flags::C.bits(), f);
}

pub fn rotate_left_through_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    let carry = if flags.contains(Flags::C) { 0x01 } else { 0x00 };

    flags.remove(Flags::Z | Flags::N | Flags::H);
    if *r & 0x80 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
    *f = flags.into();

    *r = (r.rotate_left(1) & 0xfe) | carry;
}

#[test]
fn rotate_left_through_carry_test() {
    let mut r = 0b1010_1010;
    let mut f = 0;

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0100, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_1001, r);
    assert_eq!(0, f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_0101, r);
    assert_eq!(0, f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b0100_1010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b1001_0101, r);
    assert_eq!(0, f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b0010_1010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0101, r);
    assert_eq!(0, f);

    rotate_left_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_1010, r);
    assert_eq!(0, f);
}

pub fn rotate_right_and_store_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.remove(Flags::Z | Flags::N | Flags::H);
    if *r & 0x01 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
    *f = flags.into();
    *r = r.rotate_right(1);
}

#[test]
fn rotate_right_and_store_carry_test() {
    let mut r = 0b1010_1010;
    let mut f = 0;

    for _ in 0..4  {
        rotate_right_and_store_carry(&mut r, &mut f);
        assert_eq!(0b0101_0101, r);
        assert_eq!(0, f);

        rotate_right_and_store_carry(&mut r, &mut f);
        assert_eq!(0b1010_1010, r);
        assert_eq!(Flags::C.bits(), f);
    }

    r = 0b1111_0000;
    f = 0;

    rotate_right_and_store_carry(&mut r, &mut f);
    assert_eq!(0b0111_1000, r);
    assert_eq!(0, f);
}

pub fn rotate_right_through_carry(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    let carry: u8 = if flags.contains(Flags::C) { 0x80 } else { 0x00 };

    flags.remove(Flags::Z | Flags::N | Flags::H);
    if *r & 0x01 != 0 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
    *f = flags.into();

    *r = (r.rotate_right(1) & 0x7f) | carry;
}

#[test]
fn rotate_right_through_carry_test() {
    let mut r = 0b1010_1010;
    let mut f = 0;

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0101, r);
    assert_eq!(0, f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b0010_1010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b1001_0101, r);
    assert_eq!(0, f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b0100_1010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_0101, r);
    assert_eq!(0, f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0010, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_1001, r);
    assert_eq!(0, f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b0101_0100, r);
    assert_eq!(Flags::C.bits(), f);

    rotate_right_through_carry(&mut r, &mut f);
    assert_eq!(0b1010_1010, r);
    assert_eq!(0, f);
}

pub fn decimal_adjust(r: &mut u8, f: &mut u8) {
    let adjustment = (*r / 10) * 6;
    let mut flags = Flags::from(*f);

    if *r > 0x99 {
        flags.insert(Flags::C);
    } else {
        flags.remove(Flags::C);
    }
  
    if flags.contains(Flags::N) {
        panic!("Not validated with negative numbers");
    } else {
        *r = r.wrapping_add(adjustment);
    }

    if *r == 0 {
        flags.insert(Flags::Z);
    } else {
        flags.remove(Flags::Z);
    }
    flags.remove(Flags::H);

    *f = flags.into();
}

#[test]
fn decimal_adjust_test() {
    for i in 0..100 {
        let mut r = i;
        let mut f = 0;
        decimal_adjust(&mut r, &mut f);

        let e = u8::from_str_radix(&format!("{}", i), 16).unwrap();
        println!("{} {:x} {:x}", i, r, e);
        assert_eq!(e, r);
    }
}

pub fn complement(r: &mut u8, f: &mut u8) {
    let mut flags = Flags::from(*f);
    flags.insert(Flags::N | Flags::H);
    *f = flags.into();

    *r = !*r;
}

#[test]
fn complement_test() {
    let mut r1: u8 = 0;
    let mut r2: u8 = 0xff;
    let mut r3: u8 = 0b10101010;
    let mut r4: u8 = 0b11110000;

    let mut f1: u8 = 0;
    let mut f2: u8 = 0;
    let mut f3: u8 = 0;
    let mut f4: u8 = 0;

    complement(&mut r1, &mut f1);
    complement(&mut r2, &mut f2);
    complement(&mut r3, &mut f3);
    complement(&mut r4, &mut f4);

    assert_eq!(0xff, r1);
    assert_eq!(0x00, r2);
    assert_eq!(0b01010101, r3);
    assert_eq!(0b00001111, r4);

    assert_eq!(Flags::N.bits() | Flags::H.bits(), f1);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f2);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f3);
    assert_eq!(Flags::N.bits() | Flags::H.bits(), f4);
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

#[test]
fn add_8bit_registers_test() {
    let mut r1 = 0x00;
    let mut r2 = 0x0f;
    let mut r3 = 0xff;
    let mut r4 = 0x00;

    let mut f1 = Flags::N.bits();
    let mut f2 = 0;
    let mut f3 = 0;
    let mut f4 = 0;

    add_8bit_registers(&mut r1, 0xff, &mut f1);
    add_8bit_registers(&mut r2, 0x01, &mut f2);
    add_8bit_registers(&mut r3, 0x01, &mut f3);
    add_8bit_registers(&mut r4, 0x00, &mut f4);

    assert_eq!(0xff, r1);
    assert_eq!(0x10, r2);
    assert_eq!(0x00, r3);
    assert_eq!(0x00, r4);

    assert_eq!(0, f1);
    assert_eq!(Flags::H.bits(), f2);
    assert_eq!((Flags::H | Flags::C | Flags::Z).bits(), f3);
    assert_eq!(Flags::Z.bits(), f4);
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

#[test]
fn add_8bit_registers_with_carry_test() {
    let mut r1 = 0x00;
    let mut r2 = 0x0f;
    let mut r3 = 0xff;
    let mut r4 = 0x00;

    let mut r5 = 0x00;
    let mut r6 = 0x0f;
    let mut r7 = 0xff;
    let mut r8 = 0x00;

    let mut f1 = Flags::N.bits();
    let mut f2 = 0;
    let mut f3 = 0;
    let mut f4 = 0;

    let mut f5 = Flags::N.bits() | Flags::C.bits();
    let mut f6 = Flags::C.bits();
    let mut f7 = Flags::C.bits();
    let mut f8 = Flags::C.bits();

    add_8bit_registers_with_carry(&mut r1, 0xff, &mut f1);
    add_8bit_registers_with_carry(&mut r2, 0x01, &mut f2);
    add_8bit_registers_with_carry(&mut r3, 0x01, &mut f3);
    add_8bit_registers_with_carry(&mut r4, 0x00, &mut f4);

    add_8bit_registers_with_carry(&mut r5, 0xff, &mut f5);
    add_8bit_registers_with_carry(&mut r6, 0x01, &mut f6);
    add_8bit_registers_with_carry(&mut r7, 0x01, &mut f7);
    add_8bit_registers_with_carry(&mut r8, 0x00, &mut f8);

    assert_eq!(0xff, r1);
    assert_eq!(0x10, r2);
    assert_eq!(0x00, r3);
    assert_eq!(0x00, r4);

    assert_eq!(0x00, r5);
    assert_eq!(0x11, r6);
    assert_eq!(0x01, r7);
    assert_eq!(0x01, r8);

    assert_eq!(0, f1);
    assert_eq!(Flags::H.bits(), f2);
    assert_eq!((Flags::Z | Flags::C | Flags::H).bits(), f3);
    assert_eq!(Flags::Z.bits(), f4);

    assert_eq!((Flags::Z | Flags::C | Flags::H).bits(), f5);
    assert_eq!(Flags::H.bits(), f6);
    assert_eq!((Flags::C | Flags::H).bits(), f7);
    assert_eq!(0, f8);
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

#[test]
fn add_16bit_registers_test() {
    let mut r1 = R16::from(0x0000u16);
    let mut r2 = R16::from(0x1000u16);
    let mut r3 = R16::from(0x0101u16);
    let mut r4 = R16::from(0xf1f1u16);

    let mut f1: u8 = (Flags::Z | Flags::N | Flags::H | Flags::C).bits();
    let mut f2: u8 = 0;
    let mut f3: u8 = 0;
    let mut f4: u8 = 0;

    add_16bit_registers(&mut r1.h, &mut r1.l, 0x00, 0x00, &mut f1);
    add_16bit_registers(&mut r2.h, &mut r2.l, 0x00, 0x01, &mut f2);
    add_16bit_registers(&mut r3.h, &mut r3.l, 0x00, 0xff, &mut f3);
    add_16bit_registers(&mut r4.h, &mut r4.l, 0x0e, 0x0f, &mut f4);

    assert_eq!(0x0000u16, r1.into());
    assert_eq!(0x1001u16, r2.into());
    assert_eq!(0x0200u16, r3.into());
    assert_eq!(0x0000u16, r4.into());

    assert_eq!(Flags::Z.bits(), f1);
    assert_eq!(0, f2);
    assert_eq!(Flags::H.bits(), f3);
    assert_eq!(Flags::H.bits() | Flags::C.bits(), f4);
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