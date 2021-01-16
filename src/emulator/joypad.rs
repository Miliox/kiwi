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

        const PAD_OUT = 0b0011_0000;
        const PAD_IN  = 0b0000_1111;
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

#[derive(Default)]
pub struct Joypad {
    regs: JoypadRegs,
    keys: JoypadKeys,
    interruption_requested: bool,
}

#[allow(dead_code)]
impl Joypad {
    pub fn get_p1(&self) -> u8 {
        self.regs.bits()
    }

    pub fn set_p1(&mut self, data: u8) {
        self.regs = JoypadRegs::from_bits(data).unwrap();
        self.update();
    }

    pub fn update(&mut self) {
        let pad_selector = self.regs & JoypadRegs::PAD_OUT;
        self.regs.remove(JoypadRegs::PAD_IN);

        if pad_selector.contains(JoypadRegs::P15_OUT |JoypadRegs::P14_OUT) {
            self.regs.insert(JoypadRegs::PAD_IN);
        } else if pad_selector.contains(JoypadRegs::P15_OUT) {
            self.regs.insert(JoypadRegs::from_bits(!self.keys.bits().wrapping_shr(4) & 0x0f).unwrap());
        } else if pad_selector.contains(JoypadRegs::P14_OUT) {
            self.regs.insert(JoypadRegs::from_bits(!self.keys.bits() & 0x0f).unwrap());
        }  else {
            self.regs.insert(JoypadRegs::PAD_IN);
        }
    }

    pub fn press_up(&mut self) {
        self.keys.insert(JoypadKeys::UP);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_up(&mut self) {
        self.keys.remove(JoypadKeys::UP);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_down(&mut self) {
        self.keys.insert(JoypadKeys::DOWN);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_down(&mut self) {
        self.keys.remove(JoypadKeys::DOWN);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_left(&mut self) {
        self.keys.insert(JoypadKeys::LEFT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_left(&mut self) {
        self.keys.remove(JoypadKeys::LEFT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_right(&mut self) {
        self.keys.insert(JoypadKeys::RIGHT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_right(&mut self) {
        self.keys.remove(JoypadKeys::RIGHT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_a(&mut self) {
        self.keys.insert(JoypadKeys::A);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_a(&mut self) {
        self.keys.remove(JoypadKeys::A);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_b(&mut self) {
        self.keys.insert(JoypadKeys::B);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_b(&mut self) {
        self.keys.remove(JoypadKeys::B);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_select(&mut self) {
        self.keys.insert(JoypadKeys::SELECT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_select(&mut self) {
        self.keys.remove(JoypadKeys::SELECT);
        self.update();
        self.interruption_requested = true;
    }

    pub fn press_start(&mut self) {
        self.keys.insert(JoypadKeys::START);
        self.update();
        self.interruption_requested = true;
    }

    pub fn release_start(&mut self) {
        self.keys.remove(JoypadKeys::START);
        self.update();
        self.interruption_requested = true;
    }

    pub fn interruption_requested(&self) -> bool {
        self.interruption_requested
    }

    pub fn reset_interruption_requested(&mut self) {
        self.interruption_requested = false;
    }
}

#[test]
fn joypad_down_key_test() {
    assert_eq!(JoypadKeys::from_bits(0x94).unwrap(), JoypadKeys::A | JoypadKeys::START | JoypadKeys::UP);
}

#[test]
fn joypad_p1_read_test() {
    let mut joypad = Joypad::default();
    joypad.keys = JoypadKeys::A | JoypadKeys::B | JoypadKeys::LEFT;

    // READ BUTTONS
    joypad.regs.insert(JoypadRegs::P15_OUT);
    joypad.regs.remove(JoypadRegs::P14_OUT);
    joypad.update();
    assert_eq!(JoypadRegs::P15_OUT | JoypadRegs::P13_IN | JoypadRegs::P12_IN, joypad.regs);

    // READ DIRECTION
    joypad.regs.remove(JoypadRegs::P15_OUT);
    joypad.regs.insert(JoypadRegs::P14_OUT);
    joypad.update();
    assert_eq!(JoypadRegs::P14_OUT | JoypadRegs::P13_IN | JoypadRegs::P12_IN | JoypadRegs::P10_IN, joypad.regs);
}