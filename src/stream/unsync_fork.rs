use futures::{Stream, Poll, Async};

use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;


pub type LeftFork<S, F> = Fork<S, F>;
pub type RightFork<S, F> = Fork<S, F>;


/// Fork given stream in accordance with a given closure.
/// A closure may return `bool`. In that case an item which is stamped as `true` is in `LeftFork`
/// and vice versa.
/// Or you can simply return `Side` enum.
pub fn fork<S, F, T>(stream: S, router: F) -> (LeftFork<S, F>, RightFork<S, F>)
where
    S: Stream,
    F: FnMut(&S::Item) -> T,
    T: Into<Side>
{
    let shared = Shared {
        router: router,
        stream: stream,
        queues: Queues::new(),
    };

    let shared = Rc::new(RefCell::new(shared));

    let left = Fork {
        route: Side::Left,
        shared: shared.clone(),
    };

    let right = Fork {
        route: Side::Right,
        shared: shared,
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


pub struct Fork<S: Stream, F> {
    route: Side,
    shared: Rc<RefCell<Shared<S, F>>>,
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



impl<S: Stream, F> ::std::fmt::Debug for Fork<S, F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self.route {
            Side::Left => write!(f, "LeftRoute"),
            Side::Right => write!(f, "RightRoute"),
        }
    }
}
