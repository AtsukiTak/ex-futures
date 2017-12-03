use futures::{Stream, Poll, Async};

use std::rc::{Rc, Weak};
use std::collections::VecDeque;
use std::cell::RefCell;


/// Convert given stream into `UnsyncCloneable`.
/// `UnsyncCloneable` is able to be cloned.
pub fn unsync_cloneable<S: Stream>(stream: S) -> UnsyncCloneable<S> {
    let queue = Rc::new(RefCell::new(VecDeque::new()));

    let receivers = vec![Rc::downgrade(&queue)];

    let shared = Shared {
        stream: stream,
        receivers: receivers,
    };

    UnsyncCloneable {
        queue: queue,
        shared: Rc::new(RefCell::new(shared)),
    }
}


struct Shared<S: Stream> {
    stream: S,
    receivers: Vec<Weak<Queue<S>>>,
}


type Queue<S: Stream> = RefCell<VecDeque<Result<Option<S::Item>, S::Error>>>;


/// A cloneable stream being created by `unsync_cloneable` function.
/// You can `clone` this stream as you want.
/// Each cloned stream is also cloneable.
pub struct UnsyncCloneable<S: Stream> {
    queue: Rc<Queue<S>>,
    shared: Rc<RefCell<Shared<S>>>,
}



impl<S> Stream for UnsyncCloneable<S>
where
    S: Stream,
    S::Item: Clone,
    S::Error: Clone,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
        // Check self queue
        match self.queue.borrow_mut().pop_front() { // Never panics because this is unsync.
            Some(Ok(Some(msg))) => return Ok(Async::Ready(Some(msg))),
            Some(Ok(None)) => return Ok(Async::Ready(None)),
            Some(Err(e)) => return Err(e),
            None => (),
        }

        {
            let mut shared = self.shared.borrow_mut();
            let poll = shared.stream.poll();

            let msg = match poll {
                Err(e) => Err(e.clone()),
                Ok(Async::Ready(Some(msg))) => Ok(Some(msg)),
                Ok(Async::Ready(None)) => Ok(None),
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                }
            };

            // Do not remove outdated receivers here because of performance.
            // We do it at `UnsyncCloneable::clone` function.
            for rx in shared.receivers.iter().filter_map(Weak::upgrade) {
                rx.borrow_mut().push_back(msg.clone()); // Never panics because this is unsync.
            }
        }

        self.poll()
    }
}



impl<S: Stream> Clone for UnsyncCloneable<S> {
    fn clone(&self) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));

        let mut shared = self.shared.borrow_mut();
        shared.receivers.retain(|weak| weak.upgrade().is_some()); // Remove outdated receivers.
        shared.receivers.push(Rc::downgrade(&queue));
        drop(shared);

        UnsyncCloneable {
            queue: queue,
            shared: self.shared.clone(),
        }
    }
}



impl<S: Stream> ::std::fmt::Debug for UnsyncCloneable<S> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "UnsyncCloneable(..)")
    }
}
