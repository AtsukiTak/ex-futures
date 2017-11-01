use futures::task::{self, Task};
use futures::stream::Stream;
use futures::sink::Sink;
use futures::{Async, Poll, AsyncSink, StartSend};

use rand::{Rng, OsRng};

use std::collections::{VecDeque, HashMap};
use std::rc::{Rc, Weak};
use std::cell::RefCell;


const FIRST_RECEIVER_ID: usize = 0;


pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
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
    receive_queues: HashMap<ReceiverId, VecDeque<T>>,
    blocked_receivers: Vec<Task>,
    sender_alive: bool,
}


pub struct UnboundedSender<T> {
    shared: Weak<RefCell<Shared<T>>>,
}


type ReceiverId = usize;

pub struct UnboundedReceiver<T> {
    id: ReceiverId,
    shared: Rc<RefCell<Shared<T>>>,
}


#[derive(Debug)]
pub struct SendError<T>(T);


impl<T: Clone> Sink for UnboundedSender<T> {
    type SinkItem = T;
    type SinkError = SendError<T>;

    fn start_send(&mut self, msg: T) -> StartSend<T, SendError<T>> {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return Err(SendError(msg)), // No Receiver is available.
        };
        let mut shared = shared.borrow_mut();

        // Send msg to each queue
        for queue in shared.receive_queues.values_mut() {
            queue.push_back(msg.clone());
        }

        // Notify that new msg is ready
        let tasks = ::std::mem::replace(&mut shared.blocked_receivers, Vec::new());
        for task in tasks.iter() {
            task.notify();
        }

        Ok(AsyncSink::Ready)
    }


    fn poll_complete(&mut self) -> Poll<(), SendError<T>> {
        Ok(Async::Ready(()))
    }
}



impl<T> Stream for UnboundedReceiver<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<T>, ()> {
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
