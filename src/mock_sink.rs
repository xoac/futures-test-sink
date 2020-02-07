use futures::{ready, sink::Sink};
use std::iter;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

const DEFAULT_MAX_ITEM: usize = 3usize;
const DEFAULT_FLUSH_AT_ONCE: usize = 2usize;

/// This struct represent correct implementation of sink according to [sink doc].
///
/// # Panics:
///
/// 1. Calling `start_send` without calling '`poll_ready()` with result `Poll::Ready(Ok(()))`' panic!
/// 2. Calling any method after `poll_close()` returned  `Poll::Ready(Ok(()))` once panic!
/// 3. When `flush_feedback` iterator return `None`.
///
/// [sink doc]:https://docs.rs/futures/0.3/futures/sink/trait.Sink.html
pub struct SinkMock<FlushI, ReadyI, SendI, Item> {
    flush_feedback: FlushI,
    ready_fallback: ReadyI,
    send_fallback: SendI,

    //mock inner sink
    max_item: usize,
    item_cnt: usize,
    flush_at_once: usize,
    is_closed: bool,
    can_start_send: bool,

    // marker
    item_type: PhantomData<Item>,
}

impl<FlushI, ReadyI, SendI, Item> Unpin for SinkMock<FlushI, ReadyI, SendI, Item> {}

impl<FlushI, ReadyI, SendI, Item> SinkMock<FlushI, ReadyI, SendI, Item> {
    fn check_panic(&self) {
        if self.is_closed {
            panic!("Trying use closed sink");
        }
    }

    /// Change how many buffered item will be discarded when `flush_feedback` yield
    /// `Poll::Ready(Ok(()))`
    pub fn set_flush_at_once(&mut self, flush_at_once: NonZeroUsize) -> &mut Self {
        self.flush_at_once = flush_at_once.into();
        self
    }

    /// Set how many item can be buffered by this sink before needing to flush.
    pub fn set_max_item(&mut self, max_item: usize) -> &mut Self {
        self.max_item = max_item;
        self
    }
}

impl<FlushI, E, ReadyI, SendI, Item> SinkMock<FlushI, ReadyI, SendI, Item>
where
    FlushI: Iterator<Item = Poll<Result<(), E>>>,
    // ReadyI accept iterator that can only return Error None.
    // This because Poll::Ready(Ok(())) and Poll::Pending can be determined
    // from inner implementation.
    ReadyI: Iterator<Item = E>,
    // Similar reason to ReadyI
    SendI: Iterator<Item = E>,
{
    /// Create a new instance of SinkMock
    ///
    /// # Arguments
    /// - **`flush_feedback`** - an iterator that represent reaction on flushing data to it. Every time
    /// when `poll_flush()` is called the item from iterator is taken and unwrapped and depends on
    /// result an special action is taken:
    ///   - `Poll::Ready(Ok(()))` discard `flush_at_once` items from buffer. Repeat
    ///   - `Poll::Ready(Err(e))` forward error
    ///   - `Poll::Pending` wake up Waker from Context and return Poll::Pending
    ///
    /// - **`send_fallback`** - an iterator that represent when start `start_send()` called with `item: Item` should
    /// return error. If iterator yield `iter_item` `start_send()` returns `Err(iter_item)` discarding `item`.
    /// If iterator return `None` `item` is buffered.
    ///
    /// - **`ready_fallback`** - an iterator that represent when `poll_ready()` should return
    /// error. If iterator yield `iter_item` `poll_ready()` returns `Err(iter_item)`. Otherwise
    /// return `Poll::Ready(Ok(())` if can buffer next item else return result from calling
    /// `poll_flush()`
    ///
    /// - **`max_item`** - how many item this sink can buffer
    ///
    /// - **`flush_at_once`** - how many item will be removed from buffer when `flush_feedback`
    /// return `Poll::Ready(Ok(()))`
    pub fn new(
        flush_feedback: FlushI,
        ready_fallback: ReadyI,
        send_fallback: SendI,
        max_item: usize,
        flush_at_once: usize,
    ) -> Self {
        Self {
            flush_feedback,
            ready_fallback,
            send_fallback,
            max_item,
            item_cnt: 0,
            flush_at_once,
            is_closed: false,
            can_start_send: false,
            item_type: Default::default(),
        }
    }
}

impl<FlushI, E, Item> SinkMock<FlushI, iter::Empty<E>, iter::Empty<E>, Item>
where
    FlushI: Iterator<Item = Poll<Result<(), E>>>,
{
    /// Create a sink that will never return `Error` on `start_send` and `poll_ready`.
    ///
    /// This method will use
    pub fn with_flush_feedback(flush_feedback: FlushI) -> Self {
        SinkMock::new(
            flush_feedback,
            iter::empty(),
            iter::empty(),
            DEFAULT_MAX_ITEM,
            DEFAULT_FLUSH_AT_ONCE,
        )
    }
}

