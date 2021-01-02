use crate::flags::Flags;

#[derive(Default)]
/// Gameboy (LR35902) 8 Bit Arithmetic Logic Unit
pub struct ALU8 {
    /// Accumulator
    pub acc: u8,

    /// Flags [ZNHC----]
    pub flags: Flags,
}

#[allow(dead_code)]
impl ALU8 {
    pub fn new(acc: u8, flags: u8) -> Self {
        ALU8 {
            acc: acc,
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
    pub fn add(&mut self, arg: u8) {
        let (_, half) = self.acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
        let (acc, carry) = self.acc.overflowing_add(arg);

        self.flags.set_zero_if(acc == 0);
        self.flags.reset_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        self.acc = acc;
    }

    /// Add (arg + carry) to acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Set if carry from bit 3
    /// - C: Set if carry from bit 7
    pub fn adc(&mut self, arg1: u8) {
        if self.flags.carry() {
            let (aux, half1) = arg1.wrapping_shl(4).overflowing_add(0x10);
            let (_, half2) = self.acc.wrapping_shl(4).overflowing_add(aux);

            let (arg2, carry1) = arg1.overflowing_add(1);
            let (acc, carry2) = self.acc.overflowing_add(arg2);

            self.flags.set_zero_if(acc == 0);
            self.flags.reset_sub();
            self.flags.set_half_if(half1 || half2);
            self.flags.set_carry_if(carry1 || carry2);

            self.acc = acc;
        } else {
            self.add(arg1);
        }
    }

    /// Subtract arg from acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if borrow from bit 4
    /// - C: Set if no borrow
    pub fn sub(&mut self, arg: u8) {
        let half = self.acc & 0x0f < arg & 0x0f;
        let (acc, carry) = self.acc.overflowing_sub(arg);

        self.flags.set_zero_if(acc == 0);
        self.flags.set_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        self.acc = acc;
    }

    /// Subtract (arg + carry) from acc
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if borrow from bit 4
    /// - C: Set if no borrow
    pub fn sbc(&mut self, arg: u8) {
        if self.flags.carry() {
            let half1 = arg & 0xf == 0xf;
            let half2 = self.acc & 0x0f < arg & 0x0f;

            let (aux, carry1) = arg.overflowing_add(1);
            let (acc, carry2) = self.acc.overflowing_sub(aux);

            self.flags.set_zero_if(acc == 0);
            self.flags.set_sub();
            self.flags.set_half_if(half1 || half2);
            self.flags.set_carry_if(carry1 || carry2);

            self.acc = acc;
        } else {
            self.sub(arg);
        }
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N - Reset
    /// - H - Set if carry from bit 3
    /// - C - Not affected
    pub fn inc(&mut self) {
        self.flags.set_zero_if(self.acc == 0xff);
        self.flags.reset_sub();
        self.flags.set_half_if(self.acc & 0xf == 0xf);

        self.acc = self.acc.wrapping_add(1);
    }

    /// Decrement acc by one
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Set
    /// - H: Set if no borrow from bit 4
    /// - C: Not affected
    pub fn dec(&mut self) {
        self.flags.set_zero_if(self.acc == 1);
        self.flags.set_sub();
        self.flags.set_half_if(self.acc & 0xf == 0);

        self.acc = self.acc.wrapping_sub(1);
    }

    /// Logical AND of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Set
    /// - C: Reset
    pub fn and(&mut self, arg: u8) {
        self.acc &= arg;

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.set_half();
        self.flags.reset_carry();
    }

    /// Logical OR of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn or(&mut self, arg: u8) {
        self.acc |= arg;

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();
    }

    /// Logical eXclusive OR (XOR) of acc with arg
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn xor(&mut self, arg: u8) {
        self.acc ^= arg;

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();
    }

    /// Compare acc with arg [like subtract (sub) but the result is throw away]
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero [acc == arg]
    /// - N: Set
    /// - H: Set if no borrow from bit 4
    /// - C: Set for no borrow [acc < arg]
    pub fn compare(&mut self, arg: u8) {
        let aux = self.acc;
        self.sub(arg);
        self.acc = aux;
    }

    /// Rotates acc to the left with bit 7 being moved to bit 0 and also stored into the carry.
    ///
    /// Flags Affected:
    /// - Z - Set if result is zero
    /// - N - Reset
    /// - H - Reset
    /// - C - Contains old bit 7
    pub fn rlc(&mut self) {
        self.acc = self.acc.rotate_left(1);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(self.acc & 1 << 0 != 0);
    }

    /// Rotates acc to the right with bit 0 moved to bit 7 and also stored into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn rrc(&mut self) {
        self.acc = self.acc.rotate_right(1);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(self.acc & 1 << 7 != 0);
    }

    /// Rotates acc to the left with the carry's value put into bit 0 and bit 7 is put into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 7
    pub fn rl(&mut self) {
        let acc = self.acc.rotate_left(1);
        let carry = self.acc & 0x80 != 0;
        self.acc = (acc & !(1 << 0)) | if self.flags.carry() { 1 } else { 0 } << 0;

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }

    /// Rotates acc to the right with the carry put in bit 7 and bit 0 put into the carry.
    /// 
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn rr(&mut self) {
        let acc = self.acc.rotate_right(1);
        let carry = self.acc & 0x01 != 0;
        self.acc = (acc & !(1 << 7)) |  if self.flags.carry() { 1 } else { 0 } << 7;

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }

    /// Shift acc to the left with bit 0 set to 0 and bit 7 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 7
    pub fn sl(&mut self) {
        let carry = self.acc & 1 << 7 != 0;
        self.acc = self.acc.wrapping_shl(1);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }

    /// Shift acc to the right without changing bit 7 and put bit 0 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn sr(&mut self) {
        let carry = self.acc & 1 << 0 != 0;
        self.acc = (self.acc & 0x80) | self.acc.wrapping_shr(1);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }

