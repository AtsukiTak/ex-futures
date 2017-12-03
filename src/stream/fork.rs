use futures::{Stream, Poll, Async};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, TryLockError};


pub type LeftFork<S, F> = Fork<S, F>;
pub type RightFork<S, F> = Fork<S, F>;



/// Fork given stream into two stream. Please have a look at document of `StreamExt` trait.
pub fn fork<S, F, T>(stream: S, router: F) -> (LeftFork<S, F>, RightFork<S, F>)
where
    S: Stream,
    F: FnMut(&S::Item) -> T,
    Side: From<T>,
{
    let shared = Shared {
        router: router,
        stream: stream,
    };

    let shared = Arc::new(Mutex::new(shared));

    let queues = Arc::new(Mutex::new(Queues::new()));

    let left = Fork {
        route: Side::Left,
        shared: shared.clone(),
        queues: queues.clone(),
    };

    let right = Fork {
        route: Side::Right,
        shared: shared,
        queues: queues.clone(),
    };

    (left, right)
}


#[derive(Clone, Debug)]
pub enum Side {
    Left,
    Right,
}

impl From<bool> for Side {
    fn from(b: bool) -> Side {
        if b { Side::Left } else { Side::Right }
    }
}


#[derive(Debug, Clone)]
struct Queues<T, E> {
    left: VecDeque<Result<Option<T>, E>>,
    right: VecDeque<Result<Option<T>, E>>,
}


impl<T, E> Queues<T, E> {
    fn new() -> Queues<T, E> {
        Queues {
            left: VecDeque::new(),
            right: VecDeque::new(),
        }
    }

    fn get_queue_mut(&mut self, route: Side) -> &mut VecDeque<Result<Option<T>, E>> {
        match route {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    fn push_none(&mut self) {
        self.left.push_back(Ok(None));
        self.right.push_back(Ok(None));
    }

    fn push_err(&mut self, err: E)
    where
        E: Clone,
    {
        self.left.push_back(Err(err.clone()));
        self.right.push_back(Err(err));
    }
}


#[derive(Debug)]
struct Shared<S: Stream, F> {
    router: F,
    stream: S,
}



/// Fork any kind of stream into two stream like that the river branches.
/// The closure being passed this function is called "router". Each item of original stream is
/// passed to branch following to "router" decision.
/// "Router" can return not only `Side` which is `Left` or `Right` but also
/// `bool` (`true` is considered as `Left`).
///
/// # Examples
///
/// ```
/// # extern crate futures;
/// # extern crate ex_futures;
/// use ex_futures::StreamExt;
///
/// # fn main() {
/// let (tx, rx) = ::futures::sync::mpsc::channel::<usize>(42);
///
/// let (even, odd) = rx.unsync_fork(|i| i % 2 == 0);
/// # }
/// ```
///
/// # Notice
///
/// The value being returned by this function is not `Sync`. We will provide `Sync` version later.
pub struct Fork<S: Stream, F> {
    route: Side,
    queues: Arc<Mutex<Queues<S::Item, S::Error>>>,
    shared: Arc<Mutex<Shared<S, F>>>,
}


impl<S, F, T> Stream for Fork<S, F>
where
    S: Stream,
    S::Error: Clone,
    F: FnMut(&S::Item) -> T,
    T: Into<Side>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {

        // Check self queue
        match self.queues
            .lock()
            .unwrap()
            .get_queue_mut(self.route.clone())
            .pop_front() {
            Some(Ok(Some(msg))) => return Ok(Async::Ready(Some(msg))),
            Some(Ok(None)) => return Ok(Async::Ready(None)),
            Some(Err(e)) => return Err(e),
            None => {
                // Since queue is empty, we need to call poll at original future.
            }
        };

        match self.shared.try_lock() {
            Err(TryLockError::WouldBlock) => {
                // Loop again
            }
            Err(TryLockError::Poisoned(_poisoned)) => {
                // Currently we just panic thread if mutex get poisoned.
                panic!("Other thread seemd to panic during processing cloned stream.");
            }
            Ok(mut shared) => {
                // Now we got shared value.

                match shared.stream.poll() {
                    Err(e) => self.queues.lock().unwrap().push_err(e),
                    Ok(Async::Ready(Some(msg))) => {
                        let route = (&mut shared.router)(&msg).into();
                        self.queues.lock().unwrap().get_queue_mut(route).push_back(
                            Ok(
                                Some(msg),
                            ),
                        );
                    }
                    Ok(Async::Ready(None)) => self.queues.lock().unwrap().push_none(),
                    Ok(Async::NotReady) => {
                        return Ok(Async::NotReady);
                    }
                }
            }
        }

        // Loop if
        // - queue is empty and could not get shared
        // - queue is empty and get shared and new item is arrived
        self.poll()
    }
}



impl<S: Stream, F> ::std::fmt::Debug for Fork<S, F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self.route {
            Side::Left => write!(f, "LeftRoute"),
            Side::Right => write!(f, "RightRoute"),
        }
    }
}
