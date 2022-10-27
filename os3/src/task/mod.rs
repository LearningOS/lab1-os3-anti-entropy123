use core::panic;

use super::trap::restore;
use crate::{
    loader::{get_num_app, init_task_cx},
    trap::TrapContext,
};
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_APP_NUM: usize = 16;

#[derive(PartialEq)]
pub enum TaskState {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[repr(C)]
pub struct Task {
    pub trap_ctx: TrapContext,
    pub id: usize,
    pub state: TaskState,
}

impl Task {
    fn new(app_id: usize) -> Self {
        Self {
            trap_ctx: TrapContext::app_init_context(app_id),
            id: app_id,
            state: TaskState::Ready,
        }
    }

    pub fn get_ptr(&self) -> usize {
        self as *const _ as usize
    }
}

struct TaskManager {
    next_task: usize,
    task_list: [Option<usize>; MAX_APP_NUM],
}

impl TaskManager {
    fn new() -> Self {
        let mut task_list: [Option<usize>; MAX_APP_NUM] = [None; MAX_APP_NUM];
        for i in 0..get_num_app() {
            let task_ptr = init_task_cx(Task::new(i));
            task_list[i] = Some(task_ptr);
        }
        Self {
            task_list,
            next_task: 0,
        }
    }

    fn get_task_ref(&self, app_id: usize) -> Option<&Task> {
        self.task_list[app_id].map(|ptr| unsafe { &*(ptr as *mut Task) })
    }

    fn get_task_mut_ref(&self, app_id: usize) -> Option<&mut Task> {
        self.task_list[app_id].map(|ptr| unsafe { &mut *(ptr as *mut Task) })
    }

    fn find_next_ready_task(&self) -> Option<&Task> {
        let current = self.next_task;
        for i in current..(current + MAX_APP_NUM) {
            let app_id = i % MAX_APP_NUM;
            let task = self.get_task_mut_ref(app_id);
            if task.is_none() {
                continue;
            }
            let mut task = task.unwrap();
            if task.state == TaskState::Ready {
                task.state = TaskState::Running;
                return Some(task);
            }
        }
        None
    }
}

lazy_static! {
    static ref TM: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn run_next_task() -> ! {
    let mut task_manager = TM.lock();
    let task = task_manager.find_next_ready_task();
    if task.is_none() {
        panic!("all task complete!");
    }
    let task = task.unwrap();
    let ctx_ptr = task.get_ptr();
    log::info!("will run next task, task_idx={}", task.id);
    log::debug!("app_{} ctx_addr=0x{:x}", task.id, ctx_ptr);
    task_manager.next_task = task.id + 1;
    drop(task_manager);

    restore(ctx_ptr)
}
