use super::flags::Flags;
use std::ops::RangeInclusive;

#[derive(Copy,Clone,Default)]
/// Gameboy (LR35902) 8 Bit Arithmetic Logic Unit
pub struct Alu {
    /// Flags [ZNHC----]
    pub flags: Flags,
}

impl From<Flags> for Alu {
    fn from(flags: Flags) -> Self { Self { flags: flags } }
}

impl Into<Flags> for Alu {
    fn into(self) -> Flags { self.flags }
}

impl From<u8> for Alu {
    fn from(flags: u8) -> Self { Self { flags: Flags::from(flags) } }
}

impl Into<u8> for Alu {
    fn into(self) -> u8 { self.flags.into() }
}

#[allow(dead_code)]
impl Alu {
    pub fn new(flags: u8) -> Self {
        Alu {
            flags: Flags::from(flags),
        }
    }

    /// Add arg to acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Set if carry from bit 3
    /// - C: Set if carry from bit 7
    pub fn add(&mut self, acc: u8, arg: u8) -> u8 {
        let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
        let (ret, carry) = acc.overflowing_add(arg);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        ret
    }

    /// Add arg to acc
    ///
    /// Flags Affected:
    /// - Z: Not Affected
    /// - N: Reset
    /// - H: Set if carry from bit 11
    /// - C: Set if carry from bit 15
    pub fn add16(&mut self, acc: u16, arg: u16) -> u16 {
        let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
        let (ret, carry) = acc.overflowing_add(arg);

        self.flags.set_zero_if(acc == 0);
        self.flags.reset_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        ret
    }

    /// Add (arg + carry) to acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Set if carry from bit 3
    /// - C: Set if carry from bit 7
    pub fn adc(&mut self, acc: u8, arg: u8) -> u8 {
        if !self.flags.carry() {
            return self.add(acc, arg)
        }

        let (aux, half1) = arg.wrapping_shl(4).overflowing_add(0x10);
        let (_, half2) = acc.wrapping_shl(4).overflowing_add(aux);

        let (aux, carry1) = arg.overflowing_add(1);
        let (ret, carry2) = acc.overflowing_add(aux);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.set_half_if(half1 || half2);
        self.flags.set_carry_if(carry1 || carry2);

        ret
    }

    /// Subtract arg from acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if borrow from bit 4
    /// - C: Set if no borrow
    pub fn sub(&mut self, acc: u8, arg: u8) -> u8 {
        let half = acc & 0x0f < arg & 0x0f;
        let (ret, carry) = acc.overflowing_sub(arg);

        self.flags.set_zero_if(ret == 0);
        self.flags.set_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        ret
    }

