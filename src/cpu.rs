use crate::mmu::Mmu;
use crate::alu::*;

#[allow(dead_code)]
#[derive(Default)]
pub struct Cpu {
    a: u8,       // accumulator register
    f: u8,       // flags register
    b: u8,       // general purpose register (MSB)
    c: u8,       // general purpose register (LSB) commonly used as zero address index
    d: u8,       // general purpose register (MSB)
    e: u8,       // general purpose register (LSB)
    h: u8,       // general purpose register (MSB) commonly used as address pointer
    l: u8,       // general purpose register (LSB) commonly used as address pointer
    sp : u16,    // stack pointer register
    pc : u16,    // program counter register
    clock: u64,  // accumulated clock counter
    ie_flag: bool, // interrupt enable flag
    if_flag: u8,   // interrupt flags flag
}

const INSTRUCTION_SIZE: [u8; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1,    // 0x00 ~ 0x0F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x10 ~ 0x1F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x20 ~ 0x2F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x30 ~ 0x3F

    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x40 ~ 0x4F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x50 ~ 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x60 ~ 0x6F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x70 ~ 0x7F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x80 ~ 0x8F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x90 ~ 0x9F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xA0 ~ 0xAF
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xB0 ~ 0xBF

    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 2, 3, 3, 2, 1,    // 0xC0 ~ 0xCF
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1,    // 0xD0 ~ 0xDF
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xE0 ~ 0xEF
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xF0 ~ 0xFF
];

const INSTRUCTION_TICKS: [u8; 256] = [
//  x0  x1  x2  x3  x4  x5  x6  x7  x8  x9  xA  xB  xC  xD  xE  xF
    4,  12, 8,  8,  4,  4,  8,  4,  20, 8,  8,  8,  4,  4,  8,  4,    // 0x00 ~ 0x0F
    4,  12, 8,  8,  4,  4,  8,  4,  12, 8,  8,  8,  4,  4,  8,  4,    // 0x10 ~ 0x1F
    8,  12, 8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x20 ~ 0x2F
    8,  12, 8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x30 ~ 0x3F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x40 ~ 0x4F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x50 ~ 0x5F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x60 ~ 0x6F
    8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x70 ~ 0x7F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x80 ~ 0x8F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x90 ~ 0x9F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xA0 ~ 0xAF
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xB0 ~ 0xBF

    8,  12, 12, 16, 12, 16, 8,  16, 8,  16, 12, 8,  12, 24, 8,  16,    // 0xC0 ~ 0xCF
    8,  12, 12, 4,  12, 16, 8,  16, 8,  16, 12, 4,  12, 4,  8,  16,    // 0xD0 ~ 0xDF
    12, 12, 8,  4,  4,  16, 8,  16, 16, 4,  16, 4,  4,  4,  8,  16,    // 0xE0 ~ 0xEF
    12, 12, 8,  4,  4,  16, 8,  16, 12, 8,  16, 4,  4,  4,  8,  16,    // 0xF0 ~ 0xFF
];

