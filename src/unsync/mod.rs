//! provides many kind of feature being usable only in a single thread.
mod unbounded;
mod clone;

pub use self::unbounded::{unbounded, UnboundedSender, UnboundedReceiver, SendError};
pub use self::clone::{into_cloneable, Cloneable};
