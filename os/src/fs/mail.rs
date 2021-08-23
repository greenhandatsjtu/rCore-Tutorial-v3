use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::fs::File;
use crate::mm::UserBuffer;
use core::any::Any;


pub struct Mailbox {
    buffer: Arc<Mutex<MailboxRingBuffer>>,
}

const MAILBOX_BUFFER_SIZE: usize = 16;

pub struct MailboxRingBuffer {
    arr: [Vec<u8>; MAILBOX_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: MailboxBufferStatus,
}

#[derive(Copy, Clone, PartialEq)]
enum MailboxBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}


impl Mailbox {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(MailboxRingBuffer::new()))
        }
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().status == MailboxBufferStatus::EMPTY
    }
    pub fn is_full(&self) -> bool {
        self.buffer.lock().status == MailboxBufferStatus::FULL
    }
}

impl MailboxRingBuffer {
    pub fn new() -> Self {
        let arr: [Vec<u8>; MAILBOX_BUFFER_SIZE] = Default::default();
        Self {
            arr,
            head: 0,
            tail: 0,
            status: MailboxBufferStatus::EMPTY,
        }
    }
    pub fn read_mail(&mut self) -> &Vec<u8> {
        self.status = MailboxBufferStatus::NORMAL;
        let mail = &self.arr[self.head];
        self.head = (self.head + 1) % MAILBOX_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = MailboxBufferStatus::EMPTY;
        }
        mail
    }
    pub fn write_mail(&mut self, mail: &Vec<u8>) {
        self.status = MailboxBufferStatus::NORMAL;
        self.arr[self.tail] = Vec::new();
        for byte in mail.iter() {
            self.arr[self.tail].push(byte.clone());
        }
        self.tail = (self.tail + 1) % MAILBOX_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = MailboxBufferStatus::FULL;
        }
    }
}

impl File for Mailbox {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, buf: UserBuffer) -> usize {
        let mut ring_buffer = self.buffer.lock();
        let buf_len = buf.len();
        let mut buf_iter = buf.into_iter();
        let mail = ring_buffer.read_mail();
        let mail_len = mail.len();
        for i in 0..mail_len {
            if let Some(byte_ref) = buf_iter.next() {
                unsafe { *byte_ref = mail[i]; }
            }
        }
        mail_len.min(buf_len)
    }

    fn write(&self, buf: UserBuffer) -> usize {
        let mut ring_buffer = self.buffer.lock();
        let buf_len = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut mail = Vec::new();
        for _ in 0..buf_len {
            if let Some(byte_ref) = buf_iter.next() {
                mail.push(unsafe { *byte_ref });
            }
        }
        let mail_len = mail.len();
        ring_buffer.write_mail(&mail);
        mail_len.min(buf_len)
    }
}