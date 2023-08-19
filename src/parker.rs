use crate::*;
use std::task::Waker;
use std::sync::Mutex;

struct InnerParker {
    waker: Option<Waker>,
    ready: bool,
}

impl InnerParker {
    pub fn new() -> Self {
        Self { waker: None, ready: false }
    }
}

pub struct Nudger(Arc<Mutex<InnerParker>>);

impl Nudger {
    pub fn nudge(&mut self) {
        if let Ok(mut inner) = self.0.lock() {
            inner.ready = true;
            if let Some(w) = inner.waker.take() {
                w.wake();
            }
        }
    }
}

pub struct Parker(Arc<Mutex<InnerParker>>);

impl Parker {
    pub fn new() -> (Self, Nudger) {
        let inner = Arc::new(Mutex::new(InnerParker {
            waker: None,
            ready: false,
        }));
        (Self(inner.clone()), Nudger(inner))
    }
}

impl Future for Parker {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Ok(mut inner) = self.0.lock() {
            inner.waker = Some(cx.waker().clone());
            if inner.ready { 
                inner.ready = false; 
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

impl Stream for Parker {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Ok(mut x) = self.0.lock() {
            x.waker = Some(cx.waker().clone());
            if x.ready { 
                x.ready = false; 
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}
