futures_ext
===

> I really want your contribution especially English staff.
> As you see, my English skill is not good. Please help me.

![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)
![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/ex-futures.svg)](https://crates.io/crates/ex-futures)

An extension of `futures`.

[Document](https://docs.rs/ex-futures)


For now, this crate provides
- publish subscribe channel (will be removed future)
- cloneable stream/sink
- stream fork


## How to use
### Publish-Subscribe channel
An usage is almost same with `futures::unsync::mpsc::unbounded`.


```rust
use ex_futures::unsync::pubsub::unbounded;

fn main() {
    let (tx, rx) = unbounded::<usize>();
    let rx2 = rx.clone();
    let mut rx = rx.wait();
    let mut rx2 = rx.wait(); // Subscriber is cloneable

    tx.send(1).wait().unwrap();

    assert_eq!(rx.next().unwrap().map(|i| *i), Ok(1));
    assert_eq!(rx2.next().unwrap().map(|i| *i), Ok(1));
}
```


### Cloneable stream/sink

```rust
use ex_futures::StreamExt;
use futures::unsycn::mpsc::channel;

fn main() {
    let (tx, rx) = channel(42);;

    let cloneable_rx = rx.unsync_cloneable(); // Convert "rx" into cloneable
    let cloneable_rx2 = cloneable.clone();  // Now you can clone it

    let tx = tx.wait();
    tx.send(0);
    tx.send(1);
    tx.send(2);
    tx.send(3);

    assert_eq!(cloneable_rx.collect().wait().unwrap(), [0, 1, 2, 3]);
    assert_eq!(cloneable_rx2.collect().wait().unwrap(), [0, 1, 2, 3]);
}
```


### Forked stream

```rust
use ex_futures::StreamExt;

fn main() {
    let int_stream = gen_stream(); // Somehow you create some stream

    let (even, odd) = int_stream.fork(|i| i % 2 == 0);

    assert_eq!(even.collect().wait().unwrap(), [0, 2]);
    assert_eq!(odd.collect().wait().unwrap(), [1, 3]);
}
```

