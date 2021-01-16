use crate::types::TickConsumer;
use crate::types::TICKS_PER_SECOND;

const SHIFT_PER_TICKS: u64 = TICKS_PER_SECOND / 8192;

pub struct Serial {
    control: u8,
    data: u8,

    shift_bits: u64,
    shift_ticks: u64,
    transfering: bool,
    transfer_complete: bool,
}

impl Serial {
    pub fn control(&self) -> u8 {
        self.control
    }

    pub fn set_control(&mut self, control: u8) {
        self.control = control;
        
        self.shift_bits = 8;
        self.shift_ticks = 0;
        self.transfering = (control & 0b1000_000) != 0;
    }

    pub fn data(&self) -> u8 {
        self.data
    }

    pub fn set_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn transfer_complete(&self) -> bool {
        self.transfer_complete
    }
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            control: 0b0000_0001,
            data: 0,

            shift_bits: 0,
            shift_ticks: 0,
            transfering: false,
            transfer_complete: false,
        }
    }
}

impl TickConsumer for Serial {
    fn step(&mut self, ticks: u64) {
        self.transfer_complete = false;

        if !self.transfering {
            return
        }

        self.shift_ticks += ticks;
        if self.shift_bits < SHIFT_PER_TICKS {
            return
        }

        self.data = self.data.wrapping_shl(1);
        self.shift_bits -= 1;

        if self.shift_bits <= 0 {
            self.transfering = false;
            self.transfer_complete = true;
        }
    }
}
