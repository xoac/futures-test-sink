//! This crate provide a handy mock sink implementations that can be used test own Sink.
//!
//! # Examples
//!
//! ## `SinkMock` allow to create a handy tests
//! This example contains a 3 tests. See documentation of `SinkMock` for details.
//! ```
//! use futures::{
//!     self,
//!     never::Never,
//!     stream::{self, StreamExt},
//! };
//! use futures_test_sink::SinkMock;
//! use std::iter;
//! use std::task::{Context, Poll};
//!
//! fn drain_test() {
//!     let e = iter::repeat::<Poll<Result<(), Never>>>(Poll::Ready(Ok(())));
//!     let sink = SinkMock::with_flush_feedback(e);
//!
//!     let stream =
//!         stream::iter(vec![Ok::<u8, Never>(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
//!     let send_all = stream.forward(sink);
//!     assert_eq!(Ok(()), futures::executor::block_on(send_all));
//! }
//!
//! fn interleave_pending() {
//!     let e = vec![Poll::Ready(Ok::<_, Never>(())), Poll::Pending]
//!         .into_iter()
//!         .cycle();
//!     let sink = SinkMock::with_flush_feedback(e);
//!
//!     let stream =
//!         stream::iter(vec![Ok::<u8, Never>(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
//!     let send_all = stream.forward(sink);
//!     assert_eq!(Ok(()), futures::executor::block_on(send_all));
//! }
//!
//! fn error() {
//!     let e = vec![Poll::Ready(Ok(())), Poll::Pending, Poll::Ready(Err(()))]
//!         .into_iter()
//!         .cycle();
//!     let sink = SinkMock::with_flush_feedback(e);
//!
//!     let stream = stream::iter(vec![Ok(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
//!     let send_all = stream.forward(sink);
//!     assert_eq!(Err(()), futures::executor::block_on(send_all));
//! }
//!
//! drain_test();
//! interleave_pending();
//! error();
//! ```
//!
//! ## `SinkFeedback` mock provide a full control of returned items.
//!
//! You should first use `SinkMock` if this doesn't this one may be useful.
//!
//! ```
//! use async_task::waker_fn;
//! use futures::sink::Sink;
//! use futures_test_sink::from_iter;
//! use std::{
//!     pin::Pin,
//!     sync::{atomic, Arc},
//!     task::{Context, Poll},
//! };
//!
//! // create a Context
//! let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
//! let cnt = wake_cnt.clone();
//! let waker = waker_fn(move || {
//!     wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
//! });
//! let mut cx = Context::from_waker(&waker);
//! // actual test
//! let poll_fallback = vec![
//!     Poll::Ready(Ok(())),
//!     Poll::Ready(Ok(())),
//!     Poll::Pending,
//!     Poll::Ready(Err(12)),
//! ]
//! .into_iter();
//! let start_send_fallback = vec![Ok::<_, u32>(())].into_iter().cycle();
//! // ours sink implementation
//! let mut s = from_iter(poll_fallback, start_send_fallback);
//!
//! let r1 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r1, Poll::Ready(Ok(())));
//! let s1 = Pin::new(&mut s).start_send(1);
//! assert_eq!(s1, Ok(()));
//!
//! let r2 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r2, Poll::Ready(Ok(())));
//! // start send don't panic because start_send_fallback is cycle
//! let s2 = Pin::new(&mut s).start_send(2);
//! assert_eq!(s2, Ok(()));
//!
//! // ctx.wake() wasn't called.
//! assert_eq!(0, cnt.load(atomic::Ordering::SeqCst));
//!
//! let r3 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r3, Poll::Pending);
//! assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));
//!
//! let r4 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r4, Poll::Ready(Err(12)));
//! assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));
//! ```
//!
//! You can be interested in [FuseLast](fuse_last::FuseLast) container for Iterator.
//!
//!
//! ```
//! use async_task::waker_fn;
//! use futures::sink::Sink;
//! use futures_test_sink::{from_iter, fuse_last::IteratorExt};
//! use std::{
//!     pin::Pin,
//!     sync::{atomic, Arc},
//!     task::{Context, Poll},
//! };
//!
//! // create a Context
//! let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
//! let cnt = wake_cnt.clone();
//! let waker = waker_fn(move || {
//!     wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
//! });
//! let mut cx = Context::from_waker(&waker);
//! // actual test
//! let poll_fallback = vec![
//!     Poll::Ready(Ok(())),
//!     Poll::Ready(Err(12)),
//!     Poll::Ready(Ok(())),
//! ]
//! .into_iter()
//! .fuse_last();
//! let start_send_fallback = vec![Ok::<_, u32>(())].into_iter().cycle();
//! // ours sink implementation
//! let mut s = from_iter(poll_fallback, start_send_fallback);
//!
//! let r1 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r1, Poll::Ready(Ok(())));
//! let s1 = Pin::new(&mut s).start_send(1);
//! assert_eq!(s1, Ok(()));
//!
//! let r2 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r2, Poll::Ready(Err(12)));
//!
//! let r3 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r3, Poll::Ready(Ok(())));
//!
//! // if not `fuse_last` this would panic!
//! let r4 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r3, Poll::Ready(Ok(())));
//!
//! let r5 = Pin::new(&mut s).poll_ready(&mut cx);
//! assert_eq!(r3, Poll::Ready(Ok(())));
//! ```

