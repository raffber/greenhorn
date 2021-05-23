use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use futures::Future;
    use std::convert::TryInto;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;

    #[wasm_bindgen]
    extern "C" {
        fn greenhorn_set_timeout(fun: JsValue, timeout: i32);
    }

    pub fn spawn<F, T>(future: F)
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        spawn_local(async move {
            future.await;
        });
    }

    pub fn spawn_blocking<F, T>(future: F)
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        spawn(future);
    }

    pub fn set_timeout<F: 'static + FnOnce()>(fun: F, wait_time_ms: u64) {
        let fun = Closure::once_into_js(move || fun());
        let wait_time_ms: i32 = wait_time_ms.try_into().unwrap();
        greenhorn_set_timeout(fun, wait_time_ms);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod default {
    use tokio::{task, time};
    use tokio::time::Duration;
    use futures::Future;

    pub fn spawn<F, T>(future: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn(future);
    }

    pub fn set_timeout<F: 'static + Send + FnOnce()>(fun: F, wait_time_ms: u64) {
        spawn(async move {
            time::sleep(Duration::from_millis(wait_time_ms)).await;
            fun();
        });
    }
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub use wasm::{spawn, spawn_blocking, set_timeout};
    } else {
        pub use default::{spawn, set_timeout};
    }
}
