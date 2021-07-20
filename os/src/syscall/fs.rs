use crate::batch::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, get_app_stack, USER_STACK_SIZE};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // check app mem (.bin) and user stack
            if !((buf as usize) >= APP_BASE_ADDRESS && (buf as usize + len) <= APP_BASE_ADDRESS + APP_SIZE_LIMIT || (buf as usize) >= get_app_stack() - USER_STACK_SIZE && (buf as usize + len) <= get_app_stack()) {
                println!("Illegal write!");
                return -1;
            }
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