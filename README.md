[![crates.io](https://img.shields.io/crates/v/futures-test-sink.svg)](https://crates.io/crates/futures-test-sink)
[![Documentation](https://docs.rs/futures-test-sink/badge.svg)](https://docs.rs/futures-test-sink/)
![CI master](https://github.com/xoac/futures-test-sink/workflows/Continuous%20integration/badge.svg?branch=master)


# futures-test-sink

This crate provide a handy mock sink implementations that can be used test own Sink.

## Examples

```rust
use futures_test_sink::from_iter;
use async_task::waker_fn;
use futures::sink::Sink;
use std::{
  pin::Pin,
  task::{Poll, Context},
  sync::{Arc, atomic}
  };

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
// ours sink implementation
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
```

You can be interested in [FuseLast] container for Iterator.
```rust
use futures_test_sink::{from_iter, fuse_last::IteratorExt};
use async_task::waker_fn;
use futures::sink::Sink;
use std::{
  pin::Pin,
  task::{Poll, Context},
  sync::{Arc, atomic}
  };

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
    Poll::Ready(Err(12)),
    Poll::Ready(Ok(())),
]
.into_iter().fuse_last();
let start_send_fallback = vec![Ok::<_, u32>(())].into_iter().cycle();
// ours sink implementation
let mut s = from_iter(poll_fallback, start_send_fallback);

let r1 = Pin::new(&mut s).poll_ready(&mut cx);
assert_eq!(r1, Poll::Ready(Ok(())));
let s1 = Pin::new(&mut s).start_send(1);
assert_eq!(s1, Ok(()));

let r2 = Pin::new(&mut s).poll_ready(&mut cx);
assert_eq!(r2, Poll::Ready(Err(12)));

let r3 = Pin::new(&mut s).poll_ready(&mut cx);
assert_eq!(r3, Poll::Ready(Ok(())));

// if not `fuse_last` this would panic!
let r4 = Pin::new(&mut s).poll_ready(&mut cx);
assert_eq!(r3, Poll::Ready(Ok(())));

let r5 = Pin::new(&mut s).poll_ready(&mut cx);
assert_eq!(r3, Poll::Ready(Ok(())));
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

This project try follow rules:
* [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
* [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

_This README was generated with [cargo-readme](https://github.com/livioribeiro/cargo-readme) from [template](https://github.com/xoac/crates-io-lib-template)_