    /// Subtract (arg + carry) from acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if borrow from bit 4
    /// - C: Set if no borrow
    pub fn sbc(&mut self, acc: u8, arg: u8) -> u8 {
        if !self.flags.carry() {
            return self.sub(acc, arg);
        }

        let half1 = arg & 0xf == 0xf;
        let half2 = acc & 0x0f < arg & 0x0f;

        let (aux, carry1) = arg.overflowing_add(1);
        let (ret, carry2) = acc.overflowing_sub(aux);

        self.flags.set_zero_if(ret == 0);
        self.flags.set_sub();
        self.flags.set_half_if(half1 || half2);
        self.flags.set_carry_if(carry1 || carry2);

        ret
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N - Reset
    /// - H - Set if carry from bit 3
    /// - C - Not affected
    pub fn inc(&mut self, acc: u8) -> u8 {
        self.flags.set_zero_if(acc == 0xff);
        self.flags.reset_sub();
        self.flags.set_half_if(acc & 0xf == 0xf);

        acc.wrapping_add(1)
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Not affected
    /// - H: Not affected
    /// - C: Not affected
    pub fn inc16(&mut self, acc: u16) -> u16 {
        acc.wrapping_add(1)
    }

    /// Decrement acc by one
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if no borrow from bit 4
    /// - C: Not affected
    pub fn dec(&mut self, acc: u8) -> u8 {
        self.flags.set_zero_if(acc == 1);
        self.flags.set_sub();
        self.flags.set_half_if(acc & 0xf == 0);

        acc.wrapping_sub(1)
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Not affected
    /// - H: Not affected
    /// - C: Not affected
    pub fn dec16(&mut self, acc: u16) -> u16 {
        acc.wrapping_sub(1)
    }

    /// Logical AND of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Set
    /// - C: Reset
    pub fn and(&mut self, acc: u8, arg: u8) -> u8 {
        let ret = acc & arg;

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.set_half();
        self.flags.reset_carry();

        ret
    }

    /// Logical OR of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn or(&mut self, acc: u8, arg: u8) -> u8 {
        let ret = acc | arg;

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();

        ret
    }

    /// Logical eXclusive OR (XOR) of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn xor(&mut self, acc: u8, arg: u8) -> u8 {
        let ret = acc ^ arg;

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();

        ret
    }

    /// Compare acc with arg [like subtract (sub) but the result is throw away]
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero [acc == arg]
    /// - N: Set
    /// - H: Set if no borrow from bit 4
    /// - C: Set for no borrow [acc < arg]
    pub fn compare(&mut self, acc: u8, arg: u8) {
        self.sub(acc, arg);
    }

    /// Rotates acc to the left with bit 7 being moved to bit 0 and also stored into the carry.
    ///
    /// Flags Affected:
    /// - Z - Set if result is zero
    /// - N - Reset
    /// - H - Reset
    /// - C - Contains old bit 7
    pub fn rlc(&mut self, acc: u8) -> u8 {
        let ret = acc.rotate_left(1);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(ret & 1 << 0 != 0);

        ret
    }

    /// Rotates acc to the right with bit 0 moved to bit 7 and also stored into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn rrc(&mut self, acc: u8) -> u8 {
        let ret = acc.rotate_right(1);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(ret & 1 << 7 != 0);

        ret
    }

    /// Rotates acc to the left with the carry's value put into bit 0 and bit 7 is put into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 7
    pub fn rl(&mut self, acc: u8) -> u8 {
        let ret = acc.rotate_left(1) & !(1 << 0) | if self.flags.carry() { 1 } else { 0 } << 0;

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(acc & 0x80 != 0);

        ret
    }

    /// Rotates acc to the right with the carry put in bit 7 and bit 0 put into the carry.
    /// 
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn rr(&mut self, acc: u8) -> u8 {
        let ret = acc.rotate_right(1) & !(1 << 7) |  if self.flags.carry() { 1 } else { 0 } << 7;

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(acc & 0x01 != 0);

        ret
    }

    /// Shift acc to the left with bit 0 set to 0 and bit 7 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 7
    pub fn sla(&mut self, acc: u8) -> u8 {
        let ret = acc.wrapping_shl(1);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(acc & 1 << 7 != 0);

        ret
    }

    /// Shift acc to the right without changing bit 7 and put bit 0 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn sra(&mut self, acc: u8) -> u8 {
        let ret = (acc & 0x80) | acc.wrapping_shr(1);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(acc & 1 << 0 != 0);

