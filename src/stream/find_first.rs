use futures::{Stream, Future, Poll, Async};
use futures::stream::{Filter, StreamFuture};


pub fn find_first<S, F>(stream: S, f: F) -> FindFirst<S, F>
where
    S: Stream,
    F: FnMut(&S::Item) -> bool,
{
    FindFirst(stream.filter(f).into_future())
}


pub struct FindFirst<S, F>(StreamFuture<Filter<S, F>>);


impl<S, F> Future for FindFirst<S, F>
where
    S: Stream,
    F: FnMut(&S::Item) -> bool,
{
    type Item = (Option<S::Item>, S);
    type Error = (S::Error, S);

    fn poll(&mut self) -> Poll<(Option<S::Item>, S), (S::Error, S)> {
        match self.0.poll() {
            Ok(Async::Ready((i, f))) => Ok(Async::Ready((i, f.into_inner()))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err((e, s)) => Err((e, s.into_inner())),
        }
    }
}
