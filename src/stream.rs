use cef_sys::{
    _cef_stream_reader_t,
    _cef_stream_writer_t,
};
use std::io::{self, SeekFrom};

ref_counted_ptr!{
    pub struct StreamReader(*mut _cef_stream_reader_t);
}

ref_counted_ptr!{
    pub struct StreamWriter(*mut _cef_stream_writer_t);
}

// TODO: CREATE CUSTOM READERS AND WRITERS

impl StreamReader {
    /// Read raw binary data, returning how many bytes were read.
    pub fn read(&self, data: &mut [u8]) -> usize {
        unsafe{ (self.0.read.unwrap())(self.as_ptr(), data.as_mut_ptr() as *mut _, 1, data.len()) }
    }

    /// Seek to the specified position.
    pub fn seek(&self, whence: SeekFrom) -> io::Result<()> {
        let (whence, offset) = match whence {
            SeekFrom::Start(o) => (libc::SEEK_SET, o as i64),
            SeekFrom::End(o) => (libc::SEEK_END, o),
            SeekFrom::Current(o) => (libc::SEEK_CUR, o),
        };
        let result = unsafe{ (self.0.seek.unwrap())(self.as_ptr(), offset, whence) };
        match result {
            0 => Ok(()),
            _ => Err(io::Error::from_raw_os_error(result))
        }
    }

    /// Return the current offset position.
    pub fn tell(&self) -> usize {
        unsafe{ (self.0.tell.unwrap())(self.as_ptr()) as usize }
    }

    /// Returns `true` if the reader is at the end of the file.
    pub fn eof(&self) -> bool {
        unsafe{ (self.0.eof.unwrap())(self.as_ptr()) != 0 }
    }

    /// Returns `true` if this reader performs work like accessing the file system
    /// which may block. Used as a hint for determining the thread to access the
    /// reader from.
    pub fn may_block(&self) -> bool {
        unsafe{ (self.0.may_block.unwrap())(self.as_ptr()) != 0 }
    }
}

impl StreamWriter {
    /// Write raw binary data, returning how many bytes were written.
    pub fn write(&self, data: &[u8]) -> usize {
        unsafe{ (self.0.write.unwrap())(self.as_ptr(), data.as_ptr() as *mut _, 1, data.len()) }
    }

    /// Seek to the specified position.
    pub fn seek(&self, whence: SeekFrom) -> io::Result<()> {
        let (whence, offset) = match whence {
            SeekFrom::Start(o) => (libc::SEEK_SET, o as i64),
            SeekFrom::End(o) => (libc::SEEK_END, o),
            SeekFrom::Current(o) => (libc::SEEK_CUR, o),
        };
        let result = unsafe{ (self.0.seek.unwrap())(self.as_ptr(), offset, whence) };
        match result {
            0 => Ok(()),
            _ => Err(io::Error::from_raw_os_error(result))
        }
    }

    /// Return the current offset position.
    pub fn tell(&self) -> usize {
        unsafe{ (self.0.tell.unwrap())(self.as_ptr()) as usize }
    }

    /// Returns `true` if the writer is at the end of the file.
    pub fn flush(&self) -> io::Result<()> {
        let result = unsafe{ (self.0.flush.unwrap())(self.as_ptr()) };
        match result {
            0 => Ok(()),
            _ => Err(io::Error::from_raw_os_error(result))
        }
    }

    /// Returns `true` if this writer performs work like accessing the file system
    /// which may block. Used as a hint for determining the thread to access the
    /// writer from.
    pub fn may_block(&self) -> bool {
        unsafe{ (self.0.may_block.unwrap())(self.as_ptr()) != 0 }
    }
}
