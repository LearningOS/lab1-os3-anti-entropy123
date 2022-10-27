use crate::{
    task::{run_next_task, Task, TaskState},
    timer::{get_time, get_time_us},
    trap::TrapContext,
};
const STDOUT: usize = 1;

enum Syscall {
    Exit,
    Write,
    GetTimeOfDay,
    YIELD,
}

impl From<usize> for Syscall {
    fn from(n: usize) -> Self {
        match n {
            93 => Self::Exit,          // 0x5d
            64 => Self::Write,         // 0x40
            169 => Self::GetTimeOfDay, // 0xa9
            124 => Self::YIELD,        // 0x7c
            _ => todo!(),
        }
    }
}

impl Syscall {
    fn handle(&self, task: &mut Task, arg1: usize, arg2: usize, arg3: usize) {
        let ret = match self {
            Syscall::Write => sys_write(arg1, arg2, arg3),
            Syscall::Exit => {task.state = TaskState::Exited; run_next_task()},
            Syscall::GetTimeOfDay => sys_gettimeofday(arg1, arg2) as usize,
            Syscall::YIELD => sys_yield(),
            _ => todo!(),
        };
        task.trap_ctx.set_a_n(10, ret);
    }
}

pub fn syscall_handler(ctx: &mut Task) {
    let trap_ctx = &mut ctx.trap_ctx;
    let (syscall_num, a0, a1, a2) = (
        trap_ctx.a_n(7),
        trap_ctx.a_n(0),
        trap_ctx.a_n(1),
        trap_ctx.a_n(2),
    );
    log::info!("syscall_num is {}", syscall_num);
    let syscall = Syscall::from(syscall_num);
    syscall.handle(ctx, a0, a1, a2)
}

fn sys_write(fd: usize, buf: usize, len: usize) -> usize {
    // follow line code has bug.
    // let _user_buf = unsafe { String::from_raw_parts(buf as *mut u8, len, len) };
    // follow line code is correct.
    let user_buf = {
        let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
        core::str::from_utf8(slice).unwrap()
    };

    log::info!("sys_write args, fd={}, buf=0x{:x}, len={}", fd, buf, len);
    if fd != STDOUT {
        unimplemented!()
    }
    print!("{}", user_buf);
    len
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

fn sys_gettimeofday(timeval_ptr: usize, _tz: usize) -> isize {
    let time = unsafe { &mut *(timeval_ptr as *mut TimeVal) };
    time.sec = get_time();
    time.usec = get_time_us();
    0
}

fn sys_yield() -> usize {
    return 0;
}

fn sys_exit() -> usize {
    run_next_task()
}
