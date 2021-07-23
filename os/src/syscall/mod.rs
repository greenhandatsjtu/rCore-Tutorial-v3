const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_SET_PRIORITY: usize = 140;
pub const SYSCALL_GET_TIME: usize = 169;

mod fs;
mod process;

use fs::*;
use process::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_SET_PRIORITY => setpriority(args[0] as isize),
        SYSCALL_GET_TIME => gettime(unsafe { &mut *(args[0] as *mut TimeVal) }, args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

