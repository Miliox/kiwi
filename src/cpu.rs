pub mod asm;
pub mod alu;
pub mod flags;
pub mod regs;

use std::rc::Rc;
use std::cell::RefCell;

use crate::cpu::asm::disassemble;
use crate::cpu::asm::instruction_size;
use crate::cpu::asm::instruction_ticks;
use crate::cpu::alu::Alu;
use crate::cpu::regs::Regs;

use crate::mmu::Mmu;
use crate::mmu::Memory;

#[allow(dead_code)]
#[derive(Default)]
pub struct Cpu {
    alu: Alu,    // Arithmetic Logic Unit
    regs: Regs,    // Registers
    next_pc: u16,  // next program counter position

    clock: u64,       // accumulated clock counter
    int_enable: bool, // interrupt enable flag
    int_flags:  u8,   // interrupt flags flag

    pub mmu: Option<Rc<RefCell<Mmu>>>, // Reference to Memory Management Unit
}

#[allow(dead_code)]
impl Cpu {
    #[inline(always)]
    pub fn carry(&self) -> bool {
        self.alu.flags.carry()
    }

    #[inline(always)]
    pub fn half(&self) -> bool {
        self.alu.flags.half()
    }

    #[inline(always)]
    pub fn sub(&self) -> bool {
        self.alu.flags.sub()
    }

    #[inline(always)]
    pub fn zero(&self) -> bool {
        self.alu.flags.zero()
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.mmu.as_ref().unwrap().borrow().read_byte(addr)
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        self.mmu.as_ref().unwrap().borrow_mut().write_byte(addr, data);
    }

    pub fn cycle(&mut self) -> u64 {
        let pc = self.regs.pc();
        let opcode = self.read_byte(pc);
        let immediate8: u8 = self.read_byte(pc + 1);
        let immediate16: u16 = u16::from_le_bytes([immediate8, self.read_byte(pc + 2)]);

        println!("{:<15}; ${:04x}; {:02x?}", disassemble(opcode, immediate8, immediate16), pc, self.regs);

        self.next_pc = pc + instruction_size(opcode);
        self.execute(opcode, immediate8, immediate16);
        self.regs.set_pc(self.next_pc);
        instruction_ticks(opcode)
    }