    /// Shift acc to the right with 0 put in bit 7 and put bit 0 into the carry.
    ///
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Contains old bit 0
    pub fn srl(&mut self) {
        let carry = self.acc & 0x01 != 0;
        self.acc = self.acc.wrapping_shr(1);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }

    /// Swap upper and lower nibbles of acc
    /// 
    /// Flags Affected:
    /// - Z: Set if result is zero
    /// - N: Reset
    /// - H: Reset
    /// - C: Reset
    pub fn nibble_swap(&mut self) {
        self.acc = self.acc.wrapping_shl(4) | self.acc.wrapping_shr(4);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_sub();
        self.flags.reset_half();
        self.flags.reset_carry();
    }

    /// Complement acc [Flip all bits]
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Set
    /// - H: Set
    /// - C: Not affected
    pub fn complement(&mut self) {
        self.acc = !self.acc;

        self.flags.set_sub();
        self.flags.set_half();
    }

    /// Test bit
    ///
    /// Flags Affected:
    /// - Z: Set if bit b of register r is 0
    /// - N: Reset
    /// - H: Set
    /// - C: Not affected
    pub fn test_bit(&mut self, bit_index: u8) {
        assert!(bit_index < 8);

        self.flags.set_zero_if(self.acc & 1 << bit_index == 0);
        self.flags.reset_sub();
        self.flags.set_half();
    }

    /// Set bit to 1
    ///
    /// Flags Affected: NONE
    pub fn set_bit(&mut self, bit_index: u8) {
        assert!(bit_index < 8);
        self.acc = self.acc | (1 << bit_index);
    }

    /// Reset bit to 0
    ///
    /// Flags Affected: NONE
    pub fn reset_bit(&mut self, bit_index: u8) {
        assert!(bit_index < 8);
        self.acc = self.acc & !(1 << bit_index);
    }

