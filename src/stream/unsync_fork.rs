use futures::{Stream, Poll, Async};

use super::fork::Side;

use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;


pub type LeftUnsyncFork<S, F> = UnsyncFork<S, F>;
pub type RightUnsyncFork<S, F> = UnsyncFork<S, F>;



/// UnsyncFork given stream into two stream. Please have a look at document of `StreamExt` trait.
pub fn unsync_fork<S, F, T>(stream: S, router: F) -> (LeftUnsyncFork<S, F>, RightUnsyncFork<S, F>)
where
    S: Stream,
    F: FnMut(&S::Item) -> T,
    Side: From<T>,
{
    let shared = Shared {
        router: router,
        stream: stream,
        queues: Queues::new(),
    };

    let shared = Rc::new(RefCell::new(shared));

    let left = UnsyncFork {
        route: Side::Left,
        shared: shared.clone(),
    };

    let right = UnsyncFork {
        route: Side::Right,
        shared: shared,
    };

    (left, right)
}



#[derive(Debug)]
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
    queues: Queues<S::Item, S::Error>,
}



/// UnsyncFork any kind of stream into two stream like that the river branches.
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
pub struct UnsyncFork<S: Stream, F> {
    route: Side,
    shared: Rc<RefCell<Shared<S, F>>>,
}


impl<S, F, T> Stream for UnsyncFork<S, F>
where
    S: Stream,
    S::Error: Clone,
    F: FnMut(&S::Item) -> T,
    T: Into<Side>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
        {
            let mut shared = self.shared.borrow_mut();

            let msg = shared.queues.get_queue_mut(self.route.clone()).pop_front();

            let poll = match msg {
                Some(Ok(Some(msg))) => return Ok(Async::Ready(Some(msg))),
                Some(Ok(None)) => return Ok(Async::Ready(None)),
                Some(Err(e)) => return Err(e),
                None => shared.stream.poll(),
            };

            match poll {
                Err(e) => shared.queues.push_err(e),
                Ok(Async::Ready(Some(msg))) => {
                    let route = (&mut shared.router)(&msg).into();
                    shared.queues.get_queue_mut(route).push_back(Ok(Some(msg)));
                }
                Ok(Async::Ready(None)) => shared.queues.push_none(),
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                }
            }
        }

        self.poll()
    }
}



impl<S: Stream, F> ::std::fmt::Debug for UnsyncFork<S, F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self.route {
            Side::Left => write!(f, "LeftRoute"),
            Side::Right => write!(f, "RightRoute"),
        }
    }
}