#[allow(dead_code)]
const INSTRUCTION_MNEMONIC: [&str; 256] = [
    // 0 - opcode; 1 - arg1; 2 - arg2; 3 - pc
    // 0x00 ~ 0x0F
    "NOP           ;",
    "LD BC, $0000  ;",
    "LD (BC), A    ;",
    "INC BC        ;",
    "INC B         ;",
    "DEC B         ;",
    "LD B, $00     ;",
    "RLCA          ;",
    "LD ($0000),SP ;",
    "ADD HL, BC    ;",
    "LD A, (BC)    ;",
    "DEC BC        ;",
    "INC C         ;",
    "DEC C         ;",
    "LD C, $00     ;",
    "RRCA          ;",

    // 0x10 ~ 0x1F
    "STOP 0        ;",
    "LD DE, $0000  ;",
    "LD (DE), A    ;",
    "INC DE        ;",
    "INC D         ;",
    "DEC D         ;",
    "LD D, $00     ;",
    "RLA           ;",
    "JR $00        ;",
    "ADD HL, DE    ;",
    "LD A, (DE)    ;",
    "DEC DE        ;",
    "INC E         ;",
    "DEC E         ;",
    "LD E, $00     ;",
    "RRA           ;",

    // 0x20 ~ 0x2F
    "JR NZ $00     ;",
    "LD HL, $0000  ;",
    "LDI (HL), A   ;",
    "INC HL        ;",
    "INC H         ;",
    "DEC H         ;",
    "LD H, $00     ;",
    "DAA           ;",
    "JR Z $00      ;",
    "ADD HL, HL    ;",
    "LDI A, (HL)   ;",
    "DEC HL        ;",
    "INC L         ;",
    "DEC L         ;",
    "LD L, $00     ;",
    "CPL           ;",

    // 0x30 ~ 0x3F
    "JR NC $00     ;",
    "LD SP, $0000  ;",
    "LDD (HL), A   ;",
    "INC SP        ;",
    "INC (HL)      ;",
    "DEC (HL)      ;",
    "LD (HL), $00  ;",
    "SCF           ;",
    "JR C $00      ;",
    "ADD HL, SP    ;",
    "LDD A, (HL)   ;",
    "DEC SP        ;",
    "INC A         ;",
    "DEC A         ;",
    "LD A, $00     ;",
    "CCF           ;",

    // 0x40 ~ 0x4F
    "LD B, B       ;",
    "LD B, C       ;",
    "LD B, D       ;",
    "LD B, E       ;",
    "LD B, H       ;",
    "LD B, L       ;",
    "LD B, (HL)    ;",
    "LD B, A       ;",
    "LD C, B       ;",
    "LD C, C       ;",
    "LD C, D       ;",
    "LD C, E       ;",
    "LD C, H       ;",
    "LD C, L       ;",
    "LD C, (HL)    ;",
    "LD C, A       ;",

    // 0x50 ~ 0x5F
    "LD D, B       ;",
    "LD D, C       ;",
    "LD D, D       ;",
    "LD D, E       ;",
    "LD D, H       ;",
    "LD D, L       ;",
    "LD D, (HL)    ;",
    "LD D, A       ;",
    "LD E, B       ;",
    "LD E, C       ;",
    "LD E, D       ;",
    "LD E, E       ;",
    "LD E, H       ;",
    "LD E, L       ;",
    "LD E, (HL)    ;",
    "LD E, A       ;",

    // 0x60 ~ 0x6F
    "LD H, B       ;",
    "LD H, C       ;",
    "LD H, D       ;",
    "LD H, E       ;",
    "LD H, H       ;",
    "LD H, L       ;",
    "LD H, (HL)    ;",
    "LD H, A       ;",
    "LD L, B       ;",
    "LD L, C       ;",
    "LD L, D       ;",
    "LD L, E       ;",
    "LD L, H       ;",
    "LD L, L       ;",
    "LD L, (HL)    ;",
    "LD L, A       ;",

    // 0X70 ~ 0X7F
    "LD (HL), B    ;",
    "LD (HL), C    ;",
    "LD (HL), D    ;",
    "LD (HL), E    ;",
    "LD (HL), H    ;",
    "LD (HL), L    ;",
    "LD (HL), (HL) ;",
    "LD (HL), A    ;",
    "LD A, B       ;",
    "LD A, C       ;",
    "LD A, D       ;",
    "LD A, E       ;",
    "LD A, H       ;",
    "LD A, L       ;",
    "LD A, (HL)    ;",
    "LD A, A       ;",

    // 0X80 ~ 0X8F
    "ADD A, B      ;",
    "ADD A, C      ;",
    "ADD A, D      ;",
    "ADD A, E      ;",
    "ADD A, H      ;",
    "ADD A, L      ;",
    "ADD A, (HL)   ;",
    "ADD A, A      ;",
    "ADC A, B      ;",
    "ADC A, C      ;",
    "ADC A, D      ;",
    "ADC A, E      ;",
    "ADC A, H      ;",
    "ADC A, L      ;",
    "ADC A, (HL)   ;",
    "ADC A, A      ;",

    // 0X90 ~ 0X9F
    "SUB A, B      ;",
    "SUB A, C      ;",
    "SUB A, D      ;",
    "SUB A, E      ;",
    "SUB A, H      ;",
    "SUB A, L      ;",
    "SUB A, (HL)   ;",
    "SUB A, A      ;",
    "SBC A, B      ;",
    "SBC A, C      ;",
    "SBC A, D      ;",
    "SBC A, E      ;",
    "SBC A, H      ;",
    "SBC A, L      ;",
    "SBC A, (HL)   ;",
    "SBC A, A      ;",

    // 0XA0 ~ 0XAF
    "AND A, B      ;",
    "AND A, C      ;",
    "AND A, D      ;",
    "AND A, E      ;",
    "AND A, H      ;",
    "AND A, L      ;",
    "AND A, (HL)   ;",
    "AND A, A      ;",
    "XOR A, B      ;",
    "XOR A, C      ;",
    "XOR A, D      ;",
    "XOR A, E      ;",
    "XOR A, H      ;",
    "XOR A, L      ;",
    "XOR A, (HL)   ;",
    "XOR A, A      ;",

    // 0XB0 ~ 0XBF
    "OR A, B       ;",
    "OR A, C       ;",
    "OR A, D       ;",
    "OR A, E       ;",
    "OR A, H       ;",
    "OR A, L       ;",
    "OR A, (HL)    ;",
    "OR A, A       ;",
    "CP A, B       ;",
    "CP A, C       ;",
    "CP A, D       ;",
    "CP A, E       ;",
    "CP A, H       ;",
    "CP A, L       ;",
    "CP A, (HL)    ;",
    "CP A, A       ;",

    // 0XC0 ~ 0XCF
    "RET NZ        ;",
    "POP BC        ;",
    "JP NZ $0000   ;",
    "JP $0000      ;",
    "CALL NZ $0000 ;",
    "PUSH BC       ;",
    "ADD A, $00    ;",
    "RST $00       ;",
    "RET Z         ;",
    "RET           ;",
    "JP Z $0000    ;",
    "PREFIX CB     ;",
    "CALL Z $0000  ;",
    "CALL $0000    ;",
    "ADC A, $$00   ;",
    "RST $08       ;",

    // 0XD0 ~ 0XDF
    "RET NC        ;",
    "POP DE        ;",
    "JP NC $0000   ;",
    "[D3] - INVALID;",
    "CALL NC $0000 ;",
    "PUSH DE       ;",
    "SUB $$00      ;",
    "RST $10       ;",
    "RET C         ;",
    "RETI          ;",
    "JP C $0000    ;",
    "[DB] - INVALID;",
    "CALL C $0000  ;",
    "[DD] - INVALID;",
    "SBC A, $$00   ;",
    "RST $18       ;",

    // 0XE0 ~ 0XEF
    "LDH ($FF$00),A;",
    "POP HL        ;",
    "LD (C), A     ;",
    "[E3] - INVALID;",
    "[E4] - INVALID;",
    "PUSH HL       ;",
    "AND $$00      ;",
    "RST $20       ;",
    "ADD SP,$$00   ;",
    "JP HL         ;",
    "LD ($0000), A ;",
    "[EB] - INVALID;",
    "[EC] - INVALID;",
    "[ED] - INVALID;",
    "XOR $$00      ;",
    "RST $28       ;",

    // 0XF0 ~ 0XFF
    "LDH A,($FF$00);",
    "POP AF        ;",
    "LD A, (C)     ;",
    "DI            ;",
    "[F4] - INVALID;",
    "PUSH AF       ;",
    "OR $$00       ;",
    "RST $30       ;",
    "LD HL, SP+$$00;",
    "LD SP, HL     ;",
    "LD A, ($0000) ;",
    "EI            ;",
    "[FC] - INVALID;",
    "[FD] - INVALID;",
    "CP $$00       ;",
    "RST $38       ;",
];