        ret
    }

    /// Shift acc to the right with 0 put in bit 7 and put bit 0 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn srl(&mut self, acc: u8) -> u8 {
        let ret = acc.wrapping_shr(1);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(acc & 0x01 != 0);

        ret
    }

    /// Swap upper and lower nibbles of acc
    /// 
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn nibble_swap(&mut self, acc: u8) -> u8 {
        let ret = acc.wrapping_shl(4) | acc.wrapping_shr(4);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();

        ret
    }

    /// Complement acc [Flip all bits]
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Set
    /// - H: Set
    /// - C: Not affected
    pub fn complement(&mut self, acc: u8) -> u8 {
        let ret = !acc;

        self.flags.set_sub();
        self.flags.set_half();

        ret
    }

    /// Test bit
    ///
    /// Flags Affected:
    /// - Z: Set if bit b of register r is 0
    /// - N: Reset
    /// - H: Set
    /// - C: Not affected
    pub fn test_bit(&mut self, acc: u8, bit_index: u8) {
        assert!(bit_index < 8);

        self.flags.set_zero_if(acc & (1 << bit_index) == 0);
        self.flags.reset_sub();
        self.flags.set_half();
    }

    /// Set bit to 1
    ///
    /// Flags Affected: NONE
    pub fn set_bit(&mut self, acc: u8, bit_index: u8) -> u8 {
        assert!(bit_index < 8);
        acc | (1 << bit_index)
    }

    /// Reset bit to 0
    ///
    /// Flags Affected: NONE
    pub fn reset_bit(&mut self, acc: u8, bit_index: u8) -> u8 {
        assert!(bit_index < 8);
        acc & !(1 << bit_index)
    }

    // --------------------------------------------------------------------------------
    // |           | C Flag  | HEX value in | H Flag | HEX value in | Number  | C flag|
    // | Operation | Before  | upper digit  | Before | lower digit  | added   | After |
    // |           | DAA     | (bit 7-4)    | DAA    | (bit 3-0)    | to byte | DAA   |
    // |------------------------------------------------------------------------------|
    // |           |    0    |     0-9      |   0    |     0-9      |   00    |   0   | R0
    // |   ADD     |    0    |     0-8      |   0    |     A-F      |   06    |   0   | R1
    // |           |    0    |     0-9      |   1    |     0-3      |   06    |   0   | R2
    // |   ADC     |    0    |     A-F      |   0    |     0-9      |   60    |   1   | R3
    // |           |    0    |     9-F      |   0    |     A-F      |   66    |   1   | R4
    // |   INC     |    0    |     A-F      |   1    |     0-3      |   66    |   1   | R5
    // |           |    1    |     0-2      |   0    |     0-9      |   60    |   1   | R6
    // |           |    1    |     0-2      |   0    |     A-F      |   66    |   1   | R7
    // |           |    1    |     0-3      |   1    |     0-3      |   66    |   1   | R8
    // |------------------------------------------------------------------------------|
    // |   SUB     |    0    |     0-9      |   0    |     0-9      |   00    |   0   | R9
    // |   SBC     |    0    |     0-8      |   1    |     6-F      |   FA    |   0   | R10
    // |   DEC     |    1    |     7-F      |   0    |     0-9      |   A0    |   1   | R11
    // |   NEG     |    1    |     6-F      |   1    |     6-F      |   9A    |   1   | R12
    // |------------------------------------------------------------------------------|
    // Source: http://www.z80.info/z80syntx.htm#DAA

    //   Flags, upper_range, lower_range, adjustment, carry |
    const DAA_TABLE: &'static [(Flags, RangeInclusive<u8>, RangeInclusive<u8>, u8, bool); 13] = &[
        (Flags::NONE, (0x0..=0x9), (0x0..=0x9), 0x00, false), // R0
        (Flags::NONE, (0x0..=0x8), (0xA..=0xF), 0x06, false), // R1
        (Flags::H,    (0x0..=0x9), (0x0..=0x3), 0x06, false), // R2
        (Flags::NONE, (0xA..=0xF), (0x0..=0x9), 0x60, true),  // R3
        (Flags::NONE, (0x9..=0xF), (0xA..=0xF), 0x66, true),  // R4
        (Flags::H,    (0xA..=0xF), (0x0..=0x3), 0x66, true),  // R5
        (Flags::C,    (0x0..=0x2), (0x0..=0x9), 0x60, true),  // R6
        (Flags::C,    (0x0..=0x2), (0xA..=0xF), 0x66, true),  // R7
        (Flags::HC,   (0x0..=0x3), (0x0..=0x3), 0x66, true),  // R8

        (Flags::N,    (0x0..=0x9), (0x0..=0x9), 0x00, false), // R9
        (Flags::NH,   (0x0..=0x8), (0x6..=0xF), 0xFA, false), // R10
        (Flags::NC,   (0x7..=0xF), (0x0..=0x9), 0xA0, true),  // R11
        (Flags::NHC,  (0x6..=0xF), (0x6..=0xF), 0x9A, true),  // R12
    ];

    /// Decimal Adjust acc to obtain the bcd representation
    ///
    /// - Z: Set if register acc is zero. 
    /// - N: Not affected.
    /// - H: Reset.
    /// - C: Set or reset according to operation.
    pub fn daa(&mut self, acc: u8) -> u8 {
        let upper: u8 = acc.wrapping_shr(4);
        let lower: u8 = acc & 0xf;

        let flags = self.flags & Flags::NHC;

        let mut adjustment: u8 = 0;
        let mut carry: bool = false;
        let mut found: bool = false;

        for (lookup_flags, upper_range, lower_range, next_adjustment, next_carry) in Self::DAA_TABLE {
            if &flags == lookup_flags && upper_range.contains(&upper) && lower_range.contains(&lower) {
                adjustment = *next_adjustment;
                carry = *next_carry;
                found = true;
                break;
            }
        }

        if !found {
            panic!("Invalid Value {:?} {:x}", self.flags, acc);
        }

        let ret = acc.wrapping_add(adjustment);

        self.flags.set_zero_if(ret == 0);
        self.flags.reset_half();
        self.flags.set_carry_if(carry);

        ret
    }
}

