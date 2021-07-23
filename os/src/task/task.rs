use core::cmp::Ordering;

pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub exec_time_ms: usize,
    pub priority: usize,
    pub pass: usize,
}

impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

pub struct SchedInfo {
    pub id: usize,
    pub pass: usize,
}

impl Eq for SchedInfo {}

impl PartialEq for SchedInfo {
    fn eq(&self, other: &Self) -> bool {
        self.pass == other.pass
    }
}

impl PartialOrd for SchedInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchedInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        return if self.pass > other.pass {
            Ordering::Greater
        } else if self.pass == other.pass {
            Ordering::Equal
        } else {
            Ordering::Less
        };
    }
}