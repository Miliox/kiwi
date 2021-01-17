#[derive(Copy,Clone,Default)]
pub struct Regs {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

#[allow(dead_code)]
impl Regs {
    pub fn a(&self) -> u8 { self.a }
    pub fn f(&self) -> u8 { self.f }
    pub fn b(&self) -> u8 { self.b }
    pub fn c(&self) -> u8 { self.c }
    pub fn d(&self) -> u8 { self.d }
    pub fn e(&self) -> u8 { self.e }
    pub fn h(&self) -> u8 { self.h }
    pub fn l(&self) -> u8 { self.l }

    pub fn af(&self) -> u16 { u16::from_be_bytes([self.a, self.f]) }
    pub fn bc(&self) -> u16 { u16::from_be_bytes([self.b, self.c]) }
    pub fn de(&self) -> u16 { u16::from_be_bytes([self.d, self.e]) }
    pub fn hl(&self) -> u16 { u16::from_be_bytes([self.h, self.l]) }
    pub fn sp(&self) -> u16 { self.sp }
    pub fn pc(&self) -> u16 { self.pc }

    pub fn set_a(&mut self, r: u8) { self.a = r; }
    pub fn set_f(&mut self, r: u8) { self.f = r; }
    pub fn set_b(&mut self, r: u8) { self.b = r; }
    pub fn set_c(&mut self, r: u8) { self.c = r; }
    pub fn set_d(&mut self, r: u8) { self.d = r; }
    pub fn set_e(&mut self, r: u8) { self.e = r; }
    pub fn set_h(&mut self, r: u8) { self.h = r; }
    pub fn set_l(&mut self, r: u8) { self.l = r; }

    pub fn set_af(&mut self, r: u16) {
        let bytes = r.to_be_bytes();
        self.a = bytes[0];
        self.f = bytes[1];
    }

    pub fn set_bc(&mut self, r: u16) {
        let bytes = r.to_be_bytes();
        self.b = bytes[0];
        self.c = bytes[1];
    }

    pub fn set_de(&mut self, r: u16) {
        let bytes = r.to_be_bytes();
        self.d = bytes[0];
        self.e = bytes[1];
    }

    pub fn set_hl(&mut self, r: u16) {
        let bytes = r.to_be_bytes();
        self.h = bytes[0];
        self.l = bytes[1];
    }

    pub fn set_sp(&mut self, r: u16) {
        self.sp = r;
    }

    pub fn set_pc(&mut self, r: u16) {
        self.pc = r;
    }
}

impl std::fmt::Debug for Regs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Regs")
         .field("a", &self.a())
         .field("f", &self.f())
         .field("b", &self.b())
         .field("c", &self.c())
         .field("d", &self.d())
         .field("e", &self.e())
         .field("h", &self.h())
         .field("l", &self.l())
         .field("sp", &self.sp())
         .field("pc", &self.pc())
         .finish()
    }
}