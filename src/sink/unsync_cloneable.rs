use futures::{Sink, Poll, Async, AsyncSink, StartSend};

use std::rc::Rc;
use std::cell::RefCell;


/// Convert given stream into `UnsyncCloneable`.
/// `UnsyncCloneable` is able to be cloned.
pub fn unsync_cloneable<S: Sink>(sink: S) -> UnsyncCloneable<S> {
    let shared = Shared {
        sink: sink,
        sender_count: 1,
    };

    UnsyncCloneable {
        shared: Rc::new(RefCell::new(shared)),
        closed: false,
    }
}


struct Shared<S: Sink> {
    sink: S,
    sender_count: usize,
}



/// A cloneable stream being created by `into_cloneable` function.
/// You can `clone` this stream as you want.
/// Each cloned stream is also cloneable.
pub struct UnsyncCloneable<S: Sink> {
    shared: Rc<RefCell<Shared<S>>>,
    closed: bool,
}




impl<S: Sink> Sink for UnsyncCloneable<S> {
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    fn start_send(&mut self, msg: S::SinkItem) -> StartSend<S::SinkItem, S::SinkError> {
        let mut shared = self.shared.borrow_mut();

        match shared.sink.start_send(msg) {
            Ok(AsyncSink::Ready) => Ok(AsyncSink::Ready),
            Ok(AsyncSink::NotReady(msg)) => Ok(AsyncSink::NotReady(msg)),
            Err(e) => Err(e),
        }
    }


    fn poll_complete(&mut self) -> Poll<(), S::SinkError> {
        let mut shared = self.shared.borrow_mut();

        match shared.sink.poll_complete() {
            Ok(Async::Ready(())) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }


    fn close(&mut self) -> Poll<(), S::SinkError> {
        let mut shared = self.shared.borrow_mut();
        shared.sender_count -= 1;
        self.closed = true;
        if shared.sender_count == 0 {
            shared.sink.close()
        } else {
            Ok(Async::Ready(()))
        }
    }
}



impl<S: Sink> Clone for UnsyncCloneable<S> {
    fn clone(&self) -> Self {
        let cloned = UnsyncCloneable {
            shared: self.shared.clone(),
            closed: false,
        };

        let mut shared = self.shared.borrow_mut();
        shared.sender_count += 1;

        cloned
    }
}


impl<S: Sink> Drop for UnsyncCloneable<S> {
    fn drop(&mut self) {
        if !self.closed {
            let mut shared = self.shared.borrow_mut();
            shared.sender_count -= 1;
        }
    }
}


impl<S: Sink> ::std::fmt::Debug for UnsyncCloneable<S> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "ex_futures::sink::UnsyncCloneable(..)")
    }
}
