bitflags! {
    #[derive(Default)]
    pub struct JoypadRegs: u8 {
        const P17_OUT = 0b1000_0000; // UNUSED
        const P16_OUT = 0b0100_0000; // UNUSED
        const P15_OUT = 0b0010_0000; // Button Keys
        const P14_OUT = 0b0001_0000; // Direction Keys
        const P13_IN  = 0b0000_1000;
        const P12_IN  = 0b0000_0100;
        const P11_IN  = 0b0000_0010;
        const P10_IN  = 0b0000_0001;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct JoypadKeys: u8 {
        const START  = 0b1000_0000;
        const SELECT = 0b0100_0000;
        const B      = 0b0010_0000;
        const A      = 0b0001_0000;

        const DOWN   = 0b0000_1000;
        const UP     = 0b0000_0100;
        const LEFT   = 0b0000_0010;
        const RIGHT  = 0b0000_0001;
    }
}

#[allow(dead_code)]
impl JoypadRegs {
    pub fn merge_keys(&mut self, keys: JoypadKeys) {
        self.remove(Self::P13_IN | Self::P12_IN | Self::P11_IN | Self::P10_IN);
        if self.contains(Self::P15_OUT | Self::P14_OUT) {
            self.insert(Self::P13_IN | Self::P12_IN | Self::P11_IN | Self::P10_IN);
        } else if self.contains(Self::P15_OUT) {
            self.insert(Self::from_bits(!keys.bits().wrapping_shr(4) & 0x0f).unwrap());
        } else if self.contains(Self::P14_OUT) {
            self.insert(Self::from_bits(!keys.bits() & 0x0f).unwrap());
        }  else {
            self.insert(Self::P13_IN | Self::P12_IN | Self::P11_IN | Self::P10_IN);
        }
    }
}

#[test]
fn joypad_down_key_test() {
    assert_eq!(JoypadKeys::from_bits(0x94).unwrap(), JoypadKeys::A | JoypadKeys::START | JoypadKeys::UP);
}

#[test]
fn joypad_p1_read_test() {
    let mut p1 = JoypadRegs::default();
    let keys = JoypadKeys::A | JoypadKeys::B | JoypadKeys::LEFT;

    // READ BUTTONS
    p1.insert(JoypadRegs::P15_OUT);
    p1.remove(JoypadRegs::P14_OUT);
    p1.merge_keys(keys);
    assert_eq!(JoypadRegs::P15_OUT | JoypadRegs::P13_IN | JoypadRegs::P12_IN, p1);

    // READ DIRECTION
    p1.remove(JoypadRegs::P15_OUT);
    p1.insert(JoypadRegs::P14_OUT);
    p1.merge_keys(keys);
    assert_eq!(JoypadRegs::P14_OUT | JoypadRegs::P13_IN | JoypadRegs::P12_IN | JoypadRegs::P10_IN, p1);
}