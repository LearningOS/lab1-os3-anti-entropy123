use crate::{task::run_next_task, trap::TrapContext};
const STDOUT: usize = 1;

enum Syscall {
    Exit,
    Write,
    Sleep,
}

impl From<usize> for Syscall {
    fn from(n: usize) -> Self {
        match n {
            0x5d => Self::Exit,
            0x40 => Self::Write,
            0xa9 => Self::Sleep,
            _ => todo!(),
        }
    }
}

impl Syscall {
    fn handle(&self, arg1: usize, arg2: usize, arg3: usize) -> usize {
        match self {
            Syscall::Write => sys_write(arg1, arg2, arg3),
            Syscall::Exit => sys_exit(),
            _ => todo!(),
        }
    }
}

pub fn syscall_handler(ctx: &mut TrapContext) {
    let syscall_num = ctx.a_n(7);
    log::debug!("syscall_num is {}", syscall_num);
    let syscall = Syscall::from(syscall_num);
    let ret = syscall.handle(ctx.a_n(0), ctx.a_n(1), ctx.a_n(2));
    ctx.set_a_n(0, ret);
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

fn sys_exit() -> usize {
    run_next_task()
}