#![deny(missing_docs)]

pub mod fuse_last;
mod mock_sink;

pub use mock_sink::SinkMock;

use futures::never::Never;
use futures::sink::Sink;
use std::iter::{repeat, successors, Repeat};
use std::marker::PhantomData;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

fn reverse<E>(poll: &Poll<Result<(), E>>) -> Option<Poll<Result<(), E>>> {
    match poll {
        Poll::Pending => Some(Poll::Ready(Ok(()))),
        Poll::Ready(_) => Some(Poll::Pending),
    }
}

/// This `SinkFeedback` will discard every item send to it and returned mocked feedback.
///
/// For details see [from_iter()].
///
/// [from_iter]:from_iter
pub struct SinkFeedback<E, FI, SSI, Item> {
    poll_fallback: FI,
    start_send_fallback: SSI,
    item_type: PhantomData<Item>,
    err_typpe: PhantomData<E>,
}

type Drain<Item> =
    SinkFeedback<Never, Repeat<Poll<Result<(), Never>>>, Repeat<Result<(), Never>>, Item>;

/// This method is similar to [`drain()`](futures::sink::drain) from futures crate.
pub fn ok<Item>() -> Drain<Item> {
    Drain {
        poll_fallback: repeat(Poll::Ready(Ok(()))),
        start_send_fallback: repeat(Ok(())),
        item_type: Default::default(),
        err_typpe: Default::default(),
    }
}

/// This method will additionally return `Poll::Pending` every second poll call.
///
/// Inspirited by
/// [InterleavePending](https://docs.rs/futures-test/0.3.3/futures_test/stream/struct.InterleavePending.html) from futures-test crate.
pub fn interleave_pending<Item>() -> impl Sink<Item, Error = Never>
where
    Item: Unpin,
{
    let poll_fallback = successors(Some(Poll::Ready(Ok(()))), reverse);
    let ss_value: Result<(), Never> = Ok(());
    let start_send_fallback = repeat(ss_value);

    from_iter(poll_fallback, start_send_fallback)
}

/// This method allows to create Sink from iterators.
///
/// Any time you call `poll_ready`, `poll_flush` or `push_close` the [next] method will be called on `poll_fallback` iterator.
/// If iterator return `Poll::Pending` the `cx.waker().clone().wake()` will be additionally called.
/// Where `cx` is `std::task::Context` passed to `poll_ready`, `poll_flush` or `poll_close` function.
///
/// Any time you call `start_send` the inner implementation will discard `item` and return
/// unwrapped item that `start_send_fallback` iterator return.
///
/// # Panics
///
/// If `poll_fallback` or `start_send_fallback` iterator has no more elements. To prevent this use
/// [cycle] method.
///
/// [cycle]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.cycle
/// [next]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#tymethod.next
pub fn from_iter<Item, FI, SSI, E>(
    poll_fallback: FI,
    start_send_fallback: SSI,
) -> impl Sink<Item, Error = E>
where
    FI: Iterator<Item = Poll<Result<(), E>>> + Unpin,
    SSI: Iterator<Item = Result<(), E>> + Unpin,
    E: Unpin,
    Item: Unpin,
{
    SinkFeedback {
        poll_fallback,
        start_send_fallback,
        item_type: Default::default(),
        err_typpe: Default::default(),
    }
}

