use core::{
    ffi,
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicU8, Ordering},
    task::Waker,
};
use preinterpret::preinterpret;

use crate::{
    buffer::RustBuffer,
    errors::{CallStatus, ErrorConverter},
    util::rust_call,
};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum FuturePollCode {
    Ready = 0,
    MaybeReady = 1,
}

#[repr(C)]
pub(crate) struct FutureState {
    waker_data: *const (),
    waker_vtable: NonNull<()>,
    poll_code: AtomicU8,
}

type FutureContinuation = extern "C" fn(*mut FutureState, FuturePollCode);

type PollFn<T> = unsafe extern "C" fn(FfiFutureHandle<T>, FutureContinuation, *mut FutureState);
type CancelFn<T> = unsafe extern "C" fn(FfiFutureHandle<T>);
type FreeFn<T> = unsafe extern "C" fn(FfiFutureHandle<T>);
type CompleteFn<T> = unsafe extern "C" fn(FfiFutureHandle<T>, *mut CallStatus) -> T;

pub(crate) trait FfiFutureReturnValue {
    fn poll_fn() -> PollFn<Self>
    where
        Self: core::marker::Sized;
    fn cancel_fn() -> CancelFn<Self>
    where
        Self: core::marker::Sized;
    fn free_fn() -> FreeFn<Self>
    where
        Self: core::marker::Sized;
    fn complete_fn() -> CompleteFn<Self>
    where
        Self: core::marker::Sized;
}

macro_rules! declare_futures {
    ($($kind:ty),+) => {
        preinterpret! {
            #[cfg_attr(windows, link(name = "op_uniffi_core", kind = "raw-dylib"))]
            #[cfg_attr(not(windows), link(name = "op_uniffi_core"))]
            unsafe extern "C" {
                $(
                    #[link_name = [!snake! "ffi_op_uniffi_core_rust_future_poll_" $kind]]
                    unsafe fn [!ident_snake! poll_ffi_future_ $kind](
                        future: FfiFutureHandle<$kind>,
                        continuation: FutureContinuation,
                        state_ptr: *mut FutureState,
                    );
                    #[link_name = [!snake! "ffi_op_uniffi_core_rust_future_cancel_" $kind]]
                    unsafe fn [!ident_snake! cancel_ffi_future_ $kind](future: FfiFutureHandle<$kind>);
                    #[link_name = [!snake! "ffi_op_uniffi_core_rust_future_complete_" $kind]]
                    unsafe fn [!ident_snake! complete_ffi_future_ $kind](
                        future: FfiFutureHandle<$kind>,
                        status: *mut CallStatus,
                    ) -> RustBuffer;
                    #[link_name = [!snake! "ffi_op_uniffi_core_rust_future_free_" $kind]]
                    unsafe fn [!ident_snake! free_ffi_future_ $kind](future: FfiFutureHandle<$kind>);
                ),+
            }

            $(
                impl FfiFutureReturnValue for $kind {
                    fn poll_fn() -> PollFn<Self>
                    where
                        Self: core::marker::Sized {[!ident_snake! poll_ffi_future_ $kind] }
                    fn cancel_fn() -> CancelFn<Self>
                    where
                        Self: core::marker::Sized {[!ident_snake! cancel_ffi_future_ $kind] }
                    fn free_fn() -> FreeFn<Self>
                    where
                        Self: core::marker::Sized {[!ident_snake! free_ffi_future_ $kind] }
                    fn complete_fn() -> CompleteFn<Self>
                    where
                        Self: core::marker::Sized {[!ident_snake! complete_ffi_future_ $kind] }
                }
            ),+
        }
    };
}

declare_futures!(RustBuffer);

#[repr(transparent)]
pub(crate) struct FfiFutureHandle<T: FfiFutureReturnValue>(*mut ffi::c_void, PhantomData<T>);

impl<T: FfiFutureReturnValue> Copy for FfiFutureHandle<T> {}
impl<T: FfiFutureReturnValue> Clone for FfiFutureHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: FfiFutureReturnValue> FfiFutureHandle<T> {
    pub fn into_future<C: ErrorConverter>(self) -> FfiFuture<T, C> {
        FfiFuture {
            future: self,
            waker: None,
            state: FutureState {
                waker_data: core::ptr::dangling(),
                waker_vtable: NonNull::dangling(),
                poll_code: AtomicU8::new(FuturePollCode::MaybeReady as u8),
            },
            is_finished: false,
            converter: PhantomData::<C>,
        }
    }
}

extern "C" fn future_callback(state: *mut FutureState, code: FuturePollCode) {
    let FutureState {
        waker_data,
        waker_vtable,
        poll_code,
    } = unsafe { &*state };
    poll_code.store(code as u8, Ordering::Release);
    unsafe { Waker::new(*waker_data, waker_vtable.cast().as_ref()) }.wake_by_ref();
}

pub(crate) struct FfiFuture<T: FfiFutureReturnValue, C: ErrorConverter> {
    future: FfiFutureHandle<T>,
    waker: Option<Waker>,
    state: FutureState,
    converter: PhantomData<C>,
    is_finished: bool,
}

impl<T: FfiFutureReturnValue + Unpin, C: ErrorConverter + Unpin> Future for FfiFuture<T, C> {
    type Output = Result<T, C::ErrorType>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.is_finished {
            panic!("polling finished future");
        }

        let Self {
            state,
            waker,
            future,
            is_finished,
            ..
        } = self.get_mut();

        let waker = waker.get_or_insert_with(|| cx.waker().clone());
        waker.clone_from(cx.waker());

        state.waker_data = waker.data();
        state.waker_vtable = NonNull::from(waker.vtable()).cast();

        if state.poll_code.load(Ordering::Acquire) == (FuturePollCode::Ready as u8) {
            *is_finished = true;

            let complete_fn = T::complete_fn();
            let output = unsafe { rust_call!(complete_fn -> C, *future) };

            return core::task::Poll::Ready(output);
        }

        unsafe { (T::poll_fn())(*future, future_callback, state) };
        core::task::Poll::Pending
    }
}

impl<T: FfiFutureReturnValue, C: ErrorConverter> Drop for FfiFuture<T, C> {
    fn drop(&mut self) {
        if !self.is_finished {
            unsafe { (T::cancel_fn())(self.future) };
        }

        unsafe { (T::free_fn())(self.future) }
    }
}