#[test]
fn alu8_add_test() {
    let mut alu1: Alu = Flags::N.into();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();

    let ret1 = alu1.add(0x00, 255);
    let ret2 = alu2.add(0x0f, 1);
    let ret3 = alu3.add(0xff, 1);
    let ret4 = alu4.add(0x00, 0);

    assert_eq!(0xff, ret1);
    assert_eq!(0x10, ret2);
    assert_eq!(0x00, ret3);
    assert_eq!(0x00, ret4);

    assert_eq!(Flags::NONE, alu1.into());
    assert_eq!(Flags::H,    alu2.into());
    assert_eq!(Flags::ZHC,  alu3.into());
    assert_eq!(Flags::Z,    alu4.into());
}

#[test]
fn alu8_adc_test() {
    let mut alu1: Alu = Flags::N.into();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();

    let mut alu5: Alu = Flags::NC.into();
    let mut alu6: Alu = Flags::C.into();
    let mut alu7: Alu = Flags::C.into();
    let mut alu8: Alu = Flags::C.into();

    let ret1 = alu1.adc(0x00, 0xff);
    let ret2 = alu2.adc(0x0f, 0x01);
    let ret3 = alu3.adc(0xff, 0x01);
    let ret4 = alu4.adc(0x00, 0x00);

    let ret5 = alu5.adc(0x00, 0xff);
    let ret6 = alu6.adc(0x0f, 0x01);
    let ret7 = alu7.adc(0xff, 0x01);
    let ret8 = alu8.adc(0x00, 0x00);

    assert_eq!(0xff, ret1);
    assert_eq!(0x10, ret2);
    assert_eq!(0x00, ret3);
    assert_eq!(0x00, ret4);

    assert_eq!(0x00, ret5);
    assert_eq!(0x11, ret6);
    assert_eq!(0x01, ret7);
    assert_eq!(0x01, ret8);

    assert_eq!(Flags::NONE, alu1.into());
    assert_eq!(Flags::H,    alu2.into());
    assert_eq!(Flags::ZHC,  alu3.into());
    assert_eq!(Flags::Z,    alu4.into());

    assert_eq!(Flags::ZHC,  alu5.into());
    assert_eq!(Flags::H,    alu6.into());
    assert_eq!(Flags::HC,   alu7.into());
    assert_eq!(Flags::NONE, alu8.into());
}

#[test]
fn alu8_sub_test() {
    let mut alu1 = Alu::default();
    let mut alu2 = Alu::default();
    let mut alu3 = Alu::default();
    let mut alu4 = Alu::default();
    let mut alu5 = Alu::default();

    let ret1 = alu1.sub(0x00, 0x00);
    let ret2 = alu2.sub(0xff, 0xee);
    let ret3 = alu3.sub(0x80, 0x18);
    let ret4 = alu4.sub(0x80, 0x81);
    let ret5 = alu5.sub(0xff, 0xff);

    assert_eq!(0x00, ret1);
    assert_eq!(0x11, ret2);
    assert_eq!(0x68, ret3);
    assert_eq!(0xff, ret4);
    assert_eq!(0x00, ret5);

    assert_eq!(Flags::ZN,  alu1.into());
    assert_eq!(Flags::N,   alu2.into());
    assert_eq!(Flags::NH,  alu3.into());
    assert_eq!(Flags::NHC, alu4.into());
    assert_eq!(Flags::ZN,  alu5.into());
}