    /// Decimal Adjust acc to obtain the bcd representation
    ///
    /// Z - Set if register acc is zero. 
    /// N - Not affected.
    /// H - Reset.
    /// C - Set or reset according to operation.
    pub fn daa(&mut self) {
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

        //   Flags, upper_range, lower_range, adjustment, new_carry |
        let lookup: [(Flags, std::ops::Range<u8>, std::ops::Range<u8>, u8, bool); 13] = [
            (Flags::NONE, (0x0..0x9), (0x0..0x9), 0x00, false), // R0
            (Flags::NONE, (0x0..0x8), (0xA..0xF), 0x06, false), // R1
            (Flags::H,    (0x0..0x9), (0x0..0x3), 0x06, false), // R2
            (Flags::NONE, (0xA..0xF), (0x0..0x9), 0x60, true),  // R3
            (Flags::NONE, (0x9..0xF), (0xA..0xF), 0x66, true),  // R4
            (Flags::H,    (0xA..0xF), (0x0..0x3), 0x66, true),  // R5
            (Flags::C,    (0x0..0x2), (0x0..0x9), 0x60, true),  // R6
            (Flags::C,    (0x0..0x2), (0xA..0xF), 0x66, true),  // R7
            (Flags::HC,   (0x0..0x3), (0x0..0x3), 0x66, true),  // R8

            (Flags::N,    (0x0..0x9), (0x0..0x9), 0x00, false), // R9
            (Flags::NH,   (0x0..0x8), (0x6..0xF), 0xFA, false), // R10
            (Flags::NC,   (0x7..0xF), (0x0..0x9), 0xA0, true),  // R11
            (Flags::NHC,  (0x6..0xF), (0x6..0xF), 0x9A, true),  // R12
        ];

        let upper: u8 = self.acc & 0xf;
        let lower: u8 = self.acc.wrapping_shr(4);

        let flags = self.flags & Flags::NHC;

        let mut adjustment: u8 = 0;
        let mut carry: bool = false;
        let mut found: bool = false;

        for (lookup_flags, upper_range, lower_range, next_adjustment, next_carry) in &lookup {
            if &flags == lookup_flags && upper_range.contains(&upper) && lower_range.contains(&lower) {
                adjustment = *next_adjustment;
                carry = *next_carry;
                found = true;
                break;
            }
        }

        if found {
            panic!("Invalid Value {:?} {:x}", self.flags, self.acc);
        }

        self.acc = self.acc.wrapping_add(adjustment);

        self.flags.set_zero_if(self.acc == 0);
        self.flags.reset_half();
        self.flags.set_carry_if(carry);
    }
}

#[test]
fn alu8_add_test() {
    let mut au1 = ALU8::new(0x00u8, Flags::N.bits());
    let mut au2 = ALU8::new(0x0fu8, 0x00u8);
    let mut au3 = ALU8::new(0xffu8, 0x00u8);
    let mut au4 = ALU8::new(0x00u8, 0x00u8);

    au1.add(255);
    au2.add(1);
    au3.add(1);
    au4.add(0);

    assert_eq!(0xff, au1.acc);
    assert_eq!(0x10, au2.acc);
    assert_eq!(0x00, au3.acc);
    assert_eq!(0x00, au4.acc);

    assert_eq!(0, au1.flags.bits());
    assert_eq!(Flags::H, au2.flags);
    assert_eq!(Flags::ZHC, au3.flags);
    assert_eq!(Flags::Z, au4.flags);
}

#[test]
fn alu8_adc_test() {
    let mut au1 = ALU8::new(0x00, Flags::N.bits());
    let mut au2 = ALU8::new(0x0f, 0x00);
    let mut au3 = ALU8::new(0xff, 0x00);
    let mut au4 = ALU8::new(0x00, 0x00);

    let mut au5 = ALU8::new(0x00, Flags::NC.bits());
    let mut au6 = ALU8::new(0x0f, Flags::C.bits());
    let mut au7 = ALU8::new(0xff, Flags::C.bits());
    let mut au8 = ALU8::new(0x00, Flags::C.bits());

    au1.adc(0xff);
    au2.adc(0x01);
    au3.adc(0x01);
    au4.adc(0x00);

    au5.adc(0xff);
    au6.adc(0x01);
    au7.adc(0x01);
    au8.adc(0x00);

    assert_eq!(0xff, au1.acc);
    assert_eq!(0x10, au2.acc);
    assert_eq!(0x00, au3.acc);
    assert_eq!(0x00, au4.acc);

    assert_eq!(0x00, au5.acc);
    assert_eq!(0x11, au6.acc);
    assert_eq!(0x01, au7.acc);
    assert_eq!(0x01, au8.acc);

    assert_eq!(Flags::NONE, au1.flags);
    assert_eq!(Flags::H, au2.flags);
    assert_eq!(Flags::ZHC, au3.flags);
    assert_eq!(Flags::Z, au4.flags);

    assert_eq!(Flags::ZHC, au5.flags);
    assert_eq!(Flags::H, au6.flags);
    assert_eq!(Flags::HC, au7.flags);
    assert_eq!(Flags::NONE, au8.flags);
}