    fn execute(&mut self, opcode: u8, immediate8: u8, immediate16: u16) {
        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC, $0000
                self.regs.set_bc(immediate16);
            }
            0x02 => {
                // LD (BC), A
                self.write_byte(self.regs.bc(), self.regs.a());
            },
            0x03 => {
                // INC BC
                self.regs.set_bc(self.alu.inc16(self.regs.bc()));
            }
            0x04 => {
                // INC B
                self.regs.set_b(self.alu.inc(self.regs.b()));
            }
            0x05 => {
                // DEC B
                self.regs.set_b(self.alu.dec(self.regs.b()));
            }
            0x06 => {
                // LD B, $00
                self.regs.set_b(immediate8);
            }
            0x07 => {
                // RLCA
                self.regs.set_a(self.alu.rlc(self.regs.a()));
            }
            0x08 => {
                // LD ($0000),SP
                let le_bytes = self.regs.sp().to_le_bytes();
                self.write_byte(immediate16, le_bytes[0]);
                self.write_byte(immediate16 + 1, le_bytes[1]);
            }
            0x09 => {
                // ADD HL, BC
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.bc()));
            }
            0x0A => {
                // LD A, (BC)
                let data = self.read_bc_ref();
                self.regs.set_a(data);
            }
            0x0B => {
                // DEC BC
                self.regs.set_bc(self.alu.dec16(self.regs.bc()));
            }
            0x0C => {
                // INC C
                self.regs.set_c(self.alu.inc(self.regs.c()));
            }
            0x0D => {
                // DEC C
                self.regs.set_c(self.alu.dec(self.regs.c()));
            }
            0x0E => {
                // LD C, $00
                self.regs.set_c(immediate8);
            }
            0x0F => {
                // RRCA
                self.regs.set_a(self.alu.rrc(self.regs.a()));
            }
            0x10 => {
                // STOP 0
            }
            0x11 => {
                // LD DE, $0000
                self.regs.set_de(immediate16);
            }
            0x12 => {
                // LD (DE), A
                self.write_byte(self.regs.de(), self.regs.a());
            }
            0x13 => {
                // INC DE
                self.regs.set_de(self.alu.inc16(self.regs.de()));
            }
            0x14 => {
                // INC D
                self.regs.set_d(self.alu.inc(self.regs.d()));
            }
            0x15 => {
                // DEC D
                self.regs.set_d(self.alu.dec(self.regs.d()));
            }
            0x16 => {
                // LD D, $00
                self.regs.set_d(immediate8);
            }
            0x17 => {
                // RLA
                self.regs.set_a(self.alu.rl(self.regs.a()));
            }
            0x18 => {
                // JR $00
                self.relative_jump(immediate8);
            }
            0x19 => {
                // ADD HL, DE
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.de()));
            }
            0x1A => {
                // LD A, (DE)
                self.regs.set_a(self.read_de_ref());
            }
            0x1B => {
                // DEC DE
                self.regs.set_de(self.alu.dec16(self.regs.de()));
            }
            0x1C => {
                // INC E
                self.regs.set_e(self.alu.inc(self.regs.e()));
            }
            0x1D => {
                // DEC E
                self.regs.set_e(self.alu.dec(self.regs.e()));
            }
            0x1E => {
                // LD E, $00
                self.regs.set_e(immediate8);
            }
            0x1F => {
                // RRA
                self.regs.set_a(self.alu.rr(self.regs.a()));
            }
            0x20 => {
                // JR NZ $00
                if !self.zero() {
                    self.relative_jump(immediate8);
                }
            }
            0x21 => {
                // LD HL, $0000
                self.regs.set_hl(immediate16);
            }
            0x22 => {
                // LDI (HL), A
                self.write_hl_ref_and_increment_hl(self.regs.a());
            }
            0x23 => {
                // INC HL
                self.regs.set_hl(self.alu.inc16(self.regs.hl()));
            }
            0x24 => {
                // INC H
                self.regs.set_h(self.alu.inc(self.regs.h()));
            }
            0x25 => {
                // DEC H
                self.regs.set_h(self.alu.dec(self.regs.h()));
            }
            0x26 => {
                // LD H, $00
                self.regs.set_h(immediate8);
            }
            0x27 => {
                // DAA
                self.regs.set_a(self.alu.daa(self.regs.a()));
            }
            0x28 => {
                // JR Z $00
                if self.zero() {
                    self.relative_jump(immediate8);
                }
            }
            0x29 => {
                // ADD HL, HL
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.hl()));
            }
            0x2A => {
                // LDI A, (HL)
                let data = self.read_hl_ref_and_increment_hl();
                self.regs.set_a(data);
            }
            0x2B => {
                // DEC HL
                self.regs.set_hl(self.alu.dec16(self.regs.hl()));
            }
            0x2C => {
                // INC L
                self.regs.set_l(self.alu.inc(self.regs.l()));
            }
            0x2D => {
                // DEC L
                self.regs.set_l(self.alu.dec(self.regs.l()));
            }
            0x2E => {
                // LD L, $00
                self.regs.set_l(immediate8);
            }
            0x2F => {
                // CPL
                self.regs.set_a(self.alu.complement(self.regs.a()))
            }
            0x30 => {
                // JR NC $00
                if !self.carry() {
                    self.relative_jump(immediate8);
                }
            }
            0x31 => {
                // LD SP, $0000
                self.regs.set_sp(immediate16);
            }
            0x32 => {
                // LDD (HL), A
                self.write_hl_ref_and_decrement_hl(self.regs.a());
            }
            0x33 => {
                // INC SP
                self.regs.set_sp(self.regs.sp().wrapping_add(1));
            }
            0x34 => {
                // INC (HL)
                let addr = self.regs.hl();
                let data = self.alu.inc(self.read_byte(addr));
                self.write_byte(addr, data);
            }
            0x35 => {
                // DEC (HL)
                let addr = self.regs.hl();
                let data = self.alu.dec(self.read_byte(addr));
                self.write_byte(addr, data);
            }
            0x36 => {
                // LD (HL), $00
                self.write_byte(self.regs.hl(), immediate8);
            }
            0x37 => {
                // SCF
                self.alu.flags.set_carry();
            }
            0x38 => {
                // JR C $00
                if self.carry() {
                    self.relative_jump(immediate8);
                }
            }
            0x39 => {
                // ADD HL, SP
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.sp()));
            }
            0x3A => {
                // LDD A, (HL)
                let data = self.read_hl_ref_and_decrement_hl();
                self.regs.set_a(data);
            }
            0x3B => {
                // DEC SP
                self.regs.set_sp(self.regs.sp().wrapping_sub(1));
            }
            0x3C => {
                // INC A
                self.regs.set_a(self.alu.inc(self.regs.a()));
            }
            0x3D => {
                // DEC A
                self.regs.set_a(self.alu.dec(self.regs.a()));
            }
            0x3E => {
                // LD A, $00
                self.regs.set_a(immediate8);
            }
            0x3F => {
                // CCF
                self.alu.flags.reset_carry();
            }
            0x40 => {
                // LD B, B
            }
            0x41 => {
                // LD B, C
                self.regs.set_b(self.regs.c());
            }
            0x42 => {
                // LD B, D
                self.regs.set_b(self.regs.d());
            }
            0x43 => {
                // LD B, E
                self.regs.set_b(self.regs.e());
            }
            0x44 => {
                // LD B, H
                self.regs.set_b(self.regs.h());
            }
            0x45 => {
                // LD B, L
                self.regs.set_b(self.regs.l());
            }
            0x46 => {
                // LD B, (HL)
                self.regs.set_b(self.read_hl_ref());
            }
            0x47 => {
                // LD B, A
                self.regs.set_b(self.regs.a());
            }
            0x48 => {
                // LD C, B
                self.regs.set_c(self.regs.b());
            }
            0x49 => {
                // LD C, C
            }
            0x4A => {
                // LD C, D
                self.regs.set_c(self.regs.d());
            }
            0x4B => {
                // LD C, E
                self.regs.set_c(self.regs.e());
            }
            0x4C => {
                // LD C, H
                self.regs.set_c(self.regs.h());
            }
            0x4D => {
                // LD C, L
                self.regs.set_c(self.regs.l());
            }
            0x4E => {
                // LD C, (HL)
                self.regs.set_c(self.read_hl_ref());
            }
            0x4F => {
                // LD C, A
                self.regs.set_c(self.regs.a());
            }
            0x50 => {
                // LD D, B
                self.regs.set_d(self.regs.b());
            }
            0x51 => {
                // LD D, C
                self.regs.set_d(self.regs.c());
            }
            0x52 => {
                // LD D, D
            }
            0x53 => {
                // LD D, E
                self.regs.set_d(self.regs.e());
            }
            0x54 => {
                // LD D, H
                self.regs.set_d(self.regs.h());
            }
            0x55 => {
                // LD D, L
                self.regs.set_d(self.regs.l());
            }
            0x56 => {
                // LD D, (HL)
                self.regs.set_d(self.read_hl_ref());
            }
            0x57 => {
                // LD D, A
                self.regs.set_d(self.regs.a());
            }
            0x58 => {
                // LD E, B
                self.regs.set_e(self.regs.b());
            }
            0x59 => {
                // LD E, C
                self.regs.set_e(self.regs.c());
            }
            0x5A => {
                // LD E, D
                self.regs.set_e(self.regs.d());
            }
            0x5B => {
                // LD E, E
            }
            0x5C => {
                // LD E, H
                self.regs.set_e(self.regs.h());
            }
            0x5D => {
                // LD E, L
                self.regs.set_e(self.regs.l());
            }
            0x5E => {
                // LD E, (HL)
                self.regs.set_e(self.read_hl_ref());
            }
            0x5F => {
                // LD E, A
                self.regs.set_e(self.regs.a());
            }
            0x60 => {
                // LD H, B
                self.regs.set_h(self.regs.b());
            }
            0x61 => {
                // LD H, C
                self.regs.set_h(self.regs.c());
            }
            0x62 => {
                // LD H, D
                self.regs.set_h(self.regs.d());
            }
            0x63 => {
                // LD H, E
                self.regs.set_h(self.regs.e());
            }
            0x64 => {
                // LD H, H
            }
            0x65 => {
                // LD H, L
                self.regs.set_h(self.regs.l());
            }
            0x66 => {
                // LD H, (HL)
                self.regs.set_h(self.read_hl_ref());
            }
            0x67 => {
                // LD H, A
                self.regs.set_h(self.regs.a());
            }
            0x68 => {
                // LD L, B
                self.regs.set_l(self.regs.b());
            }
            0x69 => {
                // LD L, C
                self.regs.set_l(self.regs.c());
            }
            0x6A => {
                // LD L, D
                self.regs.set_l(self.regs.d());
            }
            0x6B => {
                // LD L, E
                self.regs.set_l(self.regs.e());
            }
            0x6C => {
                // LD L, H
                self.regs.set_l(self.regs.h());
            }
            0x6D => {
                // LD L, L
            }
            0x6E => {
                // LD L, (HL)
                self.regs.set_l(self.read_hl_ref());
            }
            0x6F => {
                // LD L, A
                self.regs.set_l(self.regs.a());
            }
            0x70 => {
                // LD (HL), B
                self.write_byte(self.regs.hl(), self.regs.b());
            }
            0x71 => {
                // LD (HL), C
                self.write_byte(self.regs.hl(), self.regs.c());
            }
            0x72 => {
                // LD (HL), D
                self.write_byte(self.regs.hl(), self.regs.d());
            }
            0x73 => {
                // LD (HL), E
                self.write_byte(self.regs.hl(), self.regs.e());
            }
            0x74 => {
                // LD (HL), H
                self.write_byte(self.regs.hl(), self.regs.h());
            }
            0x75 => {
                // LD (HL), L
                self.write_byte(self.regs.hl(), self.regs.l());
            }
            0x76 => {
                // LD (HL), (HL)
            }
            0x77 => {
                // LD (HL), A
                self.write_byte(self.regs.hl(), self.regs.a());
            }
            0x78 => {
                // LD A, B
                self.regs.set_a(self.regs.b());
            }
            0x79 => {
                // LD A, C
                self.regs.set_a(self.regs.c());
            }
            0x7A => {
                // LD A, D
                self.regs.set_a(self.regs.d());
            }
            0x7B => {
                // LD A, E
                self.regs.set_a(self.regs.e());
            }
            0x7C => {
                // LD A, H
                self.regs.set_a(self.regs.h());
            }
            0x7D => {
                // LD A, L
                self.regs.set_a(self.regs.l());
            }
            0x7E => {
                // LD A, (HL)
                self.regs.set_a(self.read_hl_ref());
            }
            0x7F => {
                // LD A, A
            }
            0x80 => {
                // ADD A, B
                self.regs.set_a(self.regs.b());
            }
            0x81 => {
                // ADD A, C
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.c()));
            }
            0x82 => {
                // ADD A, D
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.d()));
            }
            0x83 => {
                // ADD A, E
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.e()));
            }
            0x84 => {
                // ADD A, H
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.h()));
            }
            0x85 => {
                // ADD A, L
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.l()));
            }
            0x86 => {
                // ADD A, (HL)
                self.regs.set_a(self.alu.add(self.regs.a(), self.read_hl_ref()));
            }
            0x87 => {
                // ADD A, A
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.a()));
            }
            0x88 => {
                // ADC A, B
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.b()));
            }
            0x89 => {
                // ADC A, C
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.c()));
            }
            0x8A => {
                // ADC A, D
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.d()));
            }
            0x8B => {
                // ADC A, E
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.e()));
            }
            0x8C => {
                // ADC A, H
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.h()));
            }
            0x8D => {
                // ADC A, L
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.l()));
            }
            0x8E => {
                // ADC A, (HL)
                self.regs.set_a(self.alu.adc(self.regs.a(), self.read_hl_ref()));
            }
            0x8F => {
                // ADC A, A
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.a()));
            }
            0x90 => {
                // SUB A, B
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.b()));
            }
            0x91 => {
                // SUB A, C
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.c()));
            }
            0x92 => {
                // SUB A, D
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.d()));
            }
            0x93 => {
                // SUB A, E
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.e()));
            }
            0x94 => {
                // SUB A, H
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.h()));
            }
            0x95 => {
                // SUB A, L
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.l()));
            }
            0x96 => {
                // SUB A, (HL)
                self.regs.set_a(self.alu.sub(self.regs.a(), self.read_hl_ref()));
            }
            0x97 => {
                // SUB A, A
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.a()));
            }
            0x98 => {
                // SBC A, B
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.b()));
            }
            0x99 => {
                // SBC A, C
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.c()));
            }
            0x9A => {
                // SBC A, D
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.d()));
            }
            0x9B => {
                // SBC A, E
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.e()));
            }
            0x9C => {
                // SBC A, H
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.h()));
            }
            0x9D => {
                // SBC A, L
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.l()));
            }
            0x9E => {
                // SBC A, (HL)
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.read_hl_ref()));
            }
            0x9F => {
                // SBC A, A
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.a()));
            }
            0xA0 => {
                // AND A, B
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.b()));
            }
            0xA1 => {
                // AND A, C
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.c()));
            }
            0xA2 => {
                // AND A, D
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.d()));
            }
            0xA3 => {
                // AND A, E
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.e()));
            }
            0xA4 => {
                // AND A, H
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.h()));
            }
            0xA5 => {
                // AND A, L
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.l()));
            }
            0xA6 => {
                // AND A, (HL)
                self.regs.set_a(self.alu.and(self.regs.a(), self.read_hl_ref()));
            }
            0xA7 => {
                // AND A, A
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.a()));
            }
            0xA8 => {
                // XOR A, B
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.b()));
            }
            0xA9 => {
                // XOR A, C
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.c()));
            }
            0xAA => {
                // XOR A, D
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.d()));
            }
            0xAB => {
                // XOR A, E
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.e()));
            }
            0xAC => {
                // XOR A, H
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.h()));
            }
            0xAD => {
                // XOR A, L
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.l()));
            }
            0xAE => {
                // XOR A, (HL)
                self.regs.set_a(self.alu.xor(self.regs.a(), self.read_hl_ref()));
            }
            0xAF => {
                // XOR A, A
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.a()));
            }
            0xB0 => {
                // OR A, B
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.b()));
            }
            0xB1 => {
                // OR A, C
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.c()));
            }
            0xB2 => {
                // OR A, D
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.d()));
            }
            0xB3 => {
                // OR A, E
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.e()));
            }
            0xB4 => {
                // OR A, H
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.h()));
            }
            0xB5 => {
                // OR A, L
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.l()));
            }
            0xB6 => {
                // OR A, (HL)
                self.regs.set_a(self.alu.or(self.regs.a(), self.read_hl_ref()));
            }
            0xB7 => {
                // OR A, A
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.a()));
            }
            0xB8 => {
                // CP A, B
                self.alu.compare(self.regs.a(), self.regs.b());
            }
            0xB9 => {
                // CP A, C
                self.alu.compare(self.regs.a(), self.regs.c());
            }
            0xBA => {
                // CP A, D
                self.alu.compare(self.regs.a(), self.regs.d());
            }
            0xBB => {
                // CP A, E
                self.alu.compare(self.regs.a(), self.regs.e());
            }
            0xBC => {
                // CP A, H
                self.alu.compare(self.regs.a(), self.regs.h());
            }
            0xBD => {
                // CP A, L
                self.alu.compare(self.regs.a(), self.regs.l());
            }
            0xBE => {
                // CP A, (HL)
                self.alu.compare(self.regs.a(), self.read_hl_ref());
            }
            0xBF => {
                // CP A, A
                self.alu.compare(self.regs.a(), self.regs.a());
            }
            0xC0 => {
                // RET NZ
                if !self.zero() {
                    self.end_call();
                }
            }
            0xC1 => {
                // POP BC
                let bc = self.pop();
                self.regs.set_bc(bc);
            }
            0xC2 => {
                // JP NZ $0000
                if !self.zero() {
                    self.absolute_jump(immediate16);
                }
            }
            0xC3 => {
                // JP $0000
                self.absolute_jump(immediate16);
            }
            0xC4 => {
                // CALL NZ $0000
                if !self.zero() {
                    self.begin_call(immediate16)
                }
            }
            0xC5 => {
                // PUSH BC
                self.push(self.regs.bc());
            }
            0xC6 => {
                // ADD A, $00
                self.regs.set_a(self.alu.add(self.regs.a(), immediate8));
            }
            0xC7 => {
                // RST $00
                self.begin_call(0x00);
            }
            0xC8 => {
                // RET Z
                if self.zero() {
                    self.end_call();
                }
            }
            0xC9 => {
                // RET
                self.end_call();
            }
            0xCA => {
                // JP Z $0000
                if self.zero() {
                    self.absolute_jump(immediate16);
                }
            }
            0xCB => {
                // PREFIX CB
                self.execute_cb_ext(immediate8)
            }
            0xCC => {
                // CALL Z $0000
                if self.zero() {
                    self.begin_call(immediate16)
                }
            }
            0xCD => {
                // CALL $0000
                self.begin_call(immediate16)
            }
            0xCE => {
                // ADC A, $00
                self.regs.set_a(self.alu.adc(self.regs.a(), immediate8));
            }
            0xCF => {
                // RST $08
                self.begin_call(0x08);
            }
            0xD0 => {
                // RET NC
                if !self.carry() {
                    self.end_call();
                }
            }
            0xD1 => {
                // POP DE
                let de = self.pop();
                self.regs.set_de(de);
            }
            0xD2 => {
                // JP NC $0000
                if !self.carry() {
                    self.absolute_jump(immediate16);
                }
            }
            0xD3 => {
                // [D3] - INVALID
            }
            0xD4 => {
                // CALL NC $0000
                if !self.carry() {
                    self.begin_call(immediate16)
                }
            }
            0xD5 => {
                // PUSH DE
                self.push(self.regs.de());
            }
            0xD6 => {
                // SUB A, $00
                self.regs.set_a(self.alu.sub(self.regs.a(), immediate8));
            }
            0xD7 => {
                // RST $10
                self.begin_call(0x10);
            }
            0xD8 => {
                // RET C
                if self.carry() {
                    self.end_call();
                }
            }
            0xD9 => {
                // RETI
                self.end_call();
                self.int_enable = true;
            }
            0xDA => {
                // JP C $0000
                if self.carry() {
                    self.absolute_jump(immediate16);
                }
            }
            0xDB => {
                // [DB] - INVALID
            }
            0xDC => {
                // CALL C $0000
                if self.carry() {
                    self.begin_call(immediate16)
                }
            }
            0xDD => {
                // [DD] - INVALID
            }
            0xDE => {
                // SBC A, $00
                self.regs.set_a(self.alu.sbc(self.regs.a(), immediate8));
            }
            0xDF => {
                // RST $18
                self.begin_call(0x18);
            }
            0xE0 => {
                // LDH ($00), A
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data = self.regs.a();
                self.write_byte(addr, data);
            }
            0xE1 => {
                // POP HL
                let hl = self.pop();
                self.regs.set_hl(hl);
            }
            0xE2 => {
                // LDH (C), A
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = self.regs.a();
                self.write_byte(addr, data);
            }
            0xE3 => {
                // [E3] - INVALID
            }
            0xE4 => {
                // [E4] - INVALID
            }
            0xE5 => {
                // PUSH HL
                self.push(self.regs.hl());
            }
            0xE6 => {
                // AND $00
                self.regs.set_a(self.alu.and(self.regs.a(), immediate8));
            }
            0xE7 => {
                // RST $20
                self.begin_call(0x20);
            }
            0xE8 => {
                // ADD SP, $00
                self.regs.set_sp(self.alu.add16(self.regs.sp(), ((immediate8 as i8) as i16) as u16))
            }
            0xE9 => {
                // JP HL
                self.absolute_jump(self.regs.hl());
            }
            0xEA => {
                // LD ($0000), A
                self.write_byte(immediate16, self.regs.a());
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
                self.regs.set_a(self.alu.xor(self.regs.a(), immediate8));
            }
            0xEF => {
                // RST $28
                self.begin_call(0x28);
            }
            0xF0 => {
                // LDH A, ($00)
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data = self.read_byte(addr);
                self.regs.set_a(data);
            }
            0xF1 => {
                // POP AF
                let af = self.pop();
                self.regs.set_af(af);
                self.alu.flags = self.regs.f().into();
            }
            0xF2 => {
                // LD A, ($FF00+C)
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = self.read_byte(addr);
                self.regs.set_a(data);
            }
            0xF3 => {
                // DI
                self.int_enable = false;
            }
            0xF4 => {
                // [F4] - INVALID
            }
            0xF5 => {
                // PUSH AF
                self.regs.set_f(self.alu.flags.into());
                self.push(self.regs.af());
            }
            0xF6 => {
                // OR $00
                self.regs.set_a(self.alu.or(self.regs.a(), immediate8));
            }
            0xF7 => {
                // RST $30
                self.begin_call(0x30);
            }
            0xF8 => {
                // LD HL,SP+$00
                self.regs.set_hl(self.regs.sp().wrapping_add((immediate8 as i8) as u16));
            }
            0xF9 => {
                // LD SP, HL
                self.regs.set_sp(self.regs.hl());
            }
            0xFA => {
                // LD A, ($0000)
                let data = self.read_byte(immediate16);
                self.regs.set_a(data);
            }
            0xFB => {
                // EI
                self.int_enable = true;
            }
            0xFC => {
                // [FC] - INVALID;
            }
            0xFD => {
                // [FD] - INVALID;
            }
            0xFE => {
                // CP $00
                self.alu.compare(self.regs.a(), immediate8);
            }
            0xFF => {
                // RST $38
                self.begin_call(0x38);
            }
        }
    }

    fn execute_cb_ext(&mut self, cb_code: u8) {
        let arg = match cb_code & 0x7 {
            0x0 => self.regs.b(),
            0x1 => self.regs.c(),
            0x2 => self.regs.d(),
            0x3 => self.regs.e(),
            0x4 => self.regs.h(),
            0x5 => self.regs.l(),
            0x6 => self.read_hl_ref(),
            0x7 => self.regs.a(),
            _ => panic!()
        };

        let mut ret = arg;

        match cb_code {
            0x00..=0x07 => {
                // RLC
                ret = self.alu.rlc(arg);
            }
            0x08..=0x0F => {
                // RRC
                ret = self.alu.rrc(arg);
            }
            0x10..=0x17 => {
                // RL
                ret = self.alu.rl(arg);
            }
            0x18..=0x1F => {
                // RR
                ret = self.alu.rr(arg);
            }
            0x20..=0x27 => {
                // SLA
                ret = self.alu.sla(arg);
            }
            0x28..=0x2F => {
                // SRA
                ret = self.alu.sra(arg);
            }
            0x30..=0x37 => {
                // SWAP
                ret = self.alu.nibble_swap(arg);
            }
            0x38..=0x3F => {
                // SRL
                ret = self.alu.srl(arg);
            }
            0x40..=0x7F => {
                // BIT
                let bit_index = (cb_code - 0x40) / 8;
                self.alu.test_bit(arg, bit_index);
            }
            0x80..=0xBF => {
                // RES
                let bit_index = (cb_code - 0x80) / 8;
                ret = self.alu.reset_bit(arg, bit_index);
            }
            0xC0..=0xFF => {
                // SET
                let bit_index = (cb_code - 0xC0) / 8;
                ret = self.alu.set_bit(arg, bit_index);
            }
        }

        if ret != arg {
            match cb_code & 0x7 {
                0x0 => self.regs.set_a(ret),
                0x1 => self.regs.set_c(ret),
                0x2 => self.regs.set_d(ret),
                0x3 => self.regs.set_e(ret),
                0x4 => self.regs.set_h(ret),
                0x5 => self.regs.set_l(ret),
                0x6 => { self.write_byte(self.regs.hl(), ret); }
                0x7 => self.regs.set_a(ret),
                _ => panic!()
            }
        }
    }

    fn read_bc_ref(&mut self) -> u8 {
        self.read_byte(self.regs.bc())
    }

    fn read_de_ref(&self) -> u8 {
        self.read_byte(self.regs.de())
    }

    fn read_hl_ref(&self) -> u8 {
        self.read_byte(self.regs.hl())
    }

    fn read_hl_ref_and_increment_hl(&mut self) -> u8 {
        let ret = self.read_hl_ref();
        self.regs.set_hl(self.alu.inc16(self.regs.hl()));
        ret
    }

    fn read_hl_ref_and_decrement_hl(&mut self) -> u8 {
        let ret = self.read_hl_ref();
        self.regs.set_hl(self.alu.dec16(self.regs.hl()));
        ret
    }

    fn write_hl_ref(&mut self, data: u8) {
        self.write_byte(self.regs.hl(), data);
    }

    fn write_hl_ref_and_increment_hl(&mut self, data: u8) {
        let addr = self.regs.hl();
        self.write_byte(addr, data);
        self.regs.set_hl(self.alu.inc16(addr));
    }

    fn write_hl_ref_and_decrement_hl(&mut self, data: u8) {
        let addr = self.regs.hl();
        self.write_byte(addr, data);
        self.regs.set_hl(self.alu.dec16(addr));
    }

    fn push(&mut self, r16: u16) {
        let be_bytes = r16.to_be_bytes();

        let sp = self.regs.sp();
        self.write_byte(sp, be_bytes[0]);
        self.write_byte(sp.wrapping_sub(1), be_bytes[1]);

        self.regs.set_sp(sp.wrapping_sub(2));
    }

    fn pop(&mut self) -> u16 {
        let sp = self.regs.sp();
        let l = self.read_byte(sp.wrapping_add(1));
        let h = self.read_byte(sp.wrapping_add(2));
        self.regs.set_sp(sp.wrapping_add(2));
        u16::from_be_bytes([h, l])
    }

    fn absolute_jump(&mut self, addr: u16) {
        self.next_pc = addr;
    }

    fn relative_jump(&mut self, offset: u8) {
        self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
    }

    fn begin_call(&mut self, call_addr: u16) {
        self.push(self.next_pc);
        self.next_pc = call_addr;
    }

    fn end_call(&mut self) {
        self.next_pc = self.pop();
    }
}

#[test]
fn unsigned_signed_test() {
    let offset: i8 = -1;
    let address: u16 = 0xff01;

    assert_eq!(0xff00u16, address.wrapping_add(offset as u16));
}