#[test]
fn alu8_sbc_test() {
    let mut alu1: Alu = Alu::default();
    let mut alu2: Alu = Flags::C.into();
    let mut alu3: Alu = Flags::C.into();
    let mut alu4: Alu = Flags::C.into();

    let ret1 = alu1.sbc(0x00, 0x00);
    let ret2 = alu2.sbc(0x01, 0x00);
    let ret3 = alu3.sbc(0x00, 0xff);
    let ret4 = alu4.sbc(0x80, 0x7f);

    assert_eq!(0x00, ret1);
    assert_eq!(0x00, ret2);
    assert_eq!(0x00, ret3);
    assert_eq!(0x00, ret4);

    assert_eq!(Flags::ZN,   alu1.into());
    assert_eq!(Flags::ZN,   alu2.into());
    assert_eq!(Flags::ZNHC, alu3.into());
    assert_eq!(Flags::ZNH,  alu4.into());
}

#[test]
fn alu8_inc_test() {
    let mut alu1: Alu = Flags::N.into();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Flags::C.into();
    let mut alu4: Alu = Alu::default();
    let mut alu5: Alu = Flags::C.into();

    let ret1 = alu1.inc(0x00);
    let ret2 = alu2.inc(0x0f);
    let ret3 = alu3.inc(0x8f);
    let ret4 = alu4.inc(0xff);
    let ret5 = alu5.inc(0xff);

    assert_eq!(0x01, ret1);
    assert_eq!(0x10, ret2);
    assert_eq!(0x90, ret3);
    assert_eq!(0x00, ret4);
    assert_eq!(0x00, ret5);

    assert_eq!(Flags::NONE, alu1.into());
    assert_eq!(Flags::H,    alu2.into());
    assert_eq!(Flags::HC,   alu3.into());
    assert_eq!(Flags::ZH,   alu4.into());
    assert_eq!(Flags::ZHC,  alu5.into());
}

#[test]
fn alu8_dec_test() {
    let mut alu1: Alu = Flags::C.into();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Flags::C.into();
    let mut alu4: Alu = Alu::default();

    let ret1 = alu1.dec(0x01);
    let ret2 = alu2.dec(0x00);
    let ret3 = alu3.dec(0x00);
    let ret4 = alu4.dec(0x10);

    assert_eq!(0x00, ret1);
    assert_eq!(0xff, ret2);
    assert_eq!(0xff, ret3);
    assert_eq!(0x0f, ret4);

    assert_eq!(Flags::ZNC, alu1.flags);
    assert_eq!(Flags::NH,  alu2.flags);
    assert_eq!(Flags::NHC, alu3.flags);
    assert_eq!(Flags::NH,  alu4.flags);
}

#[test]
fn alu8_and_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Flags::ZNHC.into();
    let mut alu4: Alu = Flags::ZNHC.into();

    let ret1 = alu1.and(0xff, 0x00);
    let ret2 = alu2.and(0xff, 0xff);
    let ret3 = alu3.and(0b1010_1010, 0b1010_1010);
    let ret4 = alu4.and(0b1010_1010, 0b0101_0101);

    assert_eq!(0x00, ret1);
    assert_eq!(0xff, ret2);
    assert_eq!(0b1010_1010, ret3);
    assert_eq!(0x00, ret4);

    assert_eq!(Flags::ZH, alu1.into());
    assert_eq!(Flags::H,  alu2.into());
    assert_eq!(Flags::H,  alu3.into());
    assert_eq!(Flags::ZH, alu4.into());
}

#[test]
fn alu8_or_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Flags::ZNHC.into();
    let mut alu4: Alu = Flags::ZNHC.into();

    let ret1 = alu1.or(0xff, 0xff);
    let ret2 = alu2.or(0b1010_1010, 0b1010_1010);
    let ret3 = alu3.or(0b1010_1010, 0b0101_0101);
    let ret4 = alu4.or(0x00, 0x00);

    assert_eq!(0xff, ret1);
    assert_eq!(0b1010_1010, ret2);
    assert_eq!(0xff, ret3);
    assert_eq!(0x00, ret4);

    assert_eq!(Flags::NONE, alu1.into());
    assert_eq!(Flags::NONE, alu2.into());
    assert_eq!(Flags::NONE, alu3.into());
    assert_eq!(Flags::Z,    alu4.into());
}

