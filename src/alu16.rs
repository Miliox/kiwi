use crate::flags::Flags;

#[derive(Default)]
/// Gameboy (LR35902) 8 Bit Arithmetic Logic Unit
pub struct ALU16 {
    /// Accumulator
    pub acc: u16,

    /// Flags [ZNHC----]
    pub flags: Flags,
}

#[allow(dead_code)]
impl ALU16 {
    pub fn new(acc: u16, flags: u8) -> Self {
        ALU16 {
            acc: acc,
            flags: Flags::from(flags),
        }
    }

    /// Add arg to acc
    ///
    /// Flags Affected:
    /// - Z: Not Affected
    /// - N: Reset
    /// - H: Set if carry from bit 11
    /// - C: Set if carry from bit 15
    pub fn add(&mut self, arg: u16) {
        let (_, half) = self.acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
        let (acc, carry) = self.acc.overflowing_add(arg);

        self.flags.set_zero_if(acc == 0);
        self.flags.reset_sub();
        self.flags.set_half_if(half);
        self.flags.set_carry_if(carry);

        self.acc = acc;
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Not affected
    /// - H: Not affected
    /// - C: Not affected
    pub fn inc(&mut self) {
        self.acc = self.acc.wrapping_add(1);
    }

    /// Increment acc by one
    ///
    /// Flags Affected:
    /// - Z: Not affected
    /// - N: Not affected
    /// - H: Not affected
    /// - C: Not affected
    pub fn dec(&mut self) {
        self.acc = self.acc.wrapping_sub(1);
    }
}