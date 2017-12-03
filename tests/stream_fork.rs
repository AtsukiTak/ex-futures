extern crate ex_futures;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use futures::stream::unfold;
use futures::future::ok;

use ex_futures::StreamExt;

use tokio_core::reactor::Core;



#[test]
fn method_should_not_panic() {
    let stream = unfold(0, |i| Some(ok::<(usize, usize), u8>((i, i + 1)))).take(8);

    let (even, odd) = stream.fork(|i| i % 2 == 0);

    assert_eq!(
        even.collect().wait().unwrap(),
        [0, 2, 4, 6]
    );
    assert_eq!(
        odd.collect().wait().unwrap(),
        [1, 3, 5, 7]
    );
}


#[test]
fn interval() {
    let mut core = Core::new().unwrap();

    let (tx, rx) = futures::sync::mpsc::unbounded();
    std::thread::spawn(move || for i in 0..4_u8 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        tx.unbounded_send(i).unwrap();
    });

    let (zero, other) = rx.fork(|i| *i == 0_u8);

    assert_eq!(
        core.run(zero.collect()).unwrap(),
        [0]
    );
    assert_eq!(
        core.run(other.collect()).unwrap(),
        [1, 2, 3]
    );
}


#[test]
fn join() {
    let mut core = Core::new().unwrap();

    let (tx, rx) = futures::sync::mpsc::unbounded();
    std::thread::spawn(move || for i in 0..4u8 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        tx.unbounded_send(i).unwrap();
    });

    let (odd, even) = rx.fork(|i| i % 2 == 1);

    let fut1 = odd.collect();
    let fut2 = even.collect();
    let joined = fut1.join(fut2);

    let (res1, res2) = core.run(joined).unwrap();

    assert_eq!(res1, [1, 3]);
    assert_eq!(res2, [0, 2]);
}
