use futures::{Stream, Poll, Async};
use futures::task::{self, Task};

use std::sync::{Arc, Weak, Mutex, TryLockError};
use std::collections::VecDeque;


/// Convert given stream into `Cloneable`.
/// `Cloneable` is able to be cloned.
pub fn cloneable<S: Stream>(stream: S) -> Cloneable<S> {
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    let receivers = vec![Arc::downgrade(&queue)];

    let (tx, rx) = ::std::sync::mpsc::channel();

    let shared = Shared {
        stream: stream,
        receivers: receivers,
        block_receiver: rx,
    };

    Cloneable {
        queue: queue,
        shared: Arc::new(Mutex::new(shared)),
        block_notifier: tx,
    }
}


struct Shared<S: Stream> {
    stream: S,
    receivers: Vec<Weak<Queue<S>>>,
    block_receiver: ::std::sync::mpsc::Receiver<Task>,
}


type Queue<S: Stream> = Mutex<VecDeque<Result<Option<S::Item>, S::Error>>>;


/// A cloneable stream being created by `unsync_cloneable` function.
/// You can `clone` this stream as you want.
/// Each cloned stream is also cloneable.
///
/// If you want to see examples, please have a look at document of `StreamExt` trait.
///
/// # Panic
///
/// This stream will panic when another `Cloneable` stream panics.
pub struct Cloneable<S: Stream> {
    queue: Arc<Queue<S>>,
    shared: Arc<Mutex<Shared<S>>>,
    block_notifier: ::std::sync::mpsc::Sender<Task>,
}



impl<S> Stream for Cloneable<S>
where
    S: Stream,
    S::Item: Clone,
    S::Error: Clone,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
        // Check self queue
        let msg_res = match self.queue.lock() {
            Ok(mut queue) => queue.pop_front(),
            Err(_poisoned) => {
                // Currently we just panic thread if mutex is poisoned.
                panic!("Other thread seems to panic during processing cloned stream.")
            }
        };

        match msg_res {
            Some(Ok(Some(msg))) => return Ok(Async::Ready(Some(msg))),
            Some(Ok(None)) => return Ok(Async::Ready(None)),
            Some(Err(e)) => return Err(e),
            None => (),
        }

        {
            // Try to get `Shared`.
            let mut shared = match self.shared.try_lock() {
                Ok(shared) => shared,
                Err(TryLockError::WouldBlock) => {
                    self.block_notifier.send(task::current()).unwrap(); // Never be disconnected.
                    return Ok(Async::NotReady);
                }
                Err(TryLockError::Poisoned(_poisoned)) => {
                    // Currently we just panic thread if mutex is poisoned.
                    panic!("Other thread seems to panic during processing cloned stream.")
                }
            };

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
            // We do it at `Cloneable::clone` function.
            for rx in shared.receivers.iter().filter_map(Weak::upgrade) {
                match rx.lock() {
                    Ok(mut queue) => queue.push_back(msg.clone()),
                    Err(_poisoned) => {
                        // Currently we just panic thread if mutex is poisoned.
                        panic!("Other thread seems to panic during processing cloned stream.")
                    }
                }
            }

            // Probably there is better design.
            let tasks = {
                let mut vec = Vec::new();
                while let Ok(task) = shared.block_receiver.try_recv() {
                    vec.push(task);
                }
                vec
            };

            drop(shared); // We need not to do this but this is more explicitly.

            tasks.iter().for_each(|task| task.notify());
        }

        self.poll()
    }
}



impl<S: Stream> Clone for Cloneable<S> {
    fn clone(&self) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        let mut shared = match self.shared.lock() {
            Ok(shared) => shared,
            Err(_poisoned) => {
                // Currently we just panic thread if mutex is poisoned.
                panic!("Other thread seems to panic during processing cloned stream.")
            }
        };

        shared.receivers.retain(|weak| weak.upgrade().is_some()); // Remove outdated receivers.
        shared.receivers.push(Arc::downgrade(&queue));
        drop(shared);

        Cloneable {
            queue: queue,
            shared: self.shared.clone(),
            block_notifier: self.block_notifier.clone(),
        }
    }
}



impl<S: Stream> ::std::fmt::Debug for Cloneable<S> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "Cloneable(..)")
    }
}