#[allow(dead_code)]
impl Cpu {
    fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn cycle(&mut self, mmu: &mut dyn Mmu) -> u64 {
        let opcode = mmu.read_byte(self.pc);

        match INSTRUCTION_SIZE[opcode as usize] {
            1 => {
                println!("{0} ${1:04x}", INSTRUCTION_MNEMONIC[opcode as usize], self.pc)
            }
            2 => {
                let param = format!("{0:02x}", mmu.read_byte(self.pc + 1));
                let mnemonic = INSTRUCTION_MNEMONIC[opcode as usize].replace("$00", &param);
                println!("{0} ${1:04x}", mnemonic, self.pc)
            }
            3 => {
                let param = format!("${1:02x}{0:02x}", mmu.read_byte(self.pc + 1), mmu.read_byte(self.pc + 2));
                let mnemonic = INSTRUCTION_MNEMONIC[opcode as usize].replace("$0000", &param);
                println!("{0} ${1:04x}", mnemonic, self.pc)
            }
            _ => {
                println!("{0} ${1:04x}", INSTRUCTION_MNEMONIC[opcode as usize], self.pc)
            }
        }

        let mut next_pc = self.pc + INSTRUCTION_SIZE[opcode as usize] as u16;

        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC, $0000
                self.b = mmu.read_byte(self.pc + 2);
                self.c = mmu.read_byte(self.pc + 1);
            }
            0x02 => {
                // LD (BC), A
                mmu.write_byte(self.bc(), self.a);
            },
            0x03 => {
                // INC BC
                increment_16bit_register(&mut self.b, &mut self.c);
            }
            0x04 => {
                // INC B
                increment_8bit_register(&mut self.b, &mut self.f);
            }
            0x05 => {
                // DEC B
                decrement_8bit_register(&mut self.b, &mut self.f);
            }
            0x06 => {
                // LD B, $00
                self.b = mmu.read_byte(self.pc + 1);
            }
            0x07 => {
                // RLCA
                rotate_left_and_store_carry(&mut self.a, &mut self.f);
            }
            0x08 => {
                // LD ($0000),SP
                mmu.write_word(mmu.read_word(self.pc + 1), self.sp);
            }
            0x09 => {
                // ADD HL, BC
                add_16bit_registers(&mut self.h, &mut self.l, self.b, self.c, &mut self.f);
            }
            0x0A => {
                // LD A, (BC)
                self.a = mmu.read_byte(self.bc());
            }
            0x0B => {
                // DEC BC
                decrement_16bit_register(&mut self.b, &mut self.c);
            }
            0x0C => {
                // INC C
                increment_8bit_register(&mut self.c, &mut self.f);
            }
            0x0D => {
                // DEC C
                decrement_8bit_register(&mut self.c, &mut self.f);
            }
            0x0E => {
                // LD C, $00
                self.c = mmu.read_byte(self.pc + 1);
            }
            0x0F => {
                // RRCA
                rotate_right_and_store_carry(&mut self.a, &mut self.f);
            }
            0x10 => {
                // STOP 0
            }
            0x11 => {
                // LD DE, $0000
                self.e = mmu.read_byte(self.pc + 1);
                self.d = mmu.read_byte(self.pc + 2);
            }
            0x12 => {
                // LD (DE), A
                mmu.write_byte(self.de(), self.a);
            }
            0x13 => {
                // INC DE
                increment_16bit_register(&mut self.d, &mut self.e);
            }
            0x14 => {
                // INC D
                increment_8bit_register(&mut self.d, &mut self.f);
            }
            0x15 => {
                // DEC D
                decrement_8bit_register(&mut self.d, &mut self.f);
            }
            0x16 => {
                // LD D, $00
                self.d = mmu.read_byte(self.pc + 1);
            }
            0x17 => {
                // RLA
                rotate_left_through_carry(&mut self.a, &mut self.f);
            }
            0x18 => {
                // JR $00
                Self::relative_jump(&mut next_pc, mmu.read_byte(self.pc + 1));
            }
            0x19 => {
                // ADD HL, DE
                add_16bit_registers(&mut self.h, &mut self.l, self.d, self.e, &mut self.f);
            }
            0x1A => {
                // LD A, (DE)
                self.a = mmu.read_byte(self.de());
            }
            0x1B => {
                // DEC DE
                decrement_16bit_register(&mut self.d, &mut self.e);
            }
            0x1C => {
                // INC E
                increment_8bit_register(&mut self.e, &mut self.f);
            }
            0x1D => {
                // DEC E
                decrement_8bit_register(&mut self.e, &mut self.f);
            }
            0x1E => {
                // LD E, $00
                self.e = mmu.read_byte(self.pc + 1);
            }
            0x1F => {
                // RRA
                rotate_right_through_carry(&mut self.a, &mut self.f)
            }
            0x20 => {
                // JR NZ $00
                if Flags::from(self.f).contains(Flags::Z) == false {
                    Self::relative_jump(&mut next_pc, mmu.read_byte(self.pc + 1));
                }
            }
            0x21 => {
                // LD HL, $0000
                self.l = mmu.read_byte(self.pc + 1);
                self.h = mmu.read_byte(self.pc + 2);
            }
            0x22 => {
                // LDI (HL), A
                mmu.write_byte(self.hl(), self.a);
                increment_16bit_register(&mut self.h, &mut self.l)
            }
            0x23 => {
                // INC HL
                increment_16bit_register(&mut self.h, &mut self.l);
            }
            0x24 => {
                // INC H
                increment_8bit_register(&mut self.h, &mut self.f);
            }
            0x25 => {
                // DEC H
                decrement_8bit_register(&mut self.h, &mut self.f);
            }
            0x26 => {
                // LD H, $00
                self.h = mmu.read_byte(self.pc + 1);
            }
            0x27 => {
                // DAA
                decimal_adjust(&mut self.a, &mut self.f);
            }
            0x28 => {
                // JR Z $00
                if Flags::from(self.f).contains(Flags::Z) {
                    Self::relative_jump(&mut next_pc, mmu.read_byte(self.pc + 1));
                }
            }
            0x29 => {
                // ADD HL, HL
                let h = self.h;
                let l = self.l;
                add_16bit_registers(&mut self.h, &mut self.l, h, l, &mut self.f);
            }
            0x2A => {
                // LDI A, (HL)
                self.a = mmu.read_byte(self.hl());
                increment_16bit_register(&mut self.h, &mut self.l);
            }
            0x2B => {
                // DEC HL
                decrement_16bit_register(&mut self.h, &mut self.l);
            }
            0x2C => {
                // INC L
                increment_8bit_register(&mut self.l, &mut self.f);
            }
            0x2D => {
                // DEC L
                decrement_8bit_register(&mut self.l, &mut self.f);
            }
            0x2E => {
                // LD L, $00
                self.l = mmu.read_byte(self.pc + 1);
            }
            0x2F => {
                // CPL
                complement(&mut self.a, &mut self.f);
            }
            0x30 => {
                // JR NC $00
                if Flags::from(self.f).contains(Flags::C) == false {
                    Self::relative_jump(&mut next_pc, mmu.read_byte(self.pc + 1));
                }
            }
            0x31 => {
                // LD SP, $0000
                self.sp = mmu.read_word(self.pc + 1);
            }
            0x32 => {
                // LDD (HL), A
                mmu.write_byte(self.hl(), self.a);
                decrement_16bit_register(&mut self.h, &mut self.l);
            }
            0x33 => {
                // INC SP
                self.sp = self.sp.wrapping_add(1);
            }
            0x34 => {
                // INC (HL)
                let addr = self.hl();
                let mut data = mmu.read_byte(addr);
                increment_8bit_register(&mut data, &mut self.f);
                mmu.write_byte(addr, data);
            }
            0x35 => {
                // DEC (HL)
                let addr = self.hl();
                let mut data = mmu.read_byte(addr);
                decrement_8bit_register(&mut data, &mut self.f);
                mmu.write_byte(addr, data);
            }
            0x36 => {
                // LD (HL), $00
                mmu.write_byte(self.hl(), mmu.read_byte(self.pc + 1));
            }
            0x37 => {
                // SCF
                self.f |= Flags::C.bits();
            }
            0x38 => {
                // JR C $00
                if Flags::from(self.f).contains(Flags::C) {
                    Self::relative_jump(&mut next_pc, mmu.read_byte(self.pc + 1));
                }
            }
            0x39 => {
                // ADD HL, SP
                let r = R16::from(self.sp);
                add_16bit_registers(&mut self.h, &mut self.l, r.h, r.l, &mut self.f);
            }
            0x3A => {
                // LDD A, (HL)
                self.a = mmu.read_byte(self.hl());
                decrement_16bit_register(&mut self.h, &mut self.l);
            }
            0x3B => {
                // DEC SP
                self.sp = self.sp.wrapping_sub(1);
            }
            0x3C => {
                // INC A
                increment_8bit_register(&mut self.a, &mut self.f);
            }
            0x3D => {
                // DEC A
                decrement_8bit_register(&mut self.a, &mut self.f);
            }
            0x3E => {
                // LD A, $00
                self.a = mmu.read_byte(self.pc + 1);
            }
            0x3F => {
                // CCF
                self.f &= !Flags::C.bits();
            }
            0x40 => {
                // LD B, B
            }
            0x41 => {
                // LD B, C
                self.b = self.c;
            }
            0x42 => {
                // LD B, D
                self.b = self.d;
            }
            0x43 => {
                // LD B, E
                self.b = self.e;
            }
            0x44 => {
                // LD B, H
                self.b = self.h;
            }
            0x45 => {
                // LD B, L
                self.b = self.l;
            }
            0x46 => {
                // LD B, (HL)
                self.b = mmu.read_byte(self.hl());
            }
            0x47 => {
                // LD B, A
                self.b = self.a;
            }
            0x48 => {
                // LD C, B
                self.c = self.b;
            }
            0x49 => {
                // LD C, C
            }
            0x4A => {
                // LD C, D
                self.c = self.d;
            }
            0x4B => {
                // LD C, E
                self.d = self.e;
            }
            0x4C => {
                // LD C, H
                self.c = self.h;
            }
            0x4D => {
                // LD C, L
                self.c = self.l;
            }
            0x4E => {
                // LD C, (HL)
                self.c = mmu.read_byte(self.hl());
            }
            0x4F => {
                // LD C, A
                self.c = self.a;
            }
            0x50 => {
                // LD D, B
                self.d = self.b;
            }
            0x51 => {
                // LD D, C
                self.d = self.c;
            }
            0x52 => {
                // LD D, D
            }
            0x53 => {
                // LD D, E
                self.d = self.e;
            }
            0x54 => {
                // LD D, H
                self.d = self.h;
            }
            0x55 => {
                // LD D, L
                self.d = self.l;
            }
            0x56 => {
                // LD D, (HL)
                self.d = mmu.read_byte(self.hl());
            }
            0x57 => {
                // LD D, A
                self.d = self.a;
            }
            0x58 => {
                // LD E, B
                self.e = self.b;
            }
            0x59 => {
                // LD E, C
                self.e = self.c;
            }
            0x5A => {
                // LD E, D
                self.e = self.d;
            }
            0x5B => {
                // LD E, E
            }
            0x5C => {
                // LD E, H
                self.e = self.h;
            }
            0x5D => {
                // LD E, L
                self.e = self.l;
            }
            0x5E => {
                // LD E, (HL)
                self.e = mmu.read_byte(self.hl());
            }
            0x5F => {
                // LD E, A
                self.e = self.a;
            }
            0x60 => {
                // LD H, B
                self.h = self.b;
            }
            0x61 => {
                // LD H, C
                self.h = self.c;
            }
            0x62 => {
                // LD H, D
                self.h = self.d;
            }
            0x63 => {
                // LD H, E
                self.h = self.e;
            }
            0x64 => {
                // LD H, H
            }
            0x65 => {
                // LD H, L
                self.h = self.l;
            }
            0x66 => {
                // LD H, (HL)
                self.h = mmu.read_byte(self.hl());
            }
            0x67 => {
                // LD H, A
                self.h = self.a;
            }
            0x68 => {
                // LD L, B
                self.l = self.b;
            }
            0x69 => {
                // LD L, C
                self.l = self.c;
            }
            0x6A => {
                // LD L, D
                self.l = self.d;
            }
            0x6B => {
                // LD L, E
                self.l = self.e;
            }
            0x6C => {
                // LD L, H
                self.l = self.h;
            }
            0x6D => {
                // LD L, L
            }
            0x6E => {
                // LD L, (HL)
                self.l = mmu.read_byte(self.hl());
            }
            0x6F => {
                // LD L, A
                self.l = self.a;
            }
            0x70 => {
                // LD (HL), B
                mmu.write_byte(self.hl(), self.b);
            }
            0x71 => {
                // LD (HL), C
                mmu.write_byte(self.hl(), self.c);
            }
            0x72 => {
                // LD (HL), D
                mmu.write_byte(self.hl(), self.d);
            }
            0x73 => {
                // LD (HL), E
                mmu.write_byte(self.hl(), self.e);
            }
            0x74 => {
                // LD (HL), H
                mmu.write_byte(self.hl(), self.h);
            }
            0x75 => {
                // LD (HL), L
                mmu.write_byte(self.hl(), self.l);
            }
            0x76 => {
                // LD (HL), (HL)
            }
            0x77 => {
                // LD (HL), A
                mmu.write_byte(self.hl(), self.a);
            }
            0x78 => {
                // LD A, B
                self.a = self.b;
            }
            0x79 => {
                // LD A, C
                self.a = self.c;
            }
            0x7A => {
                // LD A, D
                self.a = self.d;
            }
            0x7B => {
                // LD A, E
                self.a = self.e;
            }
            0x7C => {
                // LD A, H
                self.a = self.h;
            }
            0x7D => {
                // LD A, L
                self.a = self.l;
            }
            0x7E => {
                // LD A, (HL)
                self.a = mmu.read_byte(self.hl());
            }
            0x7F => {
                // LD A, A
            }
            0x80 => {
                // ADD A, B
                add_8bit_registers(&mut self.a, self.b, &mut self.f);
            }
            0x81 => {
                // ADD A, C
                add_8bit_registers(&mut self.a, self.c, &mut self.f);
            }
            0x82 => {
                // ADD A, D
                add_8bit_registers(&mut self.a, self.d, &mut self.f);
            }
            0x83 => {
                // ADD A, E
                add_8bit_registers(&mut self.a, self.e, &mut self.f);
            }
            0x84 => {
                // ADD A, H
                add_8bit_registers(&mut self.a, self.h, &mut self.f);
            }
            0x85 => {
                // ADD A, L
                add_8bit_registers(&mut self.a, self.l, &mut self.f);
            }
            0x86 => {
                // ADD A, (HL)
                let r = mmu.read_byte(self.hl());
                add_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0x87 => {
                // ADD A, A
                let r = self.a;
                add_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0x88 => {
                // ADC A, B
                add_8bit_registers_with_carry(&mut self.a, self.b, &mut self.f)
            }
            0x89 => {
                // ADC A, C
                add_8bit_registers_with_carry(&mut self.a, self.c, &mut self.f)
            }
            0x8A => {
                // ADC A, D
                add_8bit_registers_with_carry(&mut self.a, self.d, &mut self.f)
            }
            0x8B => {
                // ADC A, E
                add_8bit_registers_with_carry(&mut self.a, self.e, &mut self.f)
            }
            0x8C => {
                // ADC A, H
                add_8bit_registers_with_carry(&mut self.a, self.h, &mut self.f)
            }
            0x8D => {
                // ADC A, L
                add_8bit_registers_with_carry(&mut self.a, self.l, &mut self.f)
            }
            0x8E => {
                // ADC A, (HL)
                let r = mmu.read_byte(self.hl());
                add_8bit_registers_with_carry(&mut self.a, r, &mut self.f)
            }
            0x8F => {
                // ADC A, A
                let r = self.a;
                add_8bit_registers_with_carry(&mut self.a, r, &mut self.f)
            }
            0x90 => {
                // SUB A, B
                sub_8bit_registers(&mut self.a, self.b, &mut self.f);
            }
            0x91 => {
                // SUB A, C
                sub_8bit_registers(&mut self.a, self.c, &mut self.f);
            }
            0x92 => {
                // SUB A, D
                sub_8bit_registers(&mut self.a, self.d, &mut self.f);
            }
            0x93 => {
                // SUB A, E
                sub_8bit_registers(&mut self.a, self.e, &mut self.f);
            }
            0x94 => {
                // SUB A, H
                sub_8bit_registers(&mut self.a, self.h, &mut self.f);
            }
            0x95 => {
                // SUB A, L
                sub_8bit_registers(&mut self.a, self.l, &mut self.f);
            }
            0x96 => {
                // SUB A, (HL)
                let r = mmu.read_byte(self.hl());
                sub_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0x97 => {
                // SUB A, A
                let r = self.a;
                sub_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0x98 => {
                // SBC A, B
                sub_8bit_registers_with_carry(&mut self.a, self.b, &mut self.f);
            }
            0x99 => {
                // SBC A, C
                sub_8bit_registers_with_carry(&mut self.a, self.c, &mut self.f);
            }
            0x9A => {
                // SBC A, D
                sub_8bit_registers_with_carry(&mut self.a, self.d, &mut self.f);
            }
            0x9B => {
                // SBC A, E
                sub_8bit_registers_with_carry(&mut self.a, self.e, &mut self.f);
            }
            0x9C => {
                // SBC A, H
                sub_8bit_registers_with_carry(&mut self.a, self.h, &mut self.f);
            }
            0x9D => {
                // SBC A, L
                sub_8bit_registers_with_carry(&mut self.a, self.l, &mut self.f);
            }
            0x9E => {
                // SBC A, (HL)
                let r = mmu.read_byte(self.hl());
                sub_8bit_registers_with_carry(&mut self.a, r, &mut self.f);
            }
            0x9F => {
                // SBC A, A
                let r = self.a;
                sub_8bit_registers_with_carry(&mut self.a, r, &mut self.f);
            }
            0xA0 => {
                // AND A, B
                and_8bit_registers(&mut self.a, self.b, &mut self.f);
            }
            0xA1 => {
                // AND A, C
                and_8bit_registers(&mut self.a, self.c, &mut self.f);
            }
            0xA2 => {
                // AND A, D
                and_8bit_registers(&mut self.a, self.d, &mut self.f);
            }
            0xA3 => {
                // AND A, E
                and_8bit_registers(&mut self.a, self.e, &mut self.f);
            }
            0xA4 => {
                // AND A, H
                and_8bit_registers(&mut self.a, self.h, &mut self.f);
            }
            0xA5 => {
                // AND A, L
                and_8bit_registers(&mut self.a, self.l, &mut self.f);
            }
            0xA6 => {
                // AND A, (HL)
                let r = mmu.read_byte(self.hl());
                and_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xA7 => {
                // AND A, A
                let r = self.a;
                and_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xA8 => {
                // XOR A, B
                xor_8bit_registers(&mut self.a, self.b, &mut self.f);
            }
            0xA9 => {
                // XOR A, C
                xor_8bit_registers(&mut self.a, self.c, &mut self.f);
            }
            0xAA => {
                // XOR A, D
                xor_8bit_registers(&mut self.a, self.d, &mut self.f);
            }
            0xAB => {
                // XOR A, E
                xor_8bit_registers(&mut self.a, self.e, &mut self.f);
            }
            0xAC => {
                // XOR A, H
                xor_8bit_registers(&mut self.a, self.h, &mut self.f);
            }
            0xAD => {
                // XOR A, L
                xor_8bit_registers(&mut self.a, self.l, &mut self.f);
            }
            0xAE => {
                // XOR A, (HL)
                let r = mmu.read_byte(self.hl());
                xor_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xAF => {
                // XOR A, A
                let r = self.a;
                xor_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xB0 => {
                // OR A, B
                or_8bit_registers(&mut self.a, self.b, &mut self.f);
            }
            0xB1 => {
                // OR A, C
                or_8bit_registers(&mut self.a, self.c, &mut self.f);
            }
            0xB2 => {
                // OR A, D
                or_8bit_registers(&mut self.a, self.d, &mut self.f);
            }
            0xB3 => {
                // OR A, E
                or_8bit_registers(&mut self.a, self.e, &mut self.f);
            }
            0xB4 => {
                // OR A, H
                or_8bit_registers(&mut self.a, self.h, &mut self.f);
            }
            0xB5 => {
                // OR A, L
                or_8bit_registers(&mut self.a, self.l, &mut self.f);
            }
            0xB6 => {
                // OR A, (HL)
                let r = mmu.read_byte(self.hl());
                or_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xB7 => {
                // OR A, A
                let r = self.a;
                or_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xB8 => {
                // CP A, B
                compare_8bit_registers(self.a, self.b, &mut self.f);
            }
            0xB9 => {
                // CP A, C
                compare_8bit_registers(self.a, self.c, &mut self.f);
            }
            0xBA => {
                // CP A, D
                compare_8bit_registers(self.a, self.d, &mut self.f);
            }
            0xBB => {
                // CP A, E
                compare_8bit_registers(self.a, self.e, &mut self.f);
            }
            0xBC => {
                // CP A, H
                compare_8bit_registers(self.a, self.h, &mut self.f);
            }
            0xBD => {
                // CP A, L
                compare_8bit_registers(self.a, self.l, &mut self.f);
            }
            0xBE => {
                // CP A, (HL)
                let r = mmu.read_byte(self.hl());
                compare_8bit_registers(self.a, r, &mut self.f);
            }
            0xBF => {
                // CP A, A
                compare_8bit_registers(self.a, self.a, &mut self.f);
            }
            0xC0 => {
                // RET NZ
                if Flags::from(self.f).contains(Flags::Z) == false {
                    self.return_call(&mut next_pc, mmu);
                }
            }
            0xC1 => {
                // POP BC
                let mut rh = self.b;
                let mut rl = self.c;
                self.stack_pop(&mut rh, &mut rl, mmu);
                self.b = rh;
                self.c = rl;
            }
            0xC2 => {
                // JP NZ $0000
                if Flags::from(self.f).contains(Flags::Z) == false {
                    next_pc = mmu.read_word(self.pc + 1);
                }
            }
            0xC3 => {
                // JP $0000
                next_pc = mmu.read_word(self.pc + 1);
            }
            0xC4 => {
                // CALL NZ $0000
                if Flags::from(self.f).contains(Flags::Z) == false {
                    self.process_call(&mut next_pc, mmu)
                }
            }
            0xC5 => {
                // PUSH BC
                self.stack_push(self.b, self.c, mmu);
            }
            0xC6 => {
                // ADD A, $00
                let r = mmu.read_byte(self.pc + 1);
                add_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xC7 => {
                // RST $00
                self.restart(&mut next_pc, 0x00, mmu);
            }
            0xC8 => {
                // RET Z
                if Flags::from(self.f).contains(Flags::Z) {
                    self.return_call(&mut next_pc, mmu);
                }
            }
            0xC9 => {
                // RET
                self.return_call(&mut next_pc, mmu);
            }
            0xCA => {
                // JP Z $0000
                if Flags::from(self.f).contains(Flags::Z) {
                    next_pc = mmu.read_word(self.pc + 1);
                }
            }
            0xCB => {
                // PREFIX CB
            }
            0xCC => {
                // CALL Z $0000
                if Flags::from(self.f).contains(Flags::Z) {
                    self.process_call(&mut next_pc, mmu)
                }
            }
            0xCD => {
                // CALL $0000
                self.process_call(&mut next_pc, mmu)
            }
            0xCE => {
                // ADC A, $$00
                let r = mmu.read_byte(self.pc + 1);
                add_8bit_registers_with_carry(&mut self.a, r, &mut self.f);
            }
            0xCF => {
                // RST $08
                self.restart(&mut next_pc, 0x08, mmu);
            }
            0xD0 => {
                // RET NC
                if Flags::from(self.f).contains(Flags::C) == false {
                    self.return_call(&mut next_pc, mmu);
                }
            }
            0xD1 => {
                // POP DE
                let mut rh = self.d;
                let mut rl = self.e;
                self.stack_pop(&mut rh, &mut rl, mmu);
                self.d = rh;
                self.e = rl;
            }
            0xD2 => {
                // JP NC $0000
                if Flags::from(self.f).contains(Flags::C) == false {
                    next_pc = mmu.read_word(self.pc + 1);
                }
            }
            0xD3 => {
                // [D3] - INVALID
            }
            0xD4 => {
                // CALL NC $0000
                if Flags::from(self.f).contains(Flags::C) == false {
                    self.process_call(&mut next_pc, mmu)
                }
            }
            0xD5 => {
                // PUSH DE
                self.stack_push(self.d, self.e, mmu);
            }
            0xD6 => {
                // SUB A, $$00
                let r = mmu.read_byte(self.pc + 1);
                sub_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xD7 => {
                // RST $10
                self.restart(&mut next_pc, 0x10, mmu);
            }
            0xD8 => {
                // RET C
                if Flags::from(self.f).contains(Flags::C) {
                    self.return_call(&mut next_pc, mmu);
                }
            }
            0xD9 => {
                // RETI
                self.return_call(&mut next_pc, mmu);
                self.ie_flag = true;
            }
            0xDA => {
                // JP C $0000
                if Flags::from(self.f).contains(Flags::C) {
                    next_pc = mmu.read_word(self.pc + 1);
                }
            }
            0xDB => {
                // [DB] - INVALID
            }
            0xDC => {
                // CALL C $0000
                if Flags::from(self.f).contains(Flags::C) {
                    self.process_call(&mut next_pc, mmu)
                }
            }
            0xDD => {
                // [DD] - INVALID
            }
            0xDE => {
                // SBC A, $00
                let r = mmu.read_byte(self.pc + 1);
                sub_8bit_registers_with_carry(&mut self.a, r, &mut self.f);
            }
            0xDF => {
                // RST $18
                self.restart(&mut next_pc, 0x0018, mmu);
            }
            0xE0 => {
                // LDH ($00), A
                let addr: u16 = 0xff00 + mmu.read_byte(self.pc + 1) as u16;
                mmu.write_byte(addr, self.a);
            }
            0xE1 => {
                // POP HL
                let mut rh = self.h;
                let mut rl = self.l;
                self.stack_pop(&mut rh, &mut rl, mmu);
                self.h = rh;
                self.l = rl;
            }
            0xE2 => {
                // LDH (C), A
            }
            0xE3 => {
                // [E3] - INVALID
            }
            0xE4 => {
                // [E4] - INVALID
            }
            0xE5 => {
                // PUSH HL
                self.stack_push(self.h, self.l, mmu);
            }
            0xE6 => {
                // AND $00
                let r = mmu.read_byte(self.pc + 1);
                and_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xE7 => {
                // RST $20
                self.restart(&mut next_pc, 0x0020, mmu);
            }
            0xE8 => {
                // ADD SP, $00
                let mut r1 = R16::from(self.sp);
                let r2 = R16::from(mmu.read_byte(self.pc + 1) as u16);
                add_16bit_registers(&mut r1.l, &mut r1.h, r2.h, r2.l, &mut self.f);
            }
            0xE9 => {
                // JP HL
                next_pc = self.hl();
            }
            0xEA => {
                // LD ($0000), A
                let addr = mmu.read_word(self.pc + 1);
                mmu.write_byte(addr, self.a);
            }
            0xEB => {
                // [EB] - INVALID
            }
            0xEC => {
                // [EC] - INVALID
            }
            0xED => {
                // [ED] - INVALID
            }
            0xEE => {
                // XOR $00
                let r = mmu.read_byte(self.pc + 1);
                xor_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xEF => {
                // RST $28
                self.restart(&mut next_pc, 0x0028, mmu);
            }
            0xF0 => {
                // LDH A, ($00)
                let addr: u16 = 0xff00 + mmu.read_byte(self.pc + 1) as u16;
                self.a = mmu.read_byte(addr);
            }
            0xF1 => {
                // POP AF
                let mut rh = self.a;
                let mut rl = self.f;
                self.stack_pop(&mut rh, &mut rl, mmu);
                self.a = rh;
                self.f = rl;
            }
            0xF2 => {
                // LD A, ($FF00+C)
                self.a = mmu.read_byte(0xff00u16 | self.c as u16);
            }
            0xF3 => {
                // DI
                self.ie_flag = false;
            }
            0xF4 => {
                // [F4] - INVALID
            }
            0xF5 => {
                // PUSH AF
                self.stack_push(self.a, self.f, mmu);
            }
            0xF6 => {
                // OR $00
                let r = mmu.read_byte(self.pc + 1);
                or_8bit_registers(&mut self.a, r, &mut self.f);
            }
            0xF7 => {
                // RST $30
                self.restart(&mut next_pc, 0x30, mmu);
            }
            0xF8 => {
                // LD HL,SP+$00
                let offset = mmu.read_byte(self.pc + 1) as i8;

                let r = if offset >= 0 {
                    R16::from(self.sp.wrapping_add(offset as u16))
                } else {
                    R16::from(self.sp.wrapping_sub((-offset) as u16))
                };

                self.h = r.h;
                self.l = r.l;
            }
            0xF9 => {
                // LD SP, HL
                self.sp = self.hl();
            }
            0xFA => {
                // LD A, ($0000)
                self.a = mmu.read_byte(mmu.read_word(self.pc + 1));
            }
            0xFB => {
                // EI
                self.ie_flag = true;
            }
            0xFC => {
                // [FC] - INVALID;
            }
            0xFD => {
                // [FD] - INVALID;
            }
            0xFE => {
                // CP $00
                let r = mmu.read_byte(self.pc + 1);
                compare_8bit_registers(self.a, r, &mut self.f);
            }
            0xFF => {
                // RST $38
                self.restart(&mut next_pc, 0x38, mmu);
            }
        }
        self.pc = next_pc;
        INSTRUCTION_TICKS[opcode as usize].into()
    }

    fn stack_push(&mut self, h: u8, l: u8, mmu: &mut dyn Mmu) {
        mmu.write_byte(self.sp, h);
        mmu.write_byte(self.sp - 1, l);
        self.sp = self.sp - 2;
    }

    fn stack_pop(&mut self, h: &mut u8, l: &mut u8, mmu: &mut dyn Mmu) {
        *l = mmu.read_byte(self.sp + 1);
        *h = mmu.read_byte(self.sp + 2);
        self.sp = self.sp + 2;
    }

    fn relative_jump(pc: &mut u16, param: u8) {
        let offset = param as i8;
        if offset >= 0 {
            *pc = pc.wrapping_add(offset as u16);
        } else {
            *pc = pc.wrapping_sub((-offset) as u16);
        }
    }

    fn restart(&mut self, pc: &mut u16, rst_addr: u16, mmu: &mut dyn Mmu) {
        let ret_addr = R16::from(*pc);
        self.stack_push(ret_addr.h, ret_addr.l, mmu);
        *pc = rst_addr;
    }

    fn process_call(&mut self, pc: &mut u16, mmu: &mut dyn Mmu) {
        let ret_addr = R16::from(*pc);
        self.stack_push(ret_addr.h, ret_addr.l, mmu);
        *pc = mmu.read_word(self.pc +1);
    }

    fn return_call(&mut self, pc: &mut u16, mmu: &mut dyn Mmu) {
        let mut ret_addr = R16::zero();
        self.stack_pop(&mut ret_addr.h, &mut ret_addr.l, mmu);
        *pc = ret_addr.into();
    }
}

