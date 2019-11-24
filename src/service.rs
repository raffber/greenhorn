use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use std::future::Future;

use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

type BoxedTask = Box<dyn Future<Output = ()>>;

pub trait ServiceProcess<T>: Stream<Item = T> {
    fn stop(self);
}

pub trait Service {
    type Data: 'static;
    type Output: ServiceProcess<Self::Data> + Unpin + 'static;

    fn start(self) -> Self::Output;
}

pub struct ServiceSubscription<T: 'static> {
    inner: Box<dyn ServiceProcessMap<T>>,
}

impl<T: 'static> ServiceSubscription<T> {
    pub fn start<S: Service, Mapper: 'static + Fn(S::Data) -> T>(
        service: S,
        mapper: Mapper,
    ) -> Self {
        let process = service.start();
        let ret = ServiceProcessMapDirect {
            process,
            mapper,
            phantom_data: PhantomData,
            phantom_t: PhantomData,
        };
        ServiceSubscription {
            inner: Box::new(ret),
        }
    }

    pub fn map<U: 'static, Mapper: 'static + Fn(T) -> U>(
        self,
        mapper: Arc<Mapper>,
    ) -> ServiceSubscription<U> {
        let ret = ServiceProcessMapped {
            inner: self.inner,
            mapper,
        };
        ServiceSubscription {
            inner: Box::new(ret),
        }
    }
}

impl<T: 'static> Stream for ServiceSubscription<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

trait ServiceProcessMap<T> {
    fn stop(self: Box<Self>);
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>>;
    fn size_hint(&self) -> (usize, Option<usize>);
}

struct ServiceProcessMapDirect<
    Data,
    T,
    Mapper: Fn(Data) -> T,
    Process: ServiceProcess<Data> + Unpin,
> {
    process: Process,
    mapper: Mapper,
    phantom_data: std::marker::PhantomData<Data>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<Data, T, Mapper, Process> ServiceProcessMap<T>
    for ServiceProcessMapDirect<Data, T, Mapper, Process>
where
    Mapper: Fn(Data) -> T,
    Process: ServiceProcess<Data> + Unpin,
{
    fn stop(self: Box<Self>) {
        self.process.stop();
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let ret = self.process.poll_next_unpin(cx);
        match ret {
            Poll::Ready(Some(data)) => Poll::Ready(Some((self.mapper)(data))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.process.size_hint()
    }
}

struct ServiceProcessMapped<T, U, Mapper: Fn(T) -> U> {
    inner: Box<dyn ServiceProcessMap<T>>,
    mapper: Arc<Mapper>,
}

impl<T, U, Mapper: Fn(T) -> U> ServiceProcessMap<U> for ServiceProcessMapped<T, U, Mapper> {
    fn stop(self: Box<Self>) {
        self.inner.stop();
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<U>> {
        let ret = self.inner.poll_next(cx);
        match ret {
            Poll::Ready(Some(data)) => Poll::Ready(Some((self.mapper)(data))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
