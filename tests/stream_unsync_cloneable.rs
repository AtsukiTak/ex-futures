extern crate ex_futures;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use futures::stream::unfold;
use futures::future::ok;

use ex_futures::StreamExt;

use tokio_core::reactor::Core;



#[test]
fn clone() {
    let stream = unfold(0, |i| Some(ok::<(usize, usize), u8>((i, i + 1)))).take(4);

    let cloneable = stream.unsync_cloneable();
    let cloneable2 = cloneable.clone();

    assert_eq!(
        cloneable.map(|i| *i).collect().wait().unwrap(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        cloneable2.map(|i| *i).collect().wait().unwrap(),
        [0, 1, 2, 3]
    );
}


#[test]
fn interval() {
    let mut core = Core::new().unwrap();

    let (tx, rx) = futures::sync::mpsc::unbounded();
    std::thread::spawn(move || for i in 0..4 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        tx.unbounded_send(i).unwrap();
    });

    let cloneable = rx.unsync_cloneable();
    let cloneable2 = cloneable.clone();

    assert_eq!(
        core.run(cloneable.map(|i| *i).collect()).unwrap(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        core.run(cloneable2.map(|i| *i).collect()).unwrap(),
        [0, 1, 2, 3]
    );
}


#[test]
fn zip() {
    let mut core = Core::new().unwrap();

    let (tx, rx) = futures::sync::mpsc::unbounded();
    std::thread::spawn(move || for i in 0..4 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        tx.unbounded_send(i).unwrap();
    });

    let cloneable = rx.unsync_cloneable();
    let cloneable2 = cloneable.clone();;

    let fut1 = cloneable.map(|i| *i).collect();
    let fut2 = cloneable2.map(|i| *i).collect();
    let joined = fut1.join(fut2);

    let (res1, res2) = core.run(joined).unwrap();

    assert_eq!(res1, [0, 1, 2, 3]);
    assert_eq!(res2, [0, 1, 2, 3]);
}
