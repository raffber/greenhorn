use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use futures::Future;
    use wasm_bindgen_futures::spawn_local;

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
}

#[cfg(not(target_arch = "wasm32"))]
mod default {
    use async_std::task;
    use futures::Future;

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
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub use wasm::{spawn, spawn_blocking};
    } else {
        pub use default::{spawn, spawn_blocking};
    }
}