#[test]
fn alu8_sub_test() {
    let mut alu1 = ALU8::new(0x00, 0);
    let mut alu2 = ALU8::new(0xff, 0);
    let mut alu3 = ALU8::new(0x80, 0);
    let mut alu4 = ALU8::new(0x80, 0);
    let mut alu5 = ALU8::new(0xff, 0);

    alu1.sub(0x00);
    alu2.sub(0xee);
    alu3.sub(0x18);
    alu4.sub(0x81);
    alu5.sub(0xff);

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0x11, alu2.acc);
    assert_eq!(0x68, alu3.acc);
    assert_eq!(0xff, alu4.acc);
    assert_eq!(0x00, alu5.acc);

    assert_eq!(Flags::ZN, alu1.flags);
    assert_eq!(Flags::N, alu2.flags);
    assert_eq!(Flags::NH, alu3.flags);
    assert_eq!(Flags::NHC, alu4.flags);
    assert_eq!(Flags::ZN, alu5.flags);
}

#[test]
fn alu8_sbc_test() {
    let mut alu1 = ALU8::new(0x00, 0x00);
    let mut alu2 = ALU8::new(0x01, Flags::C.bits());
    let mut alu3 = ALU8::new(0x00, Flags::C.bits());
    let mut alu4 = ALU8::new(0x80, Flags::C.bits());

    alu1.sbc(0x00);
    alu2.sbc(0x00);
    alu3.sbc(0xff);
    alu4.sbc(0x7f);

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0x00, alu2.acc);
    assert_eq!(0x00, alu3.acc);
    assert_eq!(0x00, alu4.acc);

    assert_eq!(Flags::ZN, alu1.flags);
    assert_eq!(Flags::ZN, alu2.flags);
    assert_eq!(Flags::ZNHC, alu3.flags);
    assert_eq!(Flags::ZNH, alu4.flags);
}

#[test]
fn alu8_inc_test() {
    let mut alu1 = ALU8::new(0x00, Flags::N.bits());
    let mut alu2 = ALU8::new(0x0f, 0x00);
    let mut alu3 = ALU8::new(0x8f, Flags::C.bits());
    let mut alu4 = ALU8::new(0xff, 0x00);
    let mut alu5 = ALU8::new(0xff, Flags::C.bits());

    alu1.inc();
    alu2.inc();
    alu3.inc();
    alu4.inc();
    alu5.inc();

    assert_eq!(0x01, alu1.acc);
    assert_eq!(0x10, alu2.acc);
    assert_eq!(0x90, alu3.acc);
    assert_eq!(0x00, alu4.acc);
    assert_eq!(0x00, alu5.acc);

    assert_eq!(Flags::NONE, alu1.flags);
    assert_eq!(Flags::H, alu2.flags);
    assert_eq!(Flags::HC, alu3.flags);
    assert_eq!(Flags::ZH, alu4.flags);
    assert_eq!(Flags::ZHC, alu5.flags);
}

#[test]
fn alu8_dec_test() {
    let mut alu1 = ALU8::new(0x01, Flags::C.bits());
    let mut alu2 = ALU8::new(0x00, 0x00);
    let mut alu3 = ALU8::new(0x00, Flags::C.bits());
    let mut alu4 = ALU8::new(0x10, 0x00);

    alu1.dec();
    alu2.dec();
    alu3.dec();
    alu4.dec();

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0xff, alu2.acc);
    assert_eq!(0xff, alu3.acc);
    assert_eq!(0x0f, alu4.acc);

    assert_eq!(Flags::ZNC, alu1.flags);
    assert_eq!(Flags::NH, alu2.flags);
    assert_eq!(Flags::NHC, alu3.flags);
    assert_eq!(Flags::NH, alu4.flags);
}

