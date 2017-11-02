mod unsync_cloneable;

pub use self::unsync_cloneable::UnsyncCloneable;


use futures::Sink;


pub trait SinkExt: Sink {
    fn unsync_cloneable(self) -> UnsyncCloneable<Self>
    where
        Self: Sized,
    {
        self::unsync_cloneable::unsync_cloneable(self)
    }
}


impl<S> SinkExt for S
where
    S: Sink,
{
}
