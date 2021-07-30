use crate::mm::{translated_byte_buffer, check_buf_read};
use crate::task::current_user_token;
use crate::task::get_current_task;
use log::error;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            if !check_buf_read(current_user_token(), buf, len) {
                error!("Illegal write! TASK_ID: {}, BUF_ADDR: [{:#x},{:#x})",get_current_task(),buf as usize,buf as usize+len);
                return -1;
            }
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}