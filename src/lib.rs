//! Type-safe PUB/SUB sockets on top of [`zmq`].
//!
//! The socket kind is part of the type, so misuse is a compile error instead
//! of a runtime failure: a [`Pub`] socket can [`send`](ZmqSocket::send) but
//! not receive, and a [`Sub`] socket can [`recv`](ZmqSocket::recv) but not send.
//!
//! # Example
//!
//! ```
//! use zmq_burrito::{Endpoint, Publisher, Subscriber};
//!
//! let endpoint = Endpoint::tcp("127.0.0.1", 5601);
//!
//! // The publisher binds...
//! let publisher = Publisher::new()?.bind(&endpoint)?;
//!
//! // ...and the subscriber connects and subscribes ("" = every message).
//! let subscriber = Subscriber::new()?;
//! subscriber.subscribe(b"")?;
//! let subscriber = subscriber.connect(&endpoint)?;
//!
//! // Subscriptions arrive asynchronously ("slow joiner"), so a message sent
//! // right away can be dropped. Wait briefly, then send.
//! std::thread::sleep(std::time::Duration::from_millis(200));
//! publisher.send(b"hello")?;
//!
//! assert_eq!(subscriber.recv()?, b"hello");
//! # Ok::<(), zmq::Error>(())
//! ```
//!
//! These are rejected at compile time:
//!
//! ```compile_fail
//! # use zmq_burrito::Publisher;
//! let publisher = Publisher::new().unwrap();
//! publisher.recv().unwrap(); // `Pub` is not `CanRecv`
//! ```
//!
//! ```compile_fail
//! # use zmq_burrito::Publisher;
//! let publisher = Publisher::new().unwrap();
//! publisher.send(b"hi").unwrap(); // still `Unbound`: bind or connect first
//! ```
//!
//! ```compile_fail
//! # use zmq_burrito::Subscriber;
//! let subscriber = Subscriber::new().unwrap();
//! subscriber.send(b"hi").unwrap(); // `Sub` is not `CanSend`
//! ```

mod endpoint;
mod kind;
mod socket;
mod state;

pub use endpoint::Endpoint;
pub use kind::{CanRecv, CanSend, Pub, SocketKind, Sub};
pub use state::{Sendable, Bound, Connected, SocketState, Unbound};
pub use socket::{Publisher, Subscriber, ZmqSocket};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test(){
        let socket = Publisher::new().unwrap();
        let _ = socket.bind(&Endpoint::Ipc("/tmp/ipc1".into()));
    }
}