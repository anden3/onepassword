use preinterpret::preinterpret;

#[cfg_attr(windows, link(name = "op_uniffi_core", kind = "raw-dylib"))]
#[cfg_attr(not(windows), link(name = "op_uniffi_core"))]
unsafe extern "C" {
    #[link_name = "ffi_op_uniffi_core_uniffi_contract_version"]
    safe fn uniffi_contract_version() -> u32;
}

macro_rules! rust_call {
    ($fn:ident -> $conv:ty, $($val:expr),*) => {{
        use $crate::errors::{CallStatus, CallStatusCode, check_call_status};

        let mut call_status = CallStatus {
            code: CallStatusCode::Success,
            error_buf: RustBuffer::default(),
        };
        let result = $fn($($val),*, &mut call_status);
        check_call_status::<$conv>(call_status).and(Ok(result))
    }};

    ($fn:ident, $($val:expr),*) => {{
        use $crate::errors::{CallStatus, CallStatusCode, NoConverter, check_call_status};

        let mut call_status = CallStatus {
            code: CallStatusCode::Success,
            error_buf: RustBuffer::default(),
        };
        let result = $fn($($val),*, &mut call_status);
        check_call_status::<NoConverter>(call_status).and(Ok(result))
    }}
}

pub(crate) use rust_call;

macro_rules! link_checksum_fns {
    ($($fn:ident: $checksum:literal),+) => {preinterpret! {
        #[cfg_attr(windows, link(name = "op_uniffi_core", kind = "raw-dylib"))]
        #[cfg_attr(not(windows), link(name = "op_uniffi_core"))]
        unsafe extern "C" {
            $(
                #[link_name = concat!("uniffi_op_uniffi_core_checksum_func_", stringify!($fn))]
                safe fn [!ident_snake! uniffi_checksum_ $fn]() -> u16;
            )+
        }

        static CHECKSUMS: &[(extern "C" fn() -> u16, u16)] = &[
            $(
                ([!ident_snake! uniffi_checksum_ $fn], $checksum)
            ),+
        ];
    }};
}

link_checksum_fns! {
    init_client: 45066,
    release_client: 57155,
    invoke: 29143,
    invoke_sync: 49373
}

pub fn validate_checksums() {
    for (checksum_fn, expected) in CHECKSUMS {
        assert_eq!(checksum_fn(), *expected);
    }
}

pub fn version() -> u32 {
    uniffi_contract_version()
}
