mod context;
mod switch;
mod task;

use crate::config::{MAX_APP_NUM, DEFAULT_PRIORITY, BIG_STRIDE};
use crate::loader::{get_num_app, init_app_cx};
use core::cell::RefCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use log::debug;
use heapless::binary_heap::{BinaryHeap, Min};

pub use context::TaskContext;
use crate::task::task::SchedInfo;
use core::borrow::BorrowMut;

pub struct TaskManager {
    num_app: usize,
    priority_queue: RefCell<BinaryHeap<SchedInfo, Min, MAX_APP_NUM>>,
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut priority_queue:BinaryHeap<SchedInfo,Min,MAX_APP_NUM> = BinaryHeap::new();
        let mut tasks = [
            TaskControlBlock { task_cx_ptr: 0, task_status: TaskStatus::UnInit,exec_time_ms:0,priority:DEFAULT_PRIORITY,pass:0};
            MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx_ptr = init_app_cx(i) as * const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
            priority_queue.push(SchedInfo{id:i,pass:0});
        }
        TaskManager {
            num_app,
            priority_queue:RefCell::new(priority_queue),
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
        let mut queue = self.priority_queue.borrow_mut();
        if let Some(mut task) = queue.pop() {
            task.pass += BIG_STRIDE / inner.tasks[current].priority;
            queue.push(task);
        }
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        self.priority_queue.borrow_mut().pop(); // remove current task from priority queue
    }

    fn find_next_task(&self) -> Option<usize> {
        // let inner = self.inner.borrow();
        // let current = inner.current_task;
        // (current + 1..current + self.num_app + 1)
        //     .map(|id| id % self.num_app)
        //     .find(|id| {
        //         inner.tasks[*id].task_status == TaskStatus::Ready
        //     })
        return match self.priority_queue.borrow().peek() {
            Some(info) => {
                Some(info.id)
            }
            _ => None
        };
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

    //set current task priority
    fn set_current_prio(&self, prio: usize) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].priority = prio;
        debug!("Task {} set priority to {}",current,prio);
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
    let exec_time = exec_time.clone();
    core::mem::drop(inner);
    if exec_time >= 5000 {
        debug!("Execute time exceed 5 s, stop task: {}",current);
        mark_current_exited();
    } else { mark_current_suspended(); }
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn get_current_task() -> usize {
    TASK_MANAGER.get_current_task()
}

pub fn set_current_task_prio(prio: usize) {
    TASK_MANAGER.set_current_prio(prio);
}