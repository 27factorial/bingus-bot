use crossbeam::atomic::AtomicCell;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

pub struct CancelActivity {
    shared: Arc<Shared>,
}

impl CancelActivity {
    pub fn new_pair() -> (Self, ActivityToken) {
        let shared = Arc::new(Shared {
            cancelled: AtomicBool::new(false),
            waker: AtomicCell::new(None),
        });

        let token = ActivityToken {
            shared: Arc::clone(&shared),
        };

        let future = Self { shared };

        (future, token)
    }
}

impl Future for CancelActivity {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.shared.cancelled.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            self.shared.waker.store(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActivityToken {
    shared: Arc<Shared>,
}

impl ActivityToken {
    pub fn cancel(&self) -> bool {
        match self.shared.waker.take() {
            Some(waker) => {
                self.shared.cancelled.store(true, Ordering::Release);
                waker.wake();
                true
            }
            None => false,
        }
    }
}

struct Shared {
    cancelled: AtomicBool,
    waker: AtomicCell<Option<Waker>>,
}

impl fmt::Debug for Shared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Shared { .. }")
    }
}
