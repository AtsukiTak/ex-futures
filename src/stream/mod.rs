mod unsync_cloneable;

pub use self::unsync_cloneable::UnsyncCloneable;

use futures::Stream;


pub trait StreamExt: Stream {
    fn unsync_cloneable(self) -> UnsyncCloneable<Self>
    where
        Self: Sized,
    {
        self::unsync_cloneable::unsync_cloneable(self)
    }
}



impl<S> StreamExt for S
where
    S: Stream,
{
}
