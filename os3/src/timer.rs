use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_MILLISEC: usize = 1_000;
const MILLI_PER_SEC: usize = 1_000;
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}

pub fn get_time_ms() -> usize {
    get_time_us() / MICRO_PER_MILLISEC
}

pub fn get_time_s() -> usize {
    get_time_us() / MICRO_PER_SEC
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}
pub fn set_time_val(time: &mut TimeVal) {
    let cpu_time = get_time();
    time.sec = cpu_time / CLOCK_FREQ;
    time.usec = (cpu_time - time.sec * CLOCK_FREQ) / (CLOCK_FREQ / MICRO_PER_SEC)
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
