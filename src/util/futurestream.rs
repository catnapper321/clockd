use async_std::{
    pin::Pin,
    future::Future,
    stream::Stream,
};
use std::task::Poll;

/// A wrapper type that adapts a Future to the Stream interface.
/// After the future completes, polls will return Poll::Ready(Some(value)),
/// then Poll::Ready(None). Subsequent polls return Poll::Pending.
pub struct FutureStream<T> {
    fut: Option<Pin<Box<dyn Future<Output = T>>>>,
    finished: bool,
}

impl<T> FutureStream<T> {
    /// Wraps the future in a FutureStream.
    pub fn new(fut: impl Future<Output = T> + 'static) -> Self {
        // allow passing an already pinned, boxed future.
        // the _ is a workaround, cannot use impl type in annotations
        let x: Pin<Box<_>> = Box::pin(fut).into();
        Self { fut: Some(x), finished: false, }
    }
    /// Creates a FutureStream without wrapping a future.
    /// It returns Poll::Pending when polled.
    /// Use the set method to wrap a future.
    pub fn new_never() -> Self {
        Self { fut: None, finished: true }
    }
    /// Discard the wrapped future, and return Poll::Pending when polled.
    pub fn set_never(&mut self) {
        self.fut = None;
        self.finished = true;
    }
    /// Replace the wrapped future with a new one.
    pub fn set(&mut self, fut: impl Future<Output = T> + 'static) {
        let x: Pin<Box<_>> = Box::pin(fut).into();
        self.fut = Some(x);
        self.finished = false;
    }
    /// Wrap this future unless a future is already wrapped.
    pub fn set_if_no_future(&mut self, fut: impl Future<Output = T> + 'static) {
        if self.fut.is_none() {
            let x: Pin<Box<_>> = Box::pin(fut).into();
            self.fut = Some(x);
            self.finished = false;
        }
    }
    /// Returns true if currently wrapping a future
    pub fn may_finish(&self) -> bool {
        self.fut.is_some()
    }
}

impl<T> Stream for FutureStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        if self.finished { return Poll::Pending }
        if let Some(ref mut fut) = self.fut {
            let x = fut.as_mut().poll(cx);
            match x {
                std::task::Poll::Ready(v) => {
                    self.fut = None;
                    Poll::Ready(Some(v))
                },
                std::task::Poll::Pending => Poll::Pending,
            }
        } else {
            self.finished = true;
            Poll::Ready(None)
        }
    }
}
