use crate::mm::{UserBuffer, translated_byte_buffer, translated_refmut, check_buf_read, translated_str};
use crate::task::{current_user_token, current_task, TASK_MANAGER, PidHandle};
use crate::fs::{make_pipe, File, OpenFlags, open_file};
use log::error;
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    if !check_buf_read(token, buf, len) {
        error!("Illegal write! PID: {}, BUF_ADDR: [{:#x},{:#x})",task.pid.0,buf as usize,buf as usize+len);
        return -1;
    }
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        let buffers = match translated_byte_buffer(token, buf, len) {
            Some(b) => b,
            None => return -1,
        };
        file.write(
            UserBuffer::new(buffers)
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        let buffers = match translated_byte_buffer(token, buf, len) {
            Some(b) => b,
            None => return -1,
        };
        file.read(
            UserBuffer::new(buffers)
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.acquire_inner_lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn mailread(buf: *mut u8, mut len: usize) -> isize {
    let task = current_task().unwrap();
    if task.acquire_inner_lock().mailbox.is_empty() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    if len > 256 {
        len = 256
    }
    let token = current_user_token();
    let buffers = match translated_byte_buffer(token,buf,len){
        Some(b)=>b,
        None=>return -1,
    };
    let user_buf = UserBuffer::new(buffers);
    let count = task.acquire_inner_lock().mailbox.read(user_buf) as isize;
    count
}

pub fn mailwrite(pid: usize, buf: *mut u8, mut len: usize) -> isize {
    let current_task = current_task().unwrap();
    let task = if current_task.pid.0 == pid {
        current_task
    } else {
        match TASK_MANAGER.lock().get(pid) {
            Some(t) => t,
            None => {
                return -1;
            }
        }
    };
    if task.acquire_inner_lock().mailbox.is_full() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    if len > 256 {
        len = 256
    }
    let token = current_user_token();
    let buffers = match translated_byte_buffer(token,buf,len){
        Some(b)=>b,
        None=>return -1,
    };
    let user_buf = UserBuffer::new(buffers);
    let count = task.acquire_inner_lock().mailbox.write(user_buf) as isize;
    count
}
pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}