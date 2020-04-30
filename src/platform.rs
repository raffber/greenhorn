use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use futures::Future;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use std::convert::TryInto;
    use web_sys::window;

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
        fun();
        // let window = window().unwrap();
        // let wait_time_ms: i32 = wait_time_ms.try_into().unwrap();
        // let fun = Closure::once(move || fun());
        // window.set_timeout_with_callback_and_timeout_and_arguments_0(fun.as_ref().unchecked_ref(), wait_time_ms).unwrap();
        // fun.forget();
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod default {
    use async_std::task;
    use futures::Future;
    use async_timer::Interval;

    pub fn spawn<F, T>(future: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn(future);
    }

    pub fn spawn_blocking<F, T>(future: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn_blocking(|| {
            task::block_on(async move {
                let _ = future.await;
            })
        });
    }

    pub fn set_timeout<F: 'static + Send + FnOnce()>(fun: F, wait_time_ms: u64) {
        spawn(async move {
            let mut timer = Interval::platform_new(core::time::Duration::from_millis(wait_time_ms));
            timer.as_mut().await;
            timer.cancel();
            fun();
        });
    }
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub use wasm::{spawn, spawn_blocking, set_timeout};
    } else {
        pub use default::{spawn, spawn_blocking, set_timeout};
    }
}