impl<Item, FlushI, ReadyI, SendI, E> Sink<Item> for SinkMock<FlushI, ReadyI, SendI, Item>
where
    FlushI: Iterator<Item = Poll<Result<(), E>>>,
    // ReadyI accept iterator that can only return Error None.
    // This because Poll::Ready(Ok(())) and Poll::Pending can be determined
    // from inner implementation.
    ReadyI: Iterator<Item = E>,
    // Similar reason to ReadyI
    SendI: Iterator<Item = E>,
{
    type Error = E;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.check_panic();
        let mut this = Pin::into_inner(self);
        this.can_start_send = false;
        if let Some(e) = this.ready_fallback.next() {
            return Poll::Ready(Err(e));
        }

        if this.max_item > this.item_cnt {
            this.can_start_send = true;
            Poll::Ready(Ok(()))
        } else {
            match Pin::new(&mut this).poll_flush(cx) {
                Poll::Ready(Ok(())) => {
                    this.can_start_send = true;
                    Poll::Ready(Ok(()))
                }
                forward => forward,
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, _item: Item) -> Result<(), Self::Error> {
        self.check_panic();

        if !self.can_start_send {
            panic!("`start_send()` called without correct call of `poll_ready()`");
        }

        let this = Pin::into_inner(self);
        if let Some(e) = this.send_fallback.next() {
            return Err(e);
        }

        this.item_cnt += 1;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.check_panic();
        let this = Pin::into_inner(self);
        this.can_start_send = false;
        // we can think about it like an I/O that returned it was able to take items.
        // (And how many - `flush_at_once` parameter)
        loop {
            match this
                .flush_feedback
                .next()
                .expect("Unexpected end of `flush_feedback` iterator!")
            {
                // mocked I/O took `flush_at_once` buffered items.
                Poll::Ready(Ok(())) => {
                    this.item_cnt = this.item_cnt.saturating_sub(this.flush_at_once);
                    if this.item_cnt == 0 {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
            }
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.check_panic();
        ready!(self.as_mut().poll_flush(cx))?;
        let this = Pin::into_inner(self);
        this.is_closed = true;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_task::waker_fn;
    use futures::{
        never::Never,
        stream::{self, StreamExt},
    };

    #[test]
    #[should_panic(expected = "`start_send()` called without correct call of `poll_ready()`")]
    fn panic_no_call_poll_ready_test() {
        let e = iter::empty::<Poll<Result<(), Never>>>();
        let mut s = SinkMock::with_flush_feedback(e);
        Pin::new(&mut s).start_send(1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Trying use closed sink")]
    fn panic_after_close() {
        // create a Context
        let waker = waker_fn(move || {});
        let mut cx = Context::from_waker(&waker);
        let e = iter::once::<Poll<Result<(), Never>>>(Poll::Ready(Ok(())));
        let mut s = SinkMock::with_flush_feedback(e);
        let _ = Pin::new(&mut s).poll_close(&mut cx);
        let _ = Pin::new(&mut s).poll_ready(&mut cx);
        // just to inform compiler about Item type.
        let _ = Pin::new(&mut s).start_send(1);
    }

    #[test]
    #[should_panic(expected = "Unexpected end of `flush_feedback` iterator!")]
    fn panic_when_flus_feedback_ends() {
        let waker = waker_fn(move || {});
        let mut cx = Context::from_waker(&waker);
        let e = iter::once::<Poll<Result<(), Never>>>(Poll::Ready(Ok(())));
        let mut s = SinkMock::with_flush_feedback(e);
        let _ = Pin::new(&mut s).poll_flush(&mut cx);
        // this will panic
        let _ = Pin::new(&mut s).poll_flush(&mut cx);
        // just to inform compiler about Item type.
        let _ = Pin::new(&mut s).start_send(1);
    }

    #[test]
    fn drain_test() {
        let e = iter::repeat::<Poll<Result<(), Never>>>(Poll::Ready(Ok(())));
        let sink = SinkMock::with_flush_feedback(e);

        let stream =
            stream::iter(vec![Ok::<u8, Never>(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
        let send_all = stream.forward(sink);
        assert_eq!(Ok(()), futures::executor::block_on(send_all));
    }

    #[test]
    fn interleave_pending() {
        let e = vec![Poll::Ready(Ok::<_, Never>(())), Poll::Pending]
            .into_iter()
            .cycle();
        let sink = SinkMock::with_flush_feedback(e);

        let stream =
            stream::iter(vec![Ok::<u8, Never>(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
        let send_all = stream.forward(sink);
        assert_eq!(Ok(()), futures::executor::block_on(send_all));
    }

    #[test]
    fn error() {
        let e = vec![Poll::Ready(Ok(())), Poll::Pending, Poll::Ready(Err(()))]
            .into_iter()
            .cycle();
        let sink = SinkMock::with_flush_feedback(e);

        let stream = stream::iter(vec![Ok(5u8), Ok(7), Ok(9), Ok(77), Ok(79)].into_iter());
        let send_all = stream.forward(sink);
        assert_eq!(Err(()), futures::executor::block_on(send_all));
    }
}