#[test]
fn alu8_and_test() {
    let mut alu1 = ALU8::new(0xff, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0xff, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu4 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());

    alu1.and(0x00);
    alu2.and(0xff);
    alu3.and(0b1010_1010);
    alu4.and(0b0101_0101);

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0xff, alu2.acc);
    assert_eq!(0b1010_1010, alu3.acc);
    assert_eq!(0x00, alu4.acc);

    assert_eq!(Flags::ZH, alu1.flags);
    assert_eq!(Flags::H, alu2.flags);
    assert_eq!(Flags::H, alu3.flags);
    assert_eq!(Flags::ZH, alu4.flags);
}

#[test]
fn alu8_or_test() {
    let mut alu1 = ALU8::new(0xff, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu4 = ALU8::new(0x00, Flags::ZNHC.bits());

    alu1.or(0xff);
    alu2.or(0b1010_1010);
    alu3.or(0b0101_0101);
    alu4.or(0x00);

    assert_eq!(0xff, alu1.acc);
    assert_eq!(0b1010_1010, alu2.acc);
    assert_eq!(0xff, alu3.acc);
    assert_eq!(0x00, alu4.acc);

    assert_eq!(Flags::NONE, alu1.flags);
    assert_eq!(Flags::NONE, alu2.flags);
    assert_eq!(Flags::NONE, alu3.flags);
    assert_eq!(Flags::Z, alu4.flags);
}

#[test]
fn alu8_xor_test() {
    let mut alu1 = ALU8::new(0xff, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x00, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu4 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu5 = ALU8::new(0x00, Flags::ZNHC.bits());

    alu1.xor(0xff);
    alu2.xor(0xff);
    alu3.xor(0b1010_1010);
    alu4.xor(0b0101_0101);
    alu5.xor(0x00);

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0xff, alu2.acc);
    assert_eq!(0x00, alu3.acc);
    assert_eq!(0xff, alu4.acc);
    assert_eq!(0x00, alu5.acc);

    assert_eq!(Flags::Z, alu1.flags);
    assert_eq!(Flags::NONE, alu2.flags);
    assert_eq!(Flags::Z, alu3.flags);
    assert_eq!(Flags::NONE, alu4.flags);
    assert_eq!(Flags::Z, alu5.flags);
}

#[test]
fn alu8_compare_test() {
    let mut alu1 = ALU8::new(0x00, 0);
    let mut alu2 = ALU8::new(0x00, 0);
    let mut alu3 = ALU8::new(0x00, 0);
    let mut alu4 = ALU8::new(0x01, 0);
    let mut alu5 = ALU8::new(0xff, 0);
    let mut alu6 = ALU8::new(0x88, 0);

    alu1.compare(0x00);
    alu2.compare(0x01);
    alu3.compare(0xff);
    alu4.compare(0x00);
    alu5.compare(0xfe);
    alu6.compare(0x79);

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0x00, alu2.acc);
    assert_eq!(0x00, alu3.acc);
    assert_eq!(0x01, alu4.acc);
    assert_eq!(0xff, alu5.acc);
    assert_eq!(0x88, alu6.acc);

    assert_eq!(Flags::ZN, alu1.flags); // Equal
    assert_eq!(Flags::NHC, alu2.flags); // Greater than
    assert_eq!(Flags::NHC, alu3.flags); // Greater than
    assert_eq!(Flags::N, alu4.flags); // Less than
    assert_eq!(Flags::N, alu5.flags); // Less than
    assert_eq!(Flags::NH, alu6.flags); // Less than
}

#[test]
fn alu8_rlc_test() {
    let mut alu1 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x00, Flags::NH.bits());

    alu1.rlc();
    assert_eq!(0b0101_0101, alu1.acc);
    assert_eq!(Flags::C, alu1.flags);

    alu1.rlc();
    assert_eq!(0b1010_1010, alu1.acc);
    assert_eq!(Flags::NONE, alu1.flags);

    alu2.rlc();
    assert_eq!(0x00, alu2.acc);
    assert_eq!(Flags::Z, alu2.flags);
}

