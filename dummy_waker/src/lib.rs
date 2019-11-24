// largely copied from:
// https://github.com/TomCrypto/core-futures-stateless.git

use core::ptr::null;
use std::task::{Waker, RawWaker, RawWakerVTable, Context};


pub fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(DUMMY_RAW_WAKER) }
}

const DUMMY_RAW_WAKER: RawWaker = RawWaker::new(null(), DUMMY_VTABLE);

fn dummy_waker_clone(_: *const ()) -> RawWaker {
    DUMMY_RAW_WAKER
}

fn dummy_waker_no_op(_: *const ()) {
    // we cannot do anything here
}

const DUMMY_VTABLE: &RawWakerVTable = &RawWakerVTable::new(
    dummy_waker_clone,
    dummy_waker_no_op,
    dummy_waker_no_op,
    dummy_waker_no_op,
);

pub struct DummyContext {
    waker: Waker,
}

impl DummyContext {
    pub fn context(&self) -> Context {
        Context::from_waker(&self.waker)
    }
}

pub fn dummy_context() -> DummyContext {
    DummyContext {
        waker: dummy_waker(),
    }
}