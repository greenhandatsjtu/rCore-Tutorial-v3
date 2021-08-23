mod pipe;
mod stdio;
mod inode;
mod mail;

use crate::mm::UserBuffer;
use core::any::Any;

pub trait File: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
pub use inode::{OSInode, open_file, OpenFlags, list_apps, linkat, unlinkat};
pub use mail::Mailbox;