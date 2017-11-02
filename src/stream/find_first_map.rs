use futures::{Stream, Future, Poll, Async};
use futures::stream::{FilterMap, StreamFuture};


pub fn find_first_map<S, F, B>(stream: S, f: F) -> FindFirstMap<S, F>
where
    S: Stream,
    F: FnMut(S::Item) -> Option<B>,
{
    FindFirstMap(stream.filter_map(f).into_future())
}


pub struct FindFirstMap<S, F>(StreamFuture<FilterMap<S, F>>);


impl<S, F, B> Future for FindFirstMap<S, F>
where
    S: Stream,
    F: FnMut(S::Item) -> Option<B>,
{
    type Item = (Option<B>, S);
    type Error = (S::Error, S);

    fn poll(&mut self) -> Poll<(Option<B>, S), (S::Error, S)> {
        match self.0.poll() {
            Ok(Async::Ready((b, f))) => Ok(Async::Ready((b, f.into_inner()))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err((e, s)) => Err((e, s.into_inner())),
        }
    }
}
