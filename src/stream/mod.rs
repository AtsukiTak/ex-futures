mod unsync_cloneable;
mod fork;
mod unsync_fork;
mod cloneable;
mod find_first_map;
mod find_first;

pub use self::cloneable::Cloneable;
pub use self::unsync_cloneable::UnsyncCloneable;
pub use self::find_first_map::FindFirstMap;
pub use self::find_first::FindFirst;
pub use self::fork::{fork, LeftFork, RightFork, Fork, Side};
pub use self::unsync_fork::{unsync_fork, LeftUnsyncFork, RightUnsyncFork, UnsyncFork};

use futures::Stream;
use futures::stream::Then;


pub type AsErr<S: Stream, E> = Then<
    S,
    fn(Result<S::Item, ()>) -> Result<S::Item, E>,
    Result<S::Item, E>,
>;

/// An extention of `Stream` provided by `futures` crate.
/// Any `Stream` implements `StreamExt` automatically.
/// All you are needed to do is to import `StreamExt`
///
/// ```
/// use ex_futures::StreamExt;
/// ```
pub trait StreamExt: Stream {
    /// Convert any kind of stream into "cloneable" stream.
    /// The `Item` and `Error` need to implement `Clone`. If not, consider wrap it by `Arc`.
    ///
    /// # Notice
    ///
    /// This feature is work. But does not have high performance.
    /// If you need not to `Sync`, please use `unsync_cloneable` function. That is fast enough.
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
    /// let cloneable_rx = rx.cloneable(); // Convert "rx" into cloneable.
    /// let cloneable_rx2 = cloneable_rx.clone(); // Now you can clone it.
    /// # }
    /// ```
    fn cloneable(self) -> Cloneable<Self>
    where
        Self: Sized,
        Self::Item: Clone,
        Self::Error: Clone,
    {
        self::cloneable::cloneable(self)
    }


    /// Convert any kind of stream into "cloneable" stream but unsync.
    /// If your stream emits non `Clone` item or error, consider wrap it by `Rc`.
    ///
    /// Each cloneable stream has its own queue. And each item of original stream is cloned
    /// and queued to there.
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
    /// let cloneable_rx = rx.unsync_cloneable(); // Convert "rx" into cloneable.
    /// let cloneable_rx2 = cloneable_rx.clone(); // Now you can clone it.
    /// # }
    /// ```
    ///
    /// # Notice
    ///
    /// The value being returned by this function is not `Sync`. We will provide `Sync` version later.
    fn unsync_cloneable(self) -> UnsyncCloneable<Self>
    where
        Self: Sized,
        Self::Item: Clone,
        Self::Error: Clone,
    {
        self::unsync_cloneable::unsync_cloneable(self)
    }


    fn fork<F, T>(self, router: F) -> (LeftFork<Self, F>, RightFork<Self, F>)
    where
        Self: Sized,
        Self::Error: Clone,
        F: FnMut(&Self::Item) -> T,
        Side: From<T>,
    {
        self::fork::fork(self, router)
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
    fn unsync_fork<F, T>(self, router: F) -> (LeftUnsyncFork<Self, F>, RightUnsyncFork<Self, F>)
    where
        Self: Sized,
        Self::Error: Clone,
        F: FnMut(&Self::Item) -> T,
        Side: From<T>,
    {
        self::unsync_fork::unsync_fork(self, router)
    }


    /// Converts `Error` association type which is () into any kind of type you want.
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
    /// struct MyError();
    ///
    /// let rx = rx.as_err::<MyError>(); // Accomplished by this function.
    /// # }
    /// ```
    fn as_err<E>(self) -> AsErr<Self, E>
    where
        Self: Sized,
        Self: Stream<Error = ()>,
    {
        self.then(|never_err| Ok::<_, E>(never_err.unwrap()))
    }


    /// Returns `Future` which will be completed when find first item you want.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate futures;
    /// # extern crate ex_futures;
    /// use ex_futures::StreamExt;
    /// use futures::{Sink, Future};
    ///
    /// # fn main() {
    /// let (tx, rx) = ::futures::unsync::mpsc::channel::<usize>(42);
    ///
    /// tx.send(42).wait();
    ///
    /// let fut = rx.find_first(|i| *i == 42);
    /// let (the_Answer_to_the_Ultimate_Question_of_Life_the_Universe_and_Everything, rx) = fut.wait().unwrap();
    /// # }
    /// ```
    fn find_first<F>(self, f: F) -> FindFirst<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        self::find_first::find_first(self, f)
    }


    /// Similar function to `StreamExt::find_first`. The only deference is that this function "maps"
    /// the result.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate futures;
    /// # extern crate ex_futures;
    /// use ex_futures::StreamExt;
    /// use futures::{Sink, Future};
    ///
    /// # fn main() {
    /// let (tx, rx) = ::futures::unsync::mpsc::channel::<usize>(42);
    ///
    /// tx.send(0).wait();
    ///
    /// let first_odd_half_fut = rx.find_first_map(|i| if i % 2 == 0 { Some(i / 2) } else { None });
    /// let (first_odd_half, continue_rx) = first_odd_half_fut.wait().unwrap();
    /// # }
    /// ```
    fn find_first_map<F, B>(self, f: F) -> FindFirstMap<Self, F>
    where
        F: FnMut(Self::Item) -> Option<B>,
        Self: Sized,
    {
        self::find_first_map::find_first_map(self, f)
    }
}



impl<S> StreamExt for S
where
    S: Stream,
{
}
