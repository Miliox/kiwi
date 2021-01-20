pub mod alu;
pub mod asm;
pub mod flags;
pub mod interrupts;
pub mod regs;

pub trait Processor {
    // Execute the next instruction
    fn fetch_decode_execute_store_cycle(&mut self) -> u64;

    // Routine to check and handle interruption from software and hardware
    fn interrupt_service_routine(&mut self) -> bool;

    // Jump to target address
    fn jump_absolute(&mut self, target: u16);

    // Jump to target address if condition is true
    fn jump_absolute_if(&mut self, target: u16, cond: bool);

    // Jump relative to offset
    fn jump_relative(&mut self, offset: u8);

    // Jump relative to offset if condition is true
    fn jump_relative_if(&mut self, offset: u8, cond: bool);

    // Begin subroutine
    fn subroutine_call(&mut self, routine_addr: u16);

    // Begin subroutine if condition is true
    fn subroutine_call_if(&mut self, routine_addr: u16, cond: bool);

    // Return from subroutine
    fn subroutine_return(&mut self);

    // Return from subroutine if condition is true
    fn subroutine_return_if(&mut self, cond: bool);

    // Insert data into the stack
    fn stack_push(&mut self, data: u16);

    // Retrive data from stack
    fn stack_pop(&mut self) -> u16;
}