#[test]
fn alu8_rrc_test() {
    let mut alu1 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x00, Flags::NH.bits());

    alu1.rrc();
    assert_eq!(0b0101_0101, alu1.acc);
    assert_eq!(Flags::NONE, alu1.flags);

    alu1.rrc();
    assert_eq!(0b1010_1010, alu1.acc);
    assert_eq!(Flags::C, alu1.flags);

    alu2.rrc();
    assert_eq!(0x00, alu2.acc);
    assert_eq!(Flags::Z, alu2.flags);
}

#[test]
fn alu8_rl_test() {
    let mut alu1 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x00, Flags::NH.bits());

    alu1.rl();
    assert_eq!(0b0101_0101, alu1.acc);
    assert_eq!(Flags::C, alu1.flags);

    alu1.rl();
    assert_eq!(0b1010_1011, alu1.acc);
    assert_eq!(Flags::NONE, alu1.flags);

    alu2.rl();
    assert_eq!(0x00, alu2.acc);
    assert_eq!(Flags::Z, alu2.flags);
}

#[test]
fn alu8_rr_test() {
    let mut alu1 = ALU8::new(0b1010_1010, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x00, Flags::NH.bits());

    alu1.rr();
    assert_eq!(0b1101_0101, alu1.acc);
    assert_eq!(Flags::NONE, alu1.flags);

    alu1.rr();
    assert_eq!(0b0110_1010, alu1.acc);
    assert_eq!(Flags::C, alu1.flags);

    alu2.rr();
    assert_eq!(0x00, alu2.acc);
    assert_eq!(Flags::Z, alu2.flags);
}

#[test]
fn alu8_sl_test() {
    let mut alu1 = ALU8::new(0x00, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x80, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0x81, 0x00);
    let mut alu4 = ALU8::new(0x20, 0x00);

    alu1.sl();
    alu2.sl();
    alu3.sl();
    alu4.sl();

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0x00, alu2.acc);
    assert_eq!(0x02, alu3.acc);
    assert_eq!(0x40, alu4.acc);

    assert_eq!(Flags::Z, alu1.flags);
    assert_eq!(Flags::ZC, alu2.flags);
    assert_eq!(Flags::C, alu3.flags);
    assert_eq!(Flags::NONE, alu4.flags);
}

#[test]
fn alu8_sr_test() {
    let mut alu1 = ALU8::new(0x00, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x80, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0x81, 0x00);
    let mut alu4 = ALU8::new(0x20, 0x00);
    let mut alu5 = ALU8::new(0x01, 0x00);

    alu1.sr();
    alu2.sr();
    alu3.sr();
    alu4.sr();
    alu5.sr();

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0xc0, alu2.acc);
    assert_eq!(0xc0, alu3.acc);
    assert_eq!(0x10, alu4.acc);
    assert_eq!(0x00, alu5.acc);

    assert_eq!(Flags::Z, alu1.flags);
    assert_eq!(Flags::NONE, alu2.flags);
    assert_eq!(Flags::C, alu3.flags);
    assert_eq!(Flags::NONE, alu4.flags);
    assert_eq!(Flags::ZC, alu5.flags);
}

#[test]
fn alu8_srl_test() {
    let mut alu1 = ALU8::new(0x00, Flags::ZNHC.bits());
    let mut alu2 = ALU8::new(0x80, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0x81, 0x00);
    let mut alu4 = ALU8::new(0x20, 0x00);
    let mut alu5 = ALU8::new(0x01, 0x00);

    alu1.srl();
    alu2.srl();
    alu3.srl();
    alu4.srl();
    alu5.srl();

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0x40, alu2.acc);
    assert_eq!(0x40, alu3.acc);
    assert_eq!(0x10, alu4.acc);
    assert_eq!(0x00, alu5.acc);

    assert_eq!(Flags::Z, alu1.flags);
    assert_eq!(Flags::NONE, alu2.flags);
    assert_eq!(Flags::C, alu3.flags);
    assert_eq!(Flags::NONE, alu4.flags);
    assert_eq!(Flags::ZC, alu5.flags);
}

