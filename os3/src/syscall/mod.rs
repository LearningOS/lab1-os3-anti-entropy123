use crate::{
    config::MAX_SYSCALL_NUM,
    task::{run_next_task, Task, TaskState},
    timer::{self, get_time, get_time_ms, get_time_us, TimeVal},
    trap::TrapContext,
};
const STDOUT: usize = 1;

#[derive(Debug)]
enum Syscall {
    Exit,
    Write,
    GetTimeOfDay,
    Yield,
    TaskInfo,
}

impl From<usize> for Syscall {
    fn from(n: usize) -> Self {
        match n {
            64 => Self::Write,         // 0x40
            93 => Self::Exit,          // 0x5d
            124 => Self::Yield,        // 0x7c
            169 => Self::GetTimeOfDay, // 0xa9
            410 => Self::TaskInfo,
            _ => todo!("unsupported syscall"),
        }
    }
}

impl Syscall {
    fn handle(&self, task: &mut Task, arg1: usize, arg2: usize, arg3: usize) {
        let ret = match self {
            Syscall::Write => sys_write(arg1, arg2, arg3),
            Syscall::Exit => sys_exit(task),
            Syscall::GetTimeOfDay => sys_gettimeofday(arg1, arg2) as usize,
            Syscall::Yield => sys_yield(),
            Syscall::TaskInfo => sys_taskinfo(&task, arg1),
            _ => todo!("unsupported syscall handle function"),
        };
        task.trap_ctx.set_reg_a(0, ret);
        log::info!(
            "syscall ret={}, task.trap_ctx.x[10]={}",
            ret,
            task.trap_ctx.reg_a(0)
        );
    }
}

pub fn syscall_handler(ctx: &mut Task) {
    let trap_ctx = &mut ctx.trap_ctx;
    let (syscall_num, a0, a1, a2) = (
        trap_ctx.reg_a(7),
        trap_ctx.reg_a(0),
        trap_ctx.reg_a(1),
        trap_ctx.reg_a(2),
    );
    ctx.syscall_times[syscall_num] += 1;
    let syscall = Syscall::from(syscall_num);
    log::info!(
        "syscall_handler, num={}, name={:?}, syscall_times={:?}",
        syscall_num,
        syscall,
        ctx.syscall_times
    );
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

fn sys_gettimeofday(timeval_ptr: usize, _tz: usize) -> isize {
    let time = unsafe { &mut *(timeval_ptr as *mut TimeVal) };
    timer::set_time_val(time);
    0
}

fn sys_yield() -> usize {
    return 0;
}

pub fn sys_exit(task: &mut Task) -> ! {
    task.set_state(TaskState::Exited);
    run_next_task()
}

#[derive(Debug)]
pub struct TaskInfo {
    pub state: TaskState,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub exec_time: usize,
}

fn sys_taskinfo(task: &Task, user_info: usize) -> usize {
    let taskinfo = unsafe { &mut *(user_info as *mut TaskInfo) };
    *taskinfo = TaskInfo {
        state: task.state(),
        syscall_times: task.syscall_times,
        exec_time: get_time_ms() - task.start_time_ms,
    };
    log::debug!("sys_taskinfo, copyout user_info={:?}", taskinfo);
    0
}
