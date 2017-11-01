extern crate future_pubsub;
extern crate futures;
extern crate tokio_core;

use future_pubsub::unsync::unbounded;

use futures::{Future, Stream, Sink};
use futures::stream::unfold;
use futures::future::ok;

use tokio_core::reactor::Core;

use std::ops::Deref;



#[test]
fn send_recv() {
    let (tx, rx) = unbounded::<usize>();
    let mut rx = rx.wait();

    tx.send(1).wait().unwrap();

    assert_eq!(rx.next().unwrap().unwrap().deref(), &1);
}



#[test]
fn send_recv_shared() {
    let (tx, rx) = unbounded::<usize>();
    let rx2 = rx.clone();
    let mut rx = rx.wait();
    let mut rx2 = rx2.wait();

    tx.send(1).wait().unwrap();

    assert_eq!(rx.next().unwrap().unwrap().deref(), &1);
    assert_eq!(rx2.next().unwrap().unwrap().deref(), &1);
}


#[test]
fn send_many_items() {
    let mut core = Core::new().unwrap();
    let stream = unfold(0, |i| Some(ok::<_, _>((i, i + 1)))).take(4);

    let (tx, rx) = unbounded::<usize>();

    let future = tx.send_all(stream).map(|_| ()).map_err(|_| ());
    core.handle().spawn(future);

    assert_eq!(core.run(rx.map(|i| *i).collect()).unwrap(), [0, 1, 2, 3]);
}


#[test]
fn send_many_items_recv_shared() {
    let mut core = Core::new().unwrap();
    let stream = unfold(0, |i| Some(ok::<_, _>((i, i + 1)))).take(4);

    let (tx, rx) = unbounded::<usize>();
    let rx2 = rx.clone();
    let rx3 = rx.clone();

    let future = tx.send_all(stream).map(|_| ()).map_err(|_| ());
    core.handle().spawn(future);

    assert_eq!(core.run(rx.map(|i| *i).collect()).unwrap(), [0, 1, 2, 3]);
    assert_eq!(core.run(rx2.map(|i| *i).collect()).unwrap(), [0, 1, 2, 3]);
    assert_eq!(core.run(rx3.map(|i| *i).collect()).unwrap(), [0, 1, 2, 3]);
}
