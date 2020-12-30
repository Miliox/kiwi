use crate::cpu::Cpu;
use crate::mmu::Mmu;

#[allow(dead_code)]
pub struct Mediator {
    cpu: Cpu,
    mmu: Mmu,
}