extern crate ex_futures;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use futures::stream::unfold;
use futures::future::ok;

use ex_futures::StreamExt;



#[test]
fn clone() {
    let stream = unfold(0, |i| Some(ok::<(usize, usize), u8>((i, i + 1)))).take(4);

    let cloneable = StreamExt::unsync_cloneable(stream);
    let cloneable2 = cloneable.clone();

    assert_eq!(cloneable.map(|i| *i).collect().wait().unwrap(), [0, 1, 2, 3]);
    assert_eq!(cloneable2.map(|i| *i).collect().wait().unwrap(), [0, 1, 2, 3]);
}
