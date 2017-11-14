use futures::{Stream, Poll, Async};

use std::rc::Rc;
use std::collections::{HashMap, VecDeque};
use std::cell::RefCell;


/// Convert given stream into `UnsyncCloneable`.
/// `UnsyncCloneable` is able to be cloned.
pub fn unsync_cloneable<S: Stream>(stream: S) -> UnsyncCloneable<S> {
    const FIRST_RECEIVER_ID: usize = 0;

    let mut receive_queues = HashMap::new();
    receive_queues.insert(FIRST_RECEIVER_ID, VecDeque::new());

    let shared = Shared {
        stream: stream,
        receive_queues: receive_queues,
    };

    UnsyncCloneable {
        id: FIRST_RECEIVER_ID,
        shared: Rc::new(RefCell::new(shared)),
    }
}


struct Shared<S: Stream> {
    stream: S,
    receive_queues: HashMap<ReceiverId, VecDeque<Msg<S::Item, S::Error>>>,
}


/// A cloneable stream being created by `into_cloneable` function.
/// You can `clone` this stream as you want.
/// Each cloned stream is also cloneable.
///
/// # Notice
/// A type of `Item` and `Error` are wrapped by `Rc`
pub struct UnsyncCloneable<S: Stream> {
    id: ReceiverId,
    shared: Rc<RefCell<Shared<S>>>,
}


type Msg<T, E> = Result<Option<T>, E>;

type ReceiverId = usize;



impl<S> Stream for UnsyncCloneable<S>
where
    S: Stream,
    S::Item: Clone,
    S::Error: Clone,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
        {
            let mut shared = self.shared.borrow_mut();

            let msg = shared.receive_queues.get_mut(&self.id).unwrap().pop_front();

            let poll = match msg {
                Some(Ok(Some(msg))) => return Ok(Async::Ready(Some(msg))),
                Some(Ok(None)) => return Ok(Async::Ready(None)),
                Some(Err(e)) => return Err(e),
                None => shared.stream.poll(),
            };

            let msg = match poll {
                Err(e) => Err(e.clone()),
                Ok(Async::Ready(Some(msg))) => Ok(Some(msg.clone())),
                Ok(Async::Ready(None)) => Ok(None),
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                }
            };

            for rx in shared.receive_queues.values_mut() {
                rx.push_back(msg.clone());
            }
        }

        self.poll()
    }
}



impl<S: Stream> Clone for UnsyncCloneable<S> {
    fn clone(&self) -> Self {
        let id = find_id(next_id(self.id), &self.shared.borrow().receive_queues);

        let cloned = UnsyncCloneable {
            id: id,
            shared: self.shared.clone(),
        };

        let mut shared = self.shared.borrow_mut();
        shared.receive_queues.insert(id, VecDeque::new());

        cloned
    }
}


impl<S: Stream> Drop for UnsyncCloneable<S> {
    fn drop(&mut self) {
        let mut shared = self.shared.borrow_mut();
        shared.receive_queues.remove(&self.id);
    }
}


impl<S: Stream> ::std::fmt::Debug for UnsyncCloneable<S> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "UnsyncCloneable({})", self.id)
    }
}


fn find_id<V>(start: ReceiverId, receivers: &HashMap<ReceiverId, V>) -> ReceiverId {
    let mut id = start;
    loop {
        match receivers.get(&id) {
            Some(_) => {
                id = next_id(id);
                continue;
            }
            None => break id,
        }
    }
}


fn next_id(id: ReceiverId) -> ReceiverId {
    match id.checked_add(1) {
        Some(id) => id,
        None => ReceiverId::min_value(),
    }
}
