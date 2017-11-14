mod unsync_cloneable;
mod find_first_map;
mod find_first;

pub use self::unsync_cloneable::UnsyncCloneable;
pub use self::find_first_map::FindFirstMap;
pub use self::find_first::FindFirst;

use futures::Stream;
use futures::stream::Then;


pub type AsErr<S: Stream, E> = Then<
    S,
    fn(Result<S::Item, ()>) -> Result<S::Item, E>,
    Result<S::Item, E>,
>;

pub trait StreamExt: Stream {
    /// Convert stream into "cloneable" stream but unsync.
    /// If your stream emits non `Clone` item or error, consider wrap it by `Rc`.
    fn unsync_cloneable(self) -> UnsyncCloneable<Self>
    where
        Self: Sized,
        Self::Item: Clone,
        Self::Error: Clone,
    {
        self::unsync_cloneable::unsync_cloneable(self)
    }


    fn as_err<E>(self) -> AsErr<Self, E>
    where
        Self: Sized,
        Self: Stream<Error = ()>,
    {
        self.then(|never_err| Ok::<_, E>(never_err.unwrap()))
    }


    fn find_first<F>(self, f: F) -> FindFirst<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        self::find_first::find_first(self, f)
    }


    /// Return `Future` which will be completed when find first item you want.
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
