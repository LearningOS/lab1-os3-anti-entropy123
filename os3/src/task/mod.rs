use core::panic;

use super::trap::restore;
use crate::{
    config::MAX_SYSCALL_NUM,
    loader::{get_num_app, init_task_cx},
    timer::{get_time, get_time_ms},
    trap::TrapContext,
};
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_APP_NUM: usize = 16;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TaskState {
    UnInit,
    Ready,
    Running,
    Exited,
}

// #[derive(Clone, Debug)]
// pub struct TaskInfo {
//     pub state: TaskState,
//     pub syscall_times: [u32; MAX_SYSCALL_NUM],
//     pub exec_time: usize,
// }

#[repr(C)]
pub struct Task {
    pub trap_ctx: TrapContext,
    pub id: usize,
    state: TaskState,
    pub start_time_ms: usize,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
}

impl Task {
    fn new(app_id: usize) -> Self {
        Self {
            trap_ctx: TrapContext::app_init_context(app_id),
            id: app_id,
            state: TaskState::UnInit,
            syscall_times: [0; MAX_SYSCALL_NUM],
            start_time_ms: 0,
        }
    }

    pub fn get_ptr(&self) -> usize {
        self as *const _ as usize
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn set_state(&mut self, state: TaskState) {
        self.state = state
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

    // fn get_task_ref(&self, app_id: usize) -> Option<&Task> {
    //     self.task_list[app_id].map(|ptr| unsafe { &*(ptr as *mut Task) })
    // }

    fn get_task_mut_ref(&self, app_id: usize) -> Option<&mut Task> {
        self.task_list[app_id].map(|ptr| unsafe { &mut *(ptr as *mut Task) })
    }

    fn find_next_ready_task(&self) -> Option<&mut Task> {
        let current = self.next_task;
        for i in current..(current + MAX_APP_NUM) {
            let app_id = i % MAX_APP_NUM;
            let task = match self.get_task_mut_ref(app_id) {
                None => continue,
                Some(task) => task,
            };
            if task.state == TaskState::UnInit {
                task.state = TaskState::Ready;
                task.start_time_ms = get_time_ms();
            }
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
    let task = match task_manager.find_next_ready_task() {
        None => panic!("all task complete!"),
        Some(task) => task,
    };
    let cur_task_id = task.id;
    let ctx_ptr = task.get_ptr();
    task_manager.next_task = task.id + 1;
    drop(task_manager);
    log::info!("will run next task, task_idx={}", cur_task_id);
    log::debug!("app_{} ctx_addr=0x{:x}", cur_task_id, ctx_ptr);
    restore(ctx_ptr)
}
