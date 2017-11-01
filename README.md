future_pubsub
===

![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)
![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/tungstenite.svg?maxAge=2592000)](https://crates.io/crates/future_pubsub)

A tokio future based publish-subscribe channel.

For now, this crate provides
- unsync unbounded publish subscribe channel
- unsync unbounded cloneable stream


And will provide in future
- unsync bounded publish subscribe channel
- sync unbounded publish subscribe channel
- sync bounded publish subscribe channel
- unsync bounded cloneable stream
- sync unbounded cloneable stream
- sync bounded cloneable stream



## How to use
### Publish-Subscribe channel
An usage is almost same with `futures::unsync::mpsc::unbounded`.


```rust
use future_pubsub::unsync::unbounded;

fn main() {
    let (tx, rx) = unbounded::<usize>();
    let rx2 = rx.clone();
    let mut rx = rx.wait();
    let mut rx2 = rx.wait();

    tx.send(1).wait().unwrap();

    assert_eq!(rx.next().unwrap().map(|i| *i), Ok(1));
    assert_eq!(rx2.next().unwrap().map(|i| *i), Ok(1));
}
```


### Cloneable stream

```rust
use future_pubsub::unsync::into_cloneable;

fn main() {
    let stream = gen_inc_stream();;

    let cloneable = into_cloneable(stream);
    let cloneable2 = cloneable.clone();

    assert_eq!(cloneable.map(|i| *i).collect().wait().unwrap(), [0, 1, 2, 3]);
    assert_eq!(cloneable2.map(|i| *i).collect().wait().unwrap(), [0, 1, 2, 3]);
}
