#[allow(dead_code)]
pub const TICKS_PER_SECOND :u64 = 4_194_304;

pub trait TickProducer {
    fn step(&mut self) -> u64;
}

pub trait TickConsumer {
    fn sync(&mut self, ticks: u64);
}