#[test]
fn alu8_nibble_swap_test() {
    let mut alu1 = ALU8::new(0x00, Flags::NHC.bits());
    let mut alu2 = ALU8::new(0b1111_0000, Flags::ZNHC.bits());
    let mut alu3 = ALU8::new(0b1010_0101, Flags::ZNHC.bits());

    alu1.nibble_swap();
    alu2.nibble_swap();
    alu3.nibble_swap();

    assert_eq!(0x00, alu1.acc);
    assert_eq!(0b0000_1111, alu2.acc);
    assert_eq!(0b0101_1010, alu3.acc);

    assert_eq!(Flags::Z, alu1.flags);
    assert_eq!(Flags::NONE, alu2.flags);
    assert_eq!(Flags::NONE, alu3.flags);
}

#[test]
pub fn alu8_complement_test() {
    let mut alu1 = ALU8::new(0x00, Flags::ZC.bits());
    let mut alu2 = ALU8::new(0xff, 0x00);
    let mut alu3 = ALU8::new(0b1100_0011, Flags::Z.bits());
    let mut alu4 = ALU8::new(0b1010_1010, Flags::C.bits());

    alu1.complement();
    alu2.complement();
    alu3.complement();
    alu4.complement();

    assert_eq!(0xff, alu1.acc);
    assert_eq!(0x00, alu2.acc);
    assert_eq!(0b0011_1100, alu3.acc);
    assert_eq!(0b0101_0101, alu4.acc);

    assert_eq!(Flags::ZNHC, alu1.flags);
    assert_eq!(Flags::NH, alu2.flags);
    assert_eq!(Flags::ZNH, alu3.flags);
    assert_eq!(Flags::NHC, alu4.flags);
}

#[test]
pub fn alu8_test_bit_test() {
    let mut alu = ALU8::new(0b0011_1010, Flags::N.bits());

    alu.test_bit(0);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::ZH);

    alu.test_bit(1);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::H);

    alu.test_bit(2);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::ZH);

    alu.test_bit(3);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::H);

    alu.test_bit(4);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::H);

    alu.test_bit(5);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::H);

    alu.test_bit(6);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::ZH);

    alu.test_bit(7);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::ZH);

    alu.flags.set_carry();

    alu.test_bit(7);
    assert_eq!(alu.acc, 0b0011_1010);
    assert_eq!(alu.flags, Flags::ZHC);
}

#[test]
pub fn alu8_set_bit_test() {
    let mut alu = ALU8::new(0x00, Flags::ZNHC.bits());

    alu.set_bit(0);
    assert_eq!(0b0000_0001, alu.acc);

    alu.set_bit(1);
    assert_eq!(0b0000_0011, alu.acc);

    alu.set_bit(2);
    assert_eq!(0b0000_0111, alu.acc);

    alu.set_bit(3);
    assert_eq!(0b0000_1111, alu.acc);

    alu.set_bit(4);
    assert_eq!(0b0001_1111, alu.acc);

    alu.set_bit(5);
    assert_eq!(0b0011_1111, alu.acc);

    alu.set_bit(6);
    assert_eq!(0b0111_1111, alu.acc);

    alu.set_bit(7);
    assert_eq!(0b1111_1111, alu.acc);
}

#[test]
pub fn alu8_reset_bit_test() {
    let mut alu = ALU8::new(0xff, Flags::ZNHC.bits());

    alu.reset_bit(0);
    assert_eq!(0b1111_1110, alu.acc);

    alu.reset_bit(1);
    assert_eq!(0b1111_1100, alu.acc);

    alu.reset_bit(2);
    assert_eq!(0b1111_1000, alu.acc);

    alu.reset_bit(3);
    assert_eq!(0b1111_0000, alu.acc);

    alu.reset_bit(4);
    assert_eq!(0b1110_0000, alu.acc);

    alu.reset_bit(5);
    assert_eq!(0b1100_0000, alu.acc);

    alu.reset_bit(6);
    assert_eq!(0b1000_0000, alu.acc);

    alu.reset_bit(7);
    assert_eq!(0b0000_0000, alu.acc);

    assert_eq!(Flags::ZNHC, alu.flags);
}
