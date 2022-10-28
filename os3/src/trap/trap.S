.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.align 2
__alltraps:
    csrrw sp, sscratch, sp
    # sp -> kernel stack.
    addi sp, sp, -34*8

    SAVE_GP 1
    .set n, 3
    .rept 29
        SAVE_GP %n
        .set n, n+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp) # ctx.sstatus <- user_sstatus
    sd t1, 33*8(sp) # ctx.sepc <- user_pc
    csrr t2, sscratch
    sd t2, 2*8(sp)  # ctx.x[2] <- user_stack
    mv a0, sp
    call trap_handler

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

# Rust function define: fn __restore(ctx: TrapContext) -> ! ;
.align 2
__restore:
    mv sp, a0
    ld t0, 32*8(sp) // sstatus
    ld t1, 33*8(sp) // sepc
    ld t2, 2*8(sp)  // ctx.sp (user stack)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        LOAD_GP %n
        .set n, n+1
    .endr

    addi sp, sp, 34*8
    csrrw sp, sscratch, sp
    sret
