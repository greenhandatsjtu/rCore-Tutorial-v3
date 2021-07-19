const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            extern "C" {
                fn skernel();
                fn ekernel();
            }
            if (buf as isize) < (skernel as isize) || (buf as isize + len as isize) > (ekernel as isize) {
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