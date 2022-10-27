use core::{fmt::Display, mem};

use riscv::register::{
    scause::{self, Exception, Trap},
    sstatus::{self, Sstatus, SPP},
    stval, stvec,
    utvec::TrapMode,
};

use crate::{task::run_next_task, syscall};

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
    pub fn app_init_context(app_base: usize, user_sp: usize) -> Self {
        let mut ctx = TrapContext {
            x: [0; 32],
            sepc: app_base,
            sstatus: {
                let mut sstatus = sstatus::read();
                sstatus.set_spp(SPP::User);
                sstatus
            },
        };
        ctx.x[2] = user_sp;
        ctx
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

impl TrapContext {
    pub fn a_n(&self, n: usize) -> usize {
        self.x[10 + n]
    }

    pub fn set_a_n(&mut self, n: usize, v: usize) {
        self.x[10 + n] = v
    }
}

// 在其它 crate 里调 __restore 好像会报链接错误, 所以要包装一层.
pub fn restore(ctx: usize) -> ! {
    unsafe { __restore(ctx) }
}

pub fn restore_from_trapctx(ctx: &TrapContext) -> ! {
    let ctx_addr = ctx as *const TrapContext as usize;
    log::debug!("try to get ctx addr by raw pointer, ctx_addr=0x{:x}", ctx_addr);
    log::debug!("restore_from_trapctx, ctx={}", ctx);
    restore(ctx_addr)
}

pub fn init() {
    log::info!("size of TrapContext is {}", trap_context_size);
    unsafe { stvec::write(__alltraps as usize, TrapMode::Direct) }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> ! {
    log::debug!("user trap context is {}", ctx);
    let scause = scause::read();
    let stval = stval::read();

    log::info!(
        "scause={:#?}, stval={:?}, sepc=0x{:x?}",
        scause.cause(),
        stval,
        ctx.sepc
    );

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            syscall::syscall_handler(ctx);
            ctx.sepc += 4;
            restore_from_trapctx(ctx)
        }
        Trap::Exception(Exception::LoadFault) => {
            log::error!("load fault, core dump");
            run_next_task();
        }
        Trap::Exception(Exception::StoreFault) => {
            log::error!("store fault, core dump");
            run_next_task();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            log::error!("illegal instruction, core dump");
            run_next_task();
        }
        _ => {
            unimplemented!()
        }
    }
}

pub fn enable_timer_interrupt() {}
