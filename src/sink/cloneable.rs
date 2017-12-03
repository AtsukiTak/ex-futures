use futures::{Sink, Poll, Async, AsyncSink, StartSend};

use std::sync::{Arc, Mutex};


/// Convert given stream into `Cloneable`.
/// `Cloneable` is able to be cloned.
pub fn cloneable<S: Sink>(sink: S) -> Cloneable<S> {
    let shared = Shared {
        sink: sink,
        sender_count: 1,
    };

    Cloneable {
        shared: Arc::new(Mutex::new(shared)),
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
pub struct Cloneable<S: Sink> {
    shared: Arc<Mutex<Shared<S>>>,
    closed: bool,
}




impl<S: Sink> Sink for Cloneable<S> {
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    fn start_send(&mut self, msg: S::SinkItem) -> StartSend<S::SinkItem, S::SinkError> {
        if self.closed {
            panic!("A sink which is already closed are used !!");
        }

        let mut shared = self.shared.lock().unwrap();

        match shared.sink.start_send(msg) {
            Ok(AsyncSink::Ready) => Ok(AsyncSink::Ready),
            Ok(AsyncSink::NotReady(msg)) => Ok(AsyncSink::NotReady(msg)),
            Err(e) => Err(e),
        }
    }


    fn poll_complete(&mut self) -> Poll<(), S::SinkError> {
        if self.closed {
            panic!("A sink which is already closed are used !!");
        }

        let mut shared = self.shared.lock().unwrap();

        match shared.sink.poll_complete() {
            Ok(Async::Ready(())) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }


    fn close(&mut self) -> Poll<(), S::SinkError> {
        if self.closed {
            panic!("A sink which is already closed are used !!");
        }

        let mut shared = self.shared.lock().unwrap();
        shared.sender_count -= 1;
        self.closed = true;
        if shared.sender_count == 0 {
            shared.sink.close()
        } else {
            Ok(Async::Ready(()))
        }
    }
}



impl<S: Sink> Clone for Cloneable<S> {
    fn clone(&self) -> Self {
        let cloned = Cloneable {
            shared: self.shared.clone(),
            closed: false,
        };

        let mut shared = self.shared.lock().unwrap();
        shared.sender_count += 1;

        cloned
    }
}


impl<S: Sink> Drop for Cloneable<S> {
    fn drop(&mut self) {
        if !self.closed {
            let mut shared = self.shared.lock().unwrap();
            shared.sender_count -= 1;
        }
    }
}


impl<S: Sink> ::std::fmt::Debug for Cloneable<S> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "ex_futures::sink::Cloneable(..)")
    }
}
