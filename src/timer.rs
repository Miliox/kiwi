use crate::ticks::TickConsumer;

const COUNTER_DIV: [u64; 4] = [1024, 16, 64, 256];
const DIVIDER_DIV: u64 = 256;

pub struct Timer {
    enable: bool,

    control: u8,
    counter: u8,
    divider: u8,
    modulo: u8,

    counter_acc: u64,
    counter_div: u64,
    divider_acc: u64,
}

impl Timer {
    pub fn control(&self) -> u8 {
        self.control
    }

    pub fn set_control(&mut self, control: u8) {
        self.enable = control & 1 << 3 != 0;
        self.counter_div = COUNTER_DIV[(control & 0x3) as usize];
        self.control = control
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn set_counter(&mut self, counter: u8) {
        self.counter = counter;
    }

    pub fn divider(&self) -> u8 {
        self.divider
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0
    }

    pub fn modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, modulo: u8) {
        self.modulo = modulo
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            enable: true,

            control: 0xff,
            counter: 0,
            divider: 0,
            modulo: 0,

            counter_acc: 0,
            counter_div: COUNTER_DIV[3],
            divider_acc: 0,
        }
    }
}

impl TickConsumer for Timer {
    fn sync(&mut self, ticks: u64) {
        if self.enable {
            self.counter_acc += ticks;
            self.divider_acc += ticks;

            if self.counter_acc >= self.counter_div {
                self.counter_acc -= self.counter_div;

                let (counter, overflow) = self.counter.overflowing_add(1);

                if overflow {
                    self.counter = self.modulo
                } else {
                    self.counter = counter
                }
            }

            if self.divider_acc >= DIVIDER_DIV {
                self.divider_acc -= DIVIDER_DIV;
                self.divider = self.divider.wrapping_add(1);
            }
        }
    }
}

#[test]
fn sync_test() {
    let mut timer = Timer::default();

    for _ in 0..64 {
        assert_eq!(0, timer.counter);
        assert_eq!(0, timer.divider);
        timer.sync(4)
    }
    assert_eq!(1, timer.counter);
    assert_eq!(1, timer.divider);
    assert_eq!(0, timer.counter_acc);
    assert_eq!(0, timer.divider_acc);

    timer.set_control(0b1111_1100 | 2);

    for i in 0..64 {
        assert_eq!(1 + i / 16, timer.counter);
        assert_eq!(1, timer.divider);
        timer.sync(4)
    }

    assert_eq!(5, timer.counter);
    assert_eq!(2, timer.divider);

    timer.set_control(0b1111_1100 | 1);

    for i in 0..64 {
        assert_eq!(5 + i / 4, timer.counter);
        assert_eq!(2, timer.divider);
        timer.sync(4)
    }

    assert_eq!(21, timer.counter);
    assert_eq!(3, timer.divider);

    timer.set_control(0b1111_1100 | 0);

    for i in 0..=255 {
        assert_eq!(21, timer.counter);
        assert_eq!(3 + i / 64, timer.divider);
        timer.sync(4)
    }

    assert_eq!(22, timer.counter);
    assert_eq!(7, timer.divider);
}