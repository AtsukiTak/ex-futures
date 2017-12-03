#![feature(test)]
extern crate test;

extern crate ex_futures;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};

use ex_futures::StreamExt;

use test::Bencher;


#[bench]
fn normal(b: &mut Bencher) {
    let (tx, mut rx) = futures::unsync::mpsc::unbounded();
    b.iter(move || {
        tx.unbounded_send(42).unwrap();
        rx.poll()
    });
}

#[bench]
fn one_consumer(b: &mut Bencher) {
    let (tx, rx) = futures::unsync::mpsc::unbounded();
    let mut cloneable = rx.unsync_cloneable();
    b.iter(move || {
        tx.unbounded_send(42).unwrap();
        cloneable.poll()
    });
}

#[bench]
fn two_consumer(b: &mut Bencher) {
    let (tx, rx) = futures::unsync::mpsc::unbounded();
    let mut cloneable_rx = rx.unsync_cloneable();
    let mut cloneable_rx2 = cloneable_rx.clone();;
    b.iter(move || {
        tx.unbounded_send(42).unwrap();
        let _item = cloneable_rx.poll().unwrap();
        let _item2 = cloneable_rx2.poll().unwrap();
        (_item, _item2)
    });
}

#[bench]
fn three_consumer(b: &mut Bencher) {
    let (tx, rx) = futures::unsync::mpsc::unbounded();
    let mut cloneable_rx = rx.unsync_cloneable();
    let mut cloneable_rx2 = cloneable_rx.clone();;
    let mut cloneable_rx3 = cloneable_rx.clone();;
    b.iter(move || {
        tx.unbounded_send(42).unwrap();
        let _item = cloneable_rx.poll().unwrap();
        let _item2 = cloneable_rx2.poll().unwrap();
        let _item3 = cloneable_rx3.poll().unwrap();
        (_item, _item2, _item3)
    });
}
