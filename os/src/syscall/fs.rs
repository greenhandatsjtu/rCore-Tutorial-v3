use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, USER_STACK_SIZE};
use crate::task::get_current_task;
use log::{error, debug, Level::Error};
use crate::loader::get_app_stack;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // check memory (.bin)
            let mem_base = APP_BASE_ADDRESS + get_current_task() * APP_SIZE_LIMIT;
            // check user stack
            let stack_top = get_app_stack(get_current_task());
            if !((buf as usize) >= mem_base && (buf as usize + len) <= mem_base + APP_SIZE_LIMIT || (buf as usize) >= stack_top - USER_STACK_SIZE && (buf as usize + len) <= stack_top) {
                error!("Illegal write! APP_ID: {}, APP_MEM: [{:#x},{:#x}), APP_STACK: [{:#x},{:#x}), BUF_ADDR: [{:#x},{:#x})",get_current_task(),mem_base,mem_base+APP_SIZE_LIMIT,stack_top-USER_STACK_SIZE,stack_top,buf as usize,buf as usize+len);
                return -1;
            }
            debug!("APP_ID: {}, APP_MEM: [{:#x},{:#x}), APP_STACK: [{:#x},{:#x}), BUF_ADDR: [{:#x},{:#x})",get_current_task(),mem_base,mem_base+APP_SIZE_LIMIT,stack_top-USER_STACK_SIZE,stack_top,buf as usize,buf as usize+len);
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            println!("Unsupported fd in sys_write!");
            -1
        }
    }
}