#[test]
fn alu8_xor_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Flags::ZNHC.into();
    let mut alu4: Alu = Flags::ZNHC.into();
    let mut alu5: Alu = Flags::ZNHC.into();

    let ret1 = alu1.xor(0xff, 0xff);
    let ret2 = alu2.xor(0x00, 0xff);
    let ret3 = alu3.xor(0b1010_1010, 0b1010_1010);
    let ret4 = alu4.xor(0b1010_1010, 0b0101_0101);
    let ret5 = alu5.xor(0x00, 0x00);

    assert_eq!(0x00, ret1);
    assert_eq!(0xff, ret2);
    assert_eq!(0x00, ret3);
    assert_eq!(0xff, ret4);
    assert_eq!(0x00, ret5);

    assert_eq!(Flags::Z,    alu1.into());
    assert_eq!(Flags::NONE, alu2.into());
    assert_eq!(Flags::Z,    alu3.into());
    assert_eq!(Flags::NONE, alu4.into());
    assert_eq!(Flags::Z,    alu5.into());
}

#[test]
fn alu8_compare_test() {
    let mut alu1: Alu = Alu::default();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();
    let mut alu5: Alu = Alu::default();
    let mut alu6: Alu = Alu::default();

    alu1.compare(0x00, 0x00);
    alu2.compare(0x00, 0x01);
    alu3.compare(0x00, 0xff);
    alu4.compare(0x01, 0x00);
    alu5.compare(0xff, 0xfe);
    alu6.compare(0x88, 0x79);

    assert_eq!(Flags::ZN,  alu1.flags); // Equal
    assert_eq!(Flags::NHC, alu2.flags); // Greater than
    assert_eq!(Flags::NHC, alu3.flags); // Greater than
    assert_eq!(Flags::N,   alu4.flags); // Less than
    assert_eq!(Flags::N,   alu5.flags); // Less than
    assert_eq!(Flags::NH,  alu6.flags); // Less than
}

#[test]
fn alu8_rlc_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::NH.into();

    let ret1 = alu1.rlc(0b1010_1010);
    assert_eq!(0b0101_0101, ret1);
    assert_eq!(Flags::C, alu1.into());

    let ret2 = alu1.rlc(ret1);
    assert_eq!(0b1010_1010, ret2);
    assert_eq!(Flags::NONE, alu1.flags);

    let ret3 = alu2.rlc(0x00);
    assert_eq!(0x00, ret3);
    assert_eq!(Flags::Z, alu2.flags);
}

#[test]
fn alu8_rrc_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::NH.into();

    let ret1 = alu1.rrc(0b1010_1010);
    assert_eq!(0b0101_0101, ret1);
    assert_eq!(Flags::NONE, alu1.into());

    let ret2 = alu1.rrc(ret1);
    assert_eq!(0b1010_1010, ret2);
    assert_eq!(Flags::C, alu1.into());

    let ret3 = alu2.rrc(0x00);
    assert_eq!(0x00, ret3);
    assert_eq!(Flags::Z, alu2.into());
}

#[test]
fn alu8_rl_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::NH.into();

    let ret1 = alu1.rl(0b1010_1010);
    assert_eq!(0b0101_0101, ret1);
    assert_eq!(Flags::C, alu1.into());

    let ret2 = alu1.rl(ret1);
    assert_eq!(0b1010_1011, ret2);
    assert_eq!(Flags::NONE, alu1.into());

    let ret3 = alu2.rl(0x00);
    assert_eq!(0x00, ret3);
    assert_eq!(Flags::Z, alu2.into());
}

#[test]
fn alu8_rr_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::NH.into();

    let ret1 = alu1.rr(0b1010_1010);
    assert_eq!(0b1101_0101, ret1);
    assert_eq!(Flags::NONE, alu1.flags);

    let ret2 = alu1.rr(ret1);
    assert_eq!(0b0110_1010, ret2);
    assert_eq!(Flags::C, alu1.into());

    let ret3 = alu2.rr(0x00);
    assert_eq!(0x00, ret3);
    assert_eq!(Flags::Z, alu2.into());
}

