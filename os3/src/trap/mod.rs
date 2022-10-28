use crate::{
    loader::{get_app_sp, get_base_i},
    syscall::{self, sys_exit},
    task::{run_next_task, Task, TaskState},
    timer::set_next_trigger,
};
use core::{fmt::Display, mem};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie,
    sstatus::{self, Sstatus, SPP},
    stval, stvec,
    utvec::TrapMode,
};

core::arch::global_asm!(include_str!("trap.S"));
extern "C" {
    fn __alltraps() -> !;
    fn __restore(ctx: usize) -> !;
}

// Trap 上下文应该保存全部寄存器, 因为 trap 不是应用主动调用的.
// 那么自然也就不会有 caller-saved 寄存器.
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

#[no_mangle]
pub static trap_context_size: usize = mem::size_of::<TrapContext>();

impl TrapContext {
    pub fn app_init_context(app_id: usize) -> Self {
        let mut ctx = TrapContext {
            x: [0; 32],
            sepc: get_base_i(app_id),
            sstatus: {
                let mut sstatus = sstatus::read();
                sstatus.set_spp(SPP::User);
                sstatus
            },
        };
        ctx.x[2] = get_app_sp(app_id);
        ctx
    }

    pub fn reg_a(&self, n: usize) -> usize {
        self.x[10 + n]
    }

    pub fn set_reg_a(&mut self, n: usize, v: usize) {
        self.x[10 + n] = v
    }
}

impl Display for TrapContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "TrapContext {{x: {:x?},\n sstatus: 0x{:x},\n sepc: 0x{:x}\n}}",
            self.x,
            self.sstatus.bits(),
            self.sepc
        ))
    }
}

// 在其它 crate 里调 __restore 好像会报链接错误, 所以要包装一层.
pub fn restore(ctx: usize) -> ! {
    log::debug!("try to get ctx addr by raw pointer, ctx_addr=0x{:x}", ctx);
    log::debug!("restore_from_trapctx, ctx={}", unsafe {
        &*(ctx as *const TrapContext)
    });
    unsafe { __restore(ctx) }
}

pub fn init() {
    log::info!("size of TrapContext is {}", trap_context_size);
    unsafe { stvec::write(__alltraps as usize, TrapMode::Direct) }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut Task) -> ! {
    let trap_ctx = &mut ctx.trap_ctx;
    log::debug!(
        "trap_handler, task.id={}, task.trap_ctx={}",
        ctx.id,
        trap_ctx
    );
    let scause = scause::read();
    let stval = stval::read();

    log::info!("scause={:?}, stval={:?}", scause.cause(), stval);

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            trap_ctx.sepc += 4;
            syscall::syscall_handler(ctx);
            ctx.set_state(TaskState::Ready);
            run_next_task();
        }
        Trap::Exception(Exception::LoadFault) => {
            log::error!("load fault, core dump");
            sys_exit(ctx);
        }
        Trap::Exception(Exception::StoreFault) => {
            log::error!("store fault, core dump");
            sys_exit(ctx);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            log::error!("illegal instruction, core dump");
            sys_exit(ctx);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            log::info!("Timer interrupt.");
            set_next_trigger();
            ctx.set_state(TaskState::Ready);
            run_next_task();
        }
        _ => {
            unimplemented!()
        }
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
