use super::trap::restore;
use crate::loader::init_app_cx;
use lazy_static::lazy_static;
use spin::Mutex;

struct TaskManager {
    current_task: usize,
}

lazy_static! {
    static ref TM: Mutex<TaskManager> = Mutex::new(TaskManager { current_task: 0 });
}

pub fn run_next_task() -> ! {
    let mut task_manager = TM.lock();
    let app_id = task_manager.current_task;

    log::info!("will run next task, task_idx={}", app_id);

    task_manager.current_task += 1;
    drop(task_manager);

    let ctx_addr = init_app_cx(app_id);
    log::info!("app_{} ctx_addr=0x{:x}", app_id, ctx_addr);
    restore(ctx_addr)
}
