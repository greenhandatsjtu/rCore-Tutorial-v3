use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
};
use crate::task::{task_mmap, task_munmap};
use crate::timer::{get_time_ms, get_time_s, get_time_us};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn gettime(ts: &mut TimeVal, _tz: usize) -> isize {
    ts.sec = get_time_s();
    ts.usec = get_time_us();
    0
}

pub fn setpriority(prio: isize) -> isize {
    if prio <= 1 {
        return -1;
    }
    prio
}

pub fn mmap(start: usize, len: usize, prot: usize) -> isize {
    if prot & 7 == 0 || prot & !(7 as usize) != 0 || start & 1 << 12 - 1 != 0 {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    if start & 0xfff != 0 {
        return -1;
    }
    task_mmap(start, len, prot)
}

pub fn munmap(start: usize, len: usize) -> isize {
    if start & 0xfff != 0 {
        return -1;
    };
    task_munmap(start, len)
}