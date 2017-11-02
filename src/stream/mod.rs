mod unsync_cloneable;
mod find_first_map;
mod find_first;

pub use self::unsync_cloneable::UnsyncCloneable;
pub use self::find_first_map::FindFirstMap;
pub use self::find_first::FindFirst;

use futures::Stream;


pub trait StreamExt: Stream {
    /// Convert stream into "cloneable" stream but unsync.
    fn unsync_cloneable(self) -> UnsyncCloneable<Self>
    where
        Self: Sized,
    {
        self::unsync_cloneable::unsync_cloneable(self)
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
