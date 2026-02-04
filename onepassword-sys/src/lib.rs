use crate::{
    errors::{CallStatus, ErrorTypeConverter, FfiResult},
    util::rust_call,
};

mod buffer;
mod errors;
mod futures;
mod util;

pub use {buffer::RustBuffer, errors::Error, util::validate_checksums, util::version};

#[link(name = "op_uniffi_core", kind = "raw-dylib")]
unsafe extern "C" {
    #[link_name = "uniffi_op_uniffi_core_fn_func_init_client"]
    unsafe fn uniffi_init_client(buffer: RustBuffer) -> futures::FfiFutureHandle<RustBuffer>;
    #[link_name = "uniffi_op_uniffi_core_fn_func_release_client"]
    unsafe fn uniffi_release_client(buffer: RustBuffer, status: *mut CallStatus);
}

#[cfg(feature = "sync")]
#[link(name = "op_uniffi_core", kind = "raw-dylib")]
unsafe extern "C" {
    #[link_name = "uniffi_op_uniffi_core_fn_func_invoke_sync"]
    unsafe fn uniffi_invoke_sync(buffer: RustBuffer, status: *mut CallStatus) -> RustBuffer;
}

#[cfg(feature = "async")]
#[link(name = "op_uniffi_core", kind = "raw-dylib")]
unsafe extern "C" {
    #[link_name = "uniffi_op_uniffi_core_fn_func_invoke"]
    unsafe fn uniffi_invoke(buffer: RustBuffer) -> futures::FfiFutureHandle<RustBuffer>;
}

#[cfg(feature = "async")]
pub async fn invoke(payload: &str) -> Result<RustBuffer, Error> {
    let buffer: RustBuffer = payload.into();

    let result = unsafe { uniffi_invoke(buffer) }
        .into_future::<ErrorTypeConverter>()
        .await?;

    Ok(result)
}

#[cfg(feature = "sync")]
pub fn invoke_sync(payload: &str) -> Result<RustBuffer, Error> {
    let buffer: RustBuffer = payload.into();

    unsafe { rust_call!(uniffi_invoke_sync -> ErrorTypeConverter, buffer) }
}

#[cfg(feature = "async")]
pub async fn get_client_id_buffer(client_config: &str) -> FfiResult<RustBuffer> {
    let buffer: RustBuffer = client_config.into();

    unsafe { uniffi_init_client(buffer) }
        .into_future::<ErrorTypeConverter>()
        .await
}

#[cfg(feature = "sync")]
pub fn get_client_id_buffer_sync(client_config: &str) -> FfiResult<RustBuffer> {
    let buffer: RustBuffer = client_config.into();

    pollster::block_on(unsafe { uniffi_init_client(buffer) }.into_future::<ErrorTypeConverter>())
}

pub fn free_client(client_id: &str) {
    let buffer = RustBuffer::from(client_id);
    match unsafe { rust_call!(uniffi_release_client, buffer) } {
        Ok(()) => {}
    }
}
