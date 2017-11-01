//! Future-aware single-threaded publish-subscribe channel
mod unbounded;

pub use self::unbounded::{unbounded, UnboundedSender, UnboundedReceiver, SendError};
