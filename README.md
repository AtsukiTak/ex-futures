future_pubsub
===

A tokio future based publish-subscribe channel.

For now, this crate provides
- unsync unbounded channel


And does not provide
- unsync bounded channel
- sync unbounded channel
- sync bounded channel



## How to use
An usage is almost same with `futures::unsync::mpsc::unbounded`.

***CAUTION : Every item being passed in need to implement `Clone` trait***


```
use future_pubsub::unsync::unbounded;

fn main() {
    let (tx, rx) = unbounded::<usize>();
    let rx2 = rx.clone();
    let mut rx = rx.wait();
    let mut rx2 = rx.wait();

    tx.send(1).wait().unwrap();

    assert_eq!(rx.next().unwrap(), Ok(1));
    assert_eq!(rx2.next().unwrap(), Ok(1));
}
