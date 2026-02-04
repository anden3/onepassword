use byteorder::{BigEndian, ByteOrder};

use crate::buffer::RustBuffer;

pub type FfiResult<T = (), E = Error> = Result<T, E>;

#[derive(Debug)]
pub enum Error {
    Error { code: i32, message: String },
}

impl Error {
    pub fn code(&self) -> i32 {
        match self {
            Self::Error { code, .. } => *code,
        }
    }
}

#[repr(u8)]
#[expect(dead_code)]
pub(crate) enum CallStatusCode {
    Success = 0,
    Error = 1,
    Panic = 2,
}

#[repr(C)]
pub(crate) struct CallStatus {
    pub code: CallStatusCode,
    pub error_buf: RustBuffer,
}

pub(crate) trait ErrorConverter {
    type ErrorType;

    fn lift(buf: RustBuffer) -> Self::ErrorType;
}

pub(crate) struct NoConverter;
impl ErrorConverter for NoConverter {
    type ErrorType = std::convert::Infallible;

    fn lift(_buf: RustBuffer) -> Self::ErrorType {
        panic!("_rust_call_with_error: CALL_ERROR, but error_ffi_converter is None");
    }
}

pub(crate) struct StringConverter;
impl ErrorConverter for StringConverter {
    type ErrorType = String;

    fn lift(buf: RustBuffer) -> Self::ErrorType {
        buf.to_string()
    }
}

pub(crate) struct ErrorTypeConverter;
impl ErrorConverter for ErrorTypeConverter {
    type ErrorType = Error;

    fn lift(buf: RustBuffer) -> Self::ErrorType {
        let error_variant = BigEndian::read_i32(buf.as_ref());
        let error_code = BigEndian::read_i32(&buf.as_ref()[4..]);

        match error_variant {
            1 => {
                let remainder = &buf.as_ref()[8..];
                let msg = String::from_utf8_lossy(remainder);
                Error::Error {
                    code: error_code,
                    message: msg.into_owned(),
                }
            }
            other => unimplemented!("unknown error variant {other}"),
        }
    }
}

pub(crate) fn check_call_status<C: ErrorConverter>(
    call_status: CallStatus,
) -> Result<(), C::ErrorType> {
    match call_status.code {
        CallStatusCode::Success => Ok(()),
        CallStatusCode::Error => Err(C::lift(call_status.error_buf)),
        CallStatusCode::Panic => {
            let msg = if call_status.error_buf.len > 0 {
                StringConverter::lift(call_status.error_buf)
            } else {
                "Unknown rust panic".to_string()
            };
            panic!("{msg}");
        }
    }
}
