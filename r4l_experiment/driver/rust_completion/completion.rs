// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::Err;
use kernel::prelude::*;
use kernel::sync::Mutex;
use kernel::task::Task;
use kernel::{chrdev, file};

const GLOBALMEM_SIZE: usize = 0x1000;

module! {
    type: RustCompletion,
    name: "completion",
    author: "zhou-xingxing",
    description: "Rust completion device sample",
    license: "GPL",
}

static GLOBALMEM_BUF: Mutex<[u8;GLOBALMEM_SIZE]> = unsafe {
    Mutex::new([0u8;GLOBALMEM_SIZE])
};

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8;GLOBALMEM_SIZE]>,
}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("function open is invoked");
        Ok(
            Box::try_new(RustFile {
                inner: &GLOBALMEM_BUF
            })?
        )
    }

    fn write(_this: &Self,_file: &file::File,_reader: &mut impl kernel::io_buffer::IoBufferReader,_offset:u64,) -> Result<usize> {
        pr_info!("function write is invoked\n");
        pr_info!("process {} awakening the readers...\n", Task::current().pid());
        let offset=_offset as usize;
        let mut dev=_this.inner.lock();
        // 避免字节长度超出字符设备
        let len=core::cmp::min(_reader.len(), dev.len().saturating_sub(offset));
        _reader.read_slice(&mut dev[offset..][..len])?;
        Ok(len)
    }

    fn read(_this: &Self,_file: &file::File,_writer: &mut impl kernel::io_buffer::IoBufferWriter,_offset:u64,) -> Result<usize> {
        pr_info!("function read is invoked\n");
        pr_info!("process {} is going to sleep\n", Task::current().pid());
        let offset=_offset as usize;
        let dev=_this.inner.lock();
            // 避免字节长度超出缓冲区
        let len = core::cmp::min(_writer.len(),dev.len().saturating_sub(offset));
        _writer.write_slice(&dev[offset..][..len])?;
        pr_info!("process {} awoken\n", Task::current().pid());
        Ok(len)
    }
}

struct RustCompletion {
    _dev: Pin<Box<chrdev::Registration<2>>>,
}

impl kernel::Module for RustCompletion {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust completion device (init)\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<RustFile>()?;
        chrdev_reg.as_mut().register::<RustFile>()?;

        Ok(RustCompletion { _dev: chrdev_reg })
    }
}

impl Drop for RustCompletion {
    fn drop(&mut self) {
        pr_info!("Rust completion device (exit)\n");
    }
}