impl<E, FI, SSI, Item> Sink<Item> for SinkFeedback<E, FI, SSI, Item>
where
    Self: Sized + Unpin,
    FI: Iterator<Item = Poll<Result<(), E>>>,
    SSI: Iterator<Item = Result<(), E>>,
{
    type Error = E;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = Pin::into_inner(self);
        match this.poll_fallback.next().unwrap() {
            Poll::Ready(t) => Poll::Ready(t),
            Poll::Pending => {
                cx.waker().clone().wake();
                Poll::Pending
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, _item: Item) -> Result<(), Self::Error> {
        Pin::into_inner(self).start_send_fallback.next().unwrap()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_task::waker_fn;
    use std::sync::{atomic, Arc};

    #[test]
    fn test_ok() {
        // create a Context
        let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
        let cnt = wake_cnt.clone();
        let waker = waker_fn(move || {
            wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
        });
        let mut cx = Context::from_waker(&waker);
        // actual test
        let mut d = super::ok();
        let r1 = Pin::new(&mut d).poll_ready(&mut cx);
        let s1 = Pin::new(&mut d).start_send(1);
        assert_eq!(r1, Poll::Ready(Ok(())));
        assert_eq!(s1, Ok(()));
        assert_eq!(0, cnt.load(atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_interleave_pending() {
        // create a Context
        let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
        let cnt = wake_cnt.clone();
        let waker = waker_fn(move || {
            wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
        });
        let mut cx = Context::from_waker(&waker);
        // actual test
        let mut s = interleave_pending();
        let r1 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r1, Poll::Ready(Ok(())));
        for v in 5..140 {
            let r_s = Pin::new(&mut s).start_send(v);
            assert_eq!(r_s, Ok(()));
        }
        assert_eq!(0, cnt.load(atomic::Ordering::SeqCst));

        let r2 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r2, Poll::Pending);
        assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));

        let r3 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r3, Poll::Ready(Ok(())));
        assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_from_iter() {
        // create a Context
        let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
        let cnt = wake_cnt.clone();
        let waker = waker_fn(move || {
            wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
        });
        let mut cx = Context::from_waker(&waker);
        // actual test
        let poll_fallback = vec![
            Poll::Ready(Ok(())),
            Poll::Ready(Ok(())),
            Poll::Pending,
            Poll::Ready(Err(12)),
        ]
        .into_iter();
        let start_send_fallback = vec![Ok::<_, u32>(())].into_iter().cycle();
        let mut s = from_iter(poll_fallback, start_send_fallback);

        let r1 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r1, Poll::Ready(Ok(())));
        let s1 = Pin::new(&mut s).start_send(1);
        assert_eq!(s1, Ok(()));

        let r2 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r2, Poll::Ready(Ok(())));
        // start send don't panic because start_send_fallback is cycle
        let s2 = Pin::new(&mut s).start_send(2);
        assert_eq!(s2, Ok(()));

        // ctx.wake() wasn't called.
        assert_eq!(0, cnt.load(atomic::Ordering::SeqCst));

        let r3 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r3, Poll::Pending);
        assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));

        let r4 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r4, Poll::Ready(Err(12)));
        assert_eq!(1, cnt.load(atomic::Ordering::SeqCst));
    }

    #[test]
    #[should_panic]
    fn test_panic_on_iter_end() {
        // create a Context
        let wake_cnt = Arc::new(atomic::AtomicUsize::new(0));
        let waker = waker_fn(move || {
            wake_cnt.fetch_add(1, atomic::Ordering::SeqCst);
        });
        let mut cx = Context::from_waker(&waker);
        // actual test
        let poll_fallback = vec![Poll::Ready(Ok(()))].into_iter();
        let start_send_fallback = vec![Ok::<_, u32>(())].into_iter().cycle();
        let mut s = from_iter(poll_fallback, start_send_fallback);

        let r1 = Pin::new(&mut s).poll_ready(&mut cx);
        assert_eq!(r1, Poll::Ready(Ok(())));
        let s1 = Pin::new(&mut s).start_send(1);
        assert_eq!(s1, Ok(()));
        // now it should panic
        let _ = Pin::new(&mut s).poll_ready(&mut cx);
    }
}
