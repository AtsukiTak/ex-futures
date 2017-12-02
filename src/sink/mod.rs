mod unsync_cloneable;

pub use self::unsync_cloneable::UnsyncCloneable;


use futures::Sink;


/// An extension of `Sink` provided by `futures` crate.
/// Any `Sink` implements `SinkExt` automatically.
/// All you are needed to do is to import `SinkExt`.
///
/// ```
/// use ex_futures::SinkExt;
/// ```
pub trait SinkExt: Sink {
    /// Convert any kind of sink into "cloneable" sink but unsync.
    /// That is kind like mpsc.
    ///
    /// # Examples
    /// ```
    /// # extern crate futures;
    /// # extern crate ex_futures;
    /// use ex_futures::SinkExt;
    ///
    /// # fn main() {
    /// let (tx, rx) = ::futures::sync::mpsc::channel::<usize>(42);
    ///
    /// let cloneable_sink = tx.unsync_cloneable(); // Convert "tx" into cloneable.
    /// let cloneable_sink2 = cloneable_sink.clone(); // Now you can clone it.
    /// # }
    /// ```
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
