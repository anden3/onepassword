use core::{ffi, fmt::Display, ptr::NonNull};

use crate::util;

use super::errors::CallStatus;

#[link(name = "op_uniffi_core", kind = "raw-dylib")]
unsafe extern "C" {
    #[link_name = "ffi_op_uniffi_core_rustbuffer_alloc"]
    unsafe fn rustbuffer_alloc(size: u32, status: *mut CallStatus) -> RustBuffer;
    #[link_name = "ffi_op_uniffi_core_rustbuffer_from_bytes"]
    unsafe fn rustbuffer_from_bytes(bytes: ForeignBytes, status: *mut CallStatus) -> RustBuffer;
    #[link_name = "ffi_op_uniffi_core_rustbuffer_free"]
    unsafe fn rustbuffer_free(buffer: RustBuffer, status: *mut CallStatus);
    #[link_name = "ffi_op_uniffi_core_rustbuffer_reserve"]
    unsafe fn rustbuffer_reserve(
        buffer: RustBuffer,
        additional: u32,
        status: *mut CallStatus,
    ) -> RustBuffer;
}

#[repr(C)]
struct ForeignBytes {
    len: u32,
    data: *const ffi::c_char,
}

impl<T: AsRef<[u8]>> From<T> for ForeignBytes {
    fn from(value: T) -> Self {
        let slice = value.as_ref();
        let data = slice.as_ptr().cast();

        Self {
            data,
            len: slice.len() as _,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct RustBuffer {
    capacity: u32,
    pub len: u32,
    data: Option<NonNull<ffi::c_char>>,
}

impl Display for RustBuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.data.is_none() {
            return f.write_str("");
        };

        match core::str::from_utf8(self.as_ref()) {
            Ok(str) => f.write_str(str),
            Err(_) => f.write_str(&String::from_utf8_lossy(self.as_ref())),
        }
    }
}

impl RustBuffer {
    pub fn with_capacity(size: u32) -> RustBuffer {
        let Ok(mut buffer) = unsafe { util::rust_call!(rustbuffer_alloc, size) };
        buffer.len = 0;
        buffer
    }

    pub fn reserve(&mut self, additional: u32) {
        let buffer = core::mem::take(self);
        let Ok(new_buf) = unsafe { util::rust_call!(rustbuffer_reserve, buffer, additional) };
        *self = new_buf;
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        if self.len + (buf.len() as u32) > self.capacity {
            self.reserve(buf.len() as u32);
        }

        let current_len = self.len as usize;
        let pointer = self.data.expect("buffer should not be empty");

        for (i, byte) in buf.iter().copied().enumerate() {
            unsafe { pointer.add(current_len + i).write(byte as _) };
        }

        self.len = (current_len + buf.len()) as u32;
        buf.len()
    }
}

impl From<&str> for RustBuffer {
    fn from(value: &str) -> Self {
        let mut buffer = Self::with_capacity(value.len() as _);
        buffer.write(value.as_bytes());
        buffer
    }
}

impl Drop for RustBuffer {
    fn drop(&mut self) {
        if self.data.is_none() {
            return;
        }

        let Ok(_) = unsafe { util::rust_call!(rustbuffer_free, core::mem::take(self)) };
    }
}

impl AsRef<[u8]> for RustBuffer {
    fn as_ref(&self) -> &[u8] {
        let Some(data) = self.data else { return &[] };
        unsafe { core::slice::from_raw_parts(data.as_ptr().cast_const().cast(), self.len as _) }
    }
}

impl AsMut<[u8]> for RustBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        let Some(data) = self.data else {
            return &mut [];
        };

        unsafe { core::slice::from_raw_parts_mut(data.as_ptr().cast(), self.len as _) }
    }
}

#[cfg(feature = "std")]
impl std::io::Write for RustBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.len + (buf.len() as u32) > self.capacity {
            self.reserve(buf.len() as u32);
        }

        let current_len = self.len as usize;
        let pointer = self.data.expect("buffer should not be empty");

        for (i, byte) in buf.iter().copied().enumerate() {
            unsafe { pointer.add(current_len + i).write(byte as _) };
        }

        self.len = (current_len + buf.len()) as u32;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