#[test]
fn alu8_sla_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();

    let ret1 = alu1.sla(0x00);
    let ret2 = alu2.sla(0x80);
    let ret3 = alu3.sla(0x81);
    let ret4 = alu4.sla(0x20);

    assert_eq!(0x00, ret1);
    assert_eq!(0x00, ret2);
    assert_eq!(0x02, ret3);
    assert_eq!(0x40, ret4);

    assert_eq!(Flags::Z,    alu1.into());
    assert_eq!(Flags::ZC,   alu2.into());
    assert_eq!(Flags::C,    alu3.into());
    assert_eq!(Flags::NONE, alu4.into());
}

#[test]
fn alu8_sra_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();
    let mut alu5: Alu = Alu::default();

    let ret1 = alu1.sra(0x00);
    let ret2 = alu2.sra(0x80);
    let ret3 = alu3.sra(0x81);
    let ret4 = alu4.sra(0x20);
    let ret5 = alu5.sra(0x01);

    assert_eq!(0x00, ret1);
    assert_eq!(0xc0, ret2);
    assert_eq!(0xc0, ret3);
    assert_eq!(0x10, ret4);
    assert_eq!(0x00, ret5);

    assert_eq!(Flags::Z,    alu1.into());
    assert_eq!(Flags::NONE, alu2.into());
    assert_eq!(Flags::C,    alu3.into());
    assert_eq!(Flags::NONE, alu4.into());
    assert_eq!(Flags::ZC,   alu5.into());
}

#[test]
fn alu8_srl_test() {
    let mut alu1: Alu = Flags::ZNHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Alu::default();
    let mut alu4: Alu = Alu::default();
    let mut alu5: Alu = Alu::default();

    let ret1 = alu1.srl(0x00);
    let ret2 = alu2.srl(0x80);
    let ret3 = alu3.srl(0x81);
    let ret4 = alu4.srl(0x20);
    let ret5 = alu5.srl(0x01);

    assert_eq!(0x00, ret1);
    assert_eq!(0x40, ret2);
    assert_eq!(0x40, ret3);
    assert_eq!(0x10, ret4);
    assert_eq!(0x00, ret5);

    assert_eq!(Flags::Z,    alu1.into());
    assert_eq!(Flags::NONE, alu2.into());
    assert_eq!(Flags::C,    alu3.into());
    assert_eq!(Flags::NONE, alu4.into());
    assert_eq!(Flags::ZC,   alu5.into());
}

#[test]
fn alu8_nibble_swap_test() {
    let mut alu1: Alu = Flags::NHC.into();
    let mut alu2: Alu = Flags::ZNHC.into();
    let mut alu3: Alu = Flags::ZNHC.into();

    let ret1 = alu1.nibble_swap(0x00);
    let ret2 = alu2.nibble_swap(0b1111_0000);
    let ret3 = alu3.nibble_swap(0b1010_0101);

    assert_eq!(0x00, ret1);
    assert_eq!(0b0000_1111, ret2);
    assert_eq!(0b0101_1010, ret3);

    assert_eq!(Flags::Z,    alu1.into());
    assert_eq!(Flags::NONE, alu2.into());
    assert_eq!(Flags::NONE, alu3.into());
}

#[test]
pub fn alu8_complement_test() {
    let mut alu1: Alu = Flags::ZC.into();
    let mut alu2: Alu = Alu::default();
    let mut alu3: Alu = Flags::Z.into();
    let mut alu4: Alu = Flags::C.into();

    let ret1 = alu1.complement(0x00);
    let ret2 = alu2.complement(0xff);
    let ret3 = alu3.complement(0b1100_0011);
    let ret4 = alu4.complement(0b1010_1010);

    assert_eq!(0xff, ret1);
    assert_eq!(0x00, ret2);
    assert_eq!(0b0011_1100, ret3);
    assert_eq!(0b0101_0101, ret4);

    assert_eq!(Flags::ZNHC, alu1.flags);
    assert_eq!(Flags::NH,   alu2.flags);
    assert_eq!(Flags::ZNH,  alu3.flags);
    assert_eq!(Flags::NHC,  alu4.flags);
}

