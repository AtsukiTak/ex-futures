extern crate ex_futures;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream, Sink};
use futures::sync::mpsc::{channel, unbounded, SendError};

use ex_futures::SinkExt;



#[test]
fn send() {
    let (tx, rx) = unbounded();

    let cloneable = tx.cloneable();
    let cloneable2 = cloneable.clone();

    // After send, both sink is droped.
    // Then stream "rx" will be finish.
    cloneable.send_all(new_stream()).wait().unwrap();
    cloneable2.send_all(new_stream()).wait().unwrap();

    assert_eq!(rx.collect().wait().unwrap(), [0, 1, 0, 1]);
}



#[test]
fn join() {
    let (tx, rx) = unbounded();

    let cloneable = tx.cloneable();
    let cloneable2 = cloneable.clone();

    let fut1 = cloneable.send_all(new_stream());
    let fut2 = cloneable2.send_all(new_stream());

    fut1.join(fut2).wait().unwrap();

    assert_eq!(rx.collect().wait().unwrap(), [0, 0, 1, 1]);
}



fn new_stream() -> Box<Stream<Item = usize, Error = SendError<usize>>> {
    let (tx, rx) = channel(1);

    std::thread::spawn(move || {
        let mut tx = tx;
        for i in 0..2 {
            std::thread::sleep(::std::time::Duration::from_millis(100));
            tx = tx.send(i).wait().unwrap();
        }
    });

    Box::new(rx.then(|f| Ok::<_, _>(f.unwrap())))
}
