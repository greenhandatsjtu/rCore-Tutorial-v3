mod context;
mod switch;
mod task;

use crate::loader::{get_num_app, get_app_data};
use crate::trap::TrapContext;
use core::cell::RefCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::vec::Vec;

pub use context::TaskContext;
use crate::mm::MapPermission;

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
        }
        TaskManager {
            num_app,
            inner: RefCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
            }),
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        self.inner.borrow_mut().tasks[0].task_status = TaskStatus::Running;
        let next_task_cx_ptr2 = self.inner.borrow().tasks[0].get_task_cx_ptr2();
        let _unused: usize = 0;
        unsafe {
            __switch(
                &_unused as *const _,
                next_task_cx_ptr2,
            );
        }
    }

    fn get_current_task(&self) -> usize {
        let current = self.inner.borrow().current_task;
        current
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| {
                inner.tasks[*id].task_status == TaskStatus::Ready
            })
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe {
                __switch(
                    current_task_cx_ptr2,
                    next_task_cx_ptr2,
                );
            }
        } else {
            panic!("All applications completed!");
        }
    }

    pub fn mmap(&self, start: usize, len: usize, prot: usize) -> isize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        if !inner.tasks[current].memory_set.check_all_not_mapped(start, start + len) {
            return -1;
        }
        core::mem::drop(inner);
        let mut inner = self.inner.borrow_mut();
        inner.tasks[current].memory_set.mmap(start, start + len, MapPermission::from_bits((prot << 1) as u8).unwrap() | MapPermission::U)
    }

    pub fn unmap(&self, start: usize, len: usize) -> isize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        if !inner.tasks[current].memory_set.check_all_mapped(start, start + len) {
            return -1;
        }
        core::mem::drop(inner);
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].memory_set.munmap(start, start + len)
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    // add 10 ms execute time to current task, and if execute time >= 5s, stop it, else suspend it
    let current = TASK_MANAGER.get_current_task();
    let mut inner = TASK_MANAGER.inner.borrow_mut();
    let exec_time = &mut inner.tasks[current].exec_time_ms;
    *exec_time += 10;
    // debug!("TASK: {}, EXECUTE_TIME: {}ms",current,*exec_time);
    core::mem::drop(inner);
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn get_current_task() -> usize {
    TASK_MANAGER.get_current_task()
}

pub fn task_mmap(start: usize, len: usize, prot: usize) -> isize {
    TASK_MANAGER.mmap(start, len, prot)
}

pub fn task_munmap(start: usize, len: usize) -> isize {
    TASK_MANAGER.unmap(start, len)
}