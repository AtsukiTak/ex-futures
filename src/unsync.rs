use futures::task::{self, Task};
use futures::stream::Stream;
use futures::sink::Sink;
use futures::{Async, Poll, AsyncSink, StartSend};

use rand::{Rng, OsRng};

use std::collections::{VecDeque, HashMap};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::any::Any;
use std::error::Error;
use std::fmt;



/// Returns unbounded sender and receiver.
/// This function is like another hand of `futures::unsync::mpsc::unbounded` but
/// every item being treated need to implement `Clone` trait.
pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    const FIRST_RECEIVER_ID: usize = 0;

    let mut receive_queues = HashMap::new();
    receive_queues.insert(FIRST_RECEIVER_ID, VecDeque::new());

    let shared = Rc::new(RefCell::new(Shared {
        receive_queues: receive_queues,
        blocked_receivers: Vec::new(),
        sender_alive: true,
    }));

    let sender = UnboundedSender { shared: Rc::downgrade(&shared) };

    let receiver = UnboundedReceiver {
        id: FIRST_RECEIVER_ID,
        shared: shared,
    };

    (sender, receiver)
}



struct Shared<T> {
    receive_queues: HashMap<ReceiverId, VecDeque<Rc<T>>>,
    blocked_receivers: Vec<Task>,
    sender_alive: bool,
}


/// The transmission end of an unbounded channel.
/// This is created by the `unbounded` function.
pub struct UnboundedSender<T> {
    shared: Weak<RefCell<Shared<T>>>,
}


type ReceiverId = usize;

/// The receiving end of an unbounded channel.
/// This is created by the `unbounded` function.
///
/// This receiver is not stream of `T` but `Rc<T>`.
pub struct UnboundedReceiver<T> {
    id: ReceiverId,
    shared: Rc<RefCell<Shared<T>>>,
}



impl<T> Sink for UnboundedSender<T> {
    type SinkItem = T;
    type SinkError = SendError<T>;

    fn start_send(&mut self, msg: T) -> StartSend<T, SendError<T>> {
        self.do_send(msg)
    }


    fn poll_complete(&mut self) -> Poll<(), SendError<T>> {
        Ok(Async::Ready(()))
    }


    fn close(&mut self) -> Poll<(), SendError<T>> {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return Ok(Async::Ready(())), // No Receiver is available.
        };
        let mut shared = shared.borrow_mut();
        shared.sender_alive = false;
        Ok(Async::Ready(()))
    }
}



impl<T> UnboundedSender<T> {
    fn do_send(&self, msg: T) -> StartSend<T, SendError<T>> {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return Err(SendError(msg)), // No Receiver is available.
        };
        let mut shared = shared.borrow_mut();

        // Send msg to each queue
        let rc = Rc::new(msg);
        for queue in shared.receive_queues.values_mut() {
            queue.push_back(rc.clone());
        }

        // Notify that new msg is ready
        let tasks = ::std::mem::replace(&mut shared.blocked_receivers, Vec::new());
        for task in tasks.iter() {
            task.notify();
        }

        Ok(AsyncSink::Ready)
    }

    pub fn unbounded_send(&self, msg: T) -> Result<(), SendError<T>> {
        self.do_send(msg).map(|_| ())
    }
}




impl<T> Stream for UnboundedReceiver<T> {
    type Item = Rc<T>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Rc<T>>, ()> {
        let mut shared = self.shared.borrow_mut();

        let msg = shared.receive_queues.get_mut(&self.id).unwrap().pop_front();

        match msg {
            Some(msg) => Ok(Async::Ready(Some(msg))),
            None => {
                if !shared.sender_alive {
                    Ok(Async::Ready(None))
                } else {
                    shared.blocked_receivers.push(task::current());
                    Ok(Async::NotReady)
                }
            }
        }
    }
}



impl<T> Drop for UnboundedSender<T> {
    fn drop(&mut self) {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return,
        };
        let mut shared = shared.borrow_mut();

        shared.sender_alive = false;

        let tasks = ::std::mem::replace(&mut shared.blocked_receivers, Vec::new());
        drop(shared);
        for task in tasks.iter() {
            task.notify();
        }
    }
}



impl<T> Clone for UnboundedReceiver<T> {
    fn clone(&self) -> Self {
        let id: usize = {
            let mut rng = OsRng::new().unwrap();
            loop {
                let id = rng.gen();
                match self.shared.borrow().receive_queues.get(&id) {
                    Some(_) => continue,
                    None => break id,
                }
            }
        };

        let receiver = UnboundedReceiver {
            id: id,
            shared: self.shared.clone(),
        };
        let mut shared = self.shared.borrow_mut();
        shared.receive_queues.insert(id, VecDeque::new());
        receiver
    }
}



impl<T> Drop for UnboundedReceiver<T> {
    fn drop(&mut self) {
        let mut shared = self.shared.borrow_mut();
        shared.receive_queues.remove(&self.id);
    }
}



// {{{ SendError
pub struct SendError<T>(T);


impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("SendError").field(&"...").finish()
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "send failed because receiver is gone")
    }
}

impl<T: Any> Error for SendError<T> {
    fn description(&self) -> &str {
        "send failed because receiver is gone"
    }
}

impl<T> SendError<T> {
    /// Returns the message that was attempted to be sent but failed.
    pub fn into_inner(self) -> T {
        self.0
    }
}
// }}}
