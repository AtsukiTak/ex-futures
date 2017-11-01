future_pubsub
===

![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)
![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/tungstenite.svg?maxAge=2592000)](https://crates.io/crates/future_pubsub)

A tokio future based publish-subscribe channel.

For now, this crate provides
- unsync unbounded channel


And does not provide
- unsync bounded channel
- sync unbounded channel
- sync bounded channel



## How to use
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