#[test]
pub fn alu8_test_bit_test() {
    let mut alu: Alu = Flags::N.into();
    let data: u8 = 0b0011_1010;

    alu.test_bit(data, 0);
    assert_eq!(Flags::ZH, alu.into());

    alu.test_bit(data, 1);
    assert_eq!(Flags::H, alu.into());

    alu.test_bit(data, 2);
    assert_eq!(Flags::ZH, alu.into());

    alu.test_bit(data, 3);
    assert_eq!(Flags::H, alu.into());

    alu.test_bit(data, 4);
    assert_eq!(Flags::H, alu.into());

    alu.test_bit(data, 5);
    assert_eq!(Flags::H, alu.into());

    alu.test_bit(data, 6);
    assert_eq!(Flags::ZH, alu.into());

    alu.test_bit(data, 7);
    assert_eq!(Flags::ZH, alu.into());

    alu.flags.set_carry();

    alu.test_bit(data, 7);
    assert_eq!(Flags::ZHC, alu.into());
}

#[test]
pub fn alu8_set_bit_test() {
    let mut alu: Alu = Flags::ZNHC.into();
    let mut data: u8 = 0x00;

    data = alu.set_bit(data, 0);
    assert_eq!(data, 0b0000_0001);

    data = alu.set_bit(data, 1);
    assert_eq!(data, 0b0000_0011);

    data = alu.set_bit(data, 2);
    assert_eq!(data, 0b0000_0111);

    data = alu.set_bit(data, 3);
    assert_eq!(data, 0b0000_1111);

    data = alu.set_bit(data, 4);
    assert_eq!(data, 0b0001_1111);

    data = alu.set_bit(data, 5);
    assert_eq!(data, 0b0011_1111);

    data = alu.set_bit(data, 6);
    assert_eq!(data, 0b0111_1111);

    data = alu.set_bit(data, 7);
    assert_eq!(data, 0b1111_1111);
}

#[test]
pub fn alu8_reset_bit_test() {
    let mut alu: Alu = Flags::ZNHC.into();
    let mut data: u8 = 0b1111_1111;

    data = alu.reset_bit(data, 0);
    assert_eq!(data, 0b1111_1110);

    data = alu.reset_bit(data, 1);
    assert_eq!(data, 0b1111_1100);

    data = alu.reset_bit(data, 2);
    assert_eq!(data, 0b1111_1000);

    data = alu.reset_bit(data, 3);
    assert_eq!(data, 0b1111_0000);

    data = alu.reset_bit(data, 4);
    assert_eq!(data, 0b1110_0000);

    data = alu.reset_bit(data, 5);
    assert_eq!(data, 0b1100_0000);

    data = alu.reset_bit(data, 6);
    assert_eq!(data, 0b1000_0000);

    data = alu.reset_bit(data, 7);
    assert_eq!(data, 0b0000_0000);

    assert_eq!(Flags::ZNHC, alu.flags);
}

#[test]
pub fn alu_daa_test() {
    let mut alu1 = Alu::default();
    let ret1 = alu1.add(0x00, 0x00);
    assert_eq!(0x00, alu1.daa(ret1));

    let mut alu2 = Alu::default();
    let ret2 = alu2.add(0x06, 0x03);
    assert_eq!(0x09, alu2.daa(ret2));

    let mut alu3 = Alu::default();
    let ret3 = alu3.add(0x05, 0x05);
    assert_eq!(0x10, alu3.daa(ret3));

    let mut alu4 = Alu::default();
    let ret4 = alu4.add(0x08, 0x07);
    assert_eq!(0x15, alu4.daa(ret4));

    let mut alu5 = Alu::default();
    let ret5 = alu5.add(0x50, 0x40);
    assert_eq!(0x90, alu5.daa(ret5));

    let mut alu6 = Alu::default();
    let ret6 = alu6.add(0x50, 0x40);
    assert_eq!(0x90, alu6.daa(ret6));
}