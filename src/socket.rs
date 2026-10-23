use std::{marker::PhantomData, time::Duration};

use crate::{
    Sendable, Bound, CanRecv, CanSend, Connected, Endpoint, Pub, SocketKind, SocketState, Sub,
    Unbound,
};

pub struct ZmqSocket<K: SocketKind, S: SocketState> {
    socket: zmq::Socket,
    _marker: PhantomData<(K, S)>,
}

/// A PUB socket: sends messages.
pub type Publisher<S = Unbound> = ZmqSocket<Pub, S>;

/// A SUB socket: receives messages.
pub type Subscriber<S = Unbound> = ZmqSocket<Sub, S>;

impl<K: SocketKind> ZmqSocket<K, Unbound> {
    /// Open a socket of kind `K` on a new context of its own.
    pub fn new() -> zmq::Result<Self> {
        let ctx = zmq::Context::new();
        Ok(Self {
            socket: ctx.socket(K::TYPE)?,
            _marker: PhantomData,
        })
    }

    /// Listen on `endpoint`. Consumes the socket and returns it as `Bound`.
    pub fn bind(self, endpoint: &Endpoint) -> zmq::Result<ZmqSocket<K, Bound>> {
        self.socket.bind(&endpoint.to_string())?;
        Ok(self.into_state())
    }

    /// Connect to a socket listening on `endpoint`. Consumes the socket and
    /// returns it as `Connected`.
    pub fn connect(self, endpoint: &Endpoint) -> zmq::Result<ZmqSocket<K, Connected>> {
        self.socket.connect(&endpoint.to_string())?;
        Ok(self.into_state())
    }
}

impl<K: SocketKind, S: SocketState> ZmqSocket<K, S> {
    /// Same underlying socket, new state. Only used by the transitions above.
    fn into_state<T: SocketState>(self) -> ZmqSocket<K, T> {
        ZmqSocket {
            socket: self.socket,
            _marker: PhantomData,
        }
    }
}

/// Only `CanSend` kinds (e.g. `Pub`) that are bound or connected get `send`.
impl<K: CanSend, S: Sendable> ZmqSocket<K, S> {
    pub fn send(&self, msg: &[u8]) -> zmq::Result<()> {
        self.socket.send(msg, 0)
    }
}

/// Only `CanRecv` kinds (e.g. `Sub`) that are bound or connected get `recv`.
impl<K: CanRecv, S: Sendable> ZmqSocket<K, S> {
    pub fn recv(&self) -> zmq::Result<Vec<u8>> {
        self.socket.recv_bytes(0)
    }
}

/// Options every kind has. Set them before `bind`/`connect`; some (like the
/// high-water marks) only take effect on connections made afterwards.
impl<K: SocketKind, S: SocketState> ZmqSocket<K, S> {
    /// How long to keep unsent messages after the socket is dropped.
    /// `Duration::ZERO` discards them immediately.
    pub fn set_linger(&self, linger: Duration) -> zmq::Result<()> {
        self.socket.set_linger(millis(linger))
    }
}

/// Options for kinds that send.
impl<K: CanSend, S: SocketState> ZmqSocket<K, S> {
    /// Most messages to queue per peer before the socket starts dropping them.
    pub fn set_send_hwm(&self, messages: i32) -> zmq::Result<()> {
        self.socket.set_sndhwm(messages)
    }
}

/// Options for kinds that receive.
impl<K: CanRecv, S: SocketState> ZmqSocket<K, S> {
    /// Most messages to queue per peer before the sender is held back.
    pub fn set_recv_hwm(&self, messages: i32) -> zmq::Result<()> {
        self.socket.set_rcvhwm(messages)
    }

    /// How long `recv` waits before failing with `EAGAIN`.
    /// `None` (the default) waits forever.
    pub fn set_recv_timeout(&self, timeout: Option<Duration>) -> zmq::Result<()> {
        self.socket.set_rcvtimeo(timeout.map_or(-1, millis))
    }
}

/// Milliseconds as the `i32` zmq wants, saturating instead of wrapping.
fn millis(d: Duration) -> i32 {
    d.as_millis().min(i32::MAX as u128) as i32
}

/// A SUB socket receives nothing until it subscribes to a prefix.
impl<S: SocketState> ZmqSocket<Sub, S> {
    /// Receive messages starting with `prefix` (empty = everything).
    pub fn subscribe(&self, prefix: &[u8]) -> zmq::Result<()> {
        self.socket.set_subscribe(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_sockets_of_each_kind() {
        let _p = Publisher::new().unwrap();
        let _s = Subscriber::new().unwrap();
    }

    #[test]
    fn bind_and_connect_tcp() {
        let endpoint = Endpoint::tcp("127.0.0.1", 5599);
        let _publisher = Publisher::new().unwrap().bind(&endpoint).unwrap();
        let _subscriber = Subscriber::new().unwrap().connect(&endpoint).unwrap();
    }

    #[test]
    fn publish_and_receive() {
        let endpoint = Endpoint::tcp("127.0.0.1", 5600);

        let publisher = Publisher::new().unwrap().bind(&endpoint).unwrap();

        let subscriber = Subscriber::new().unwrap();
        subscriber.subscribe(b"").unwrap();
        subscriber.set_recv_timeout(Some(Duration::from_millis(50))).unwrap();
        let subscriber = subscriber.connect(&endpoint).unwrap();

        // The subscription arrives asynchronously, so early messages can be
        // dropped ("slow joiner"). Keep publishing until one gets through.
        let msg = (0..100)
            .find_map(|_| {
                publisher.send(b"hello").unwrap();
                subscriber.recv().ok()
            })
            .expect("no message received");
        assert_eq!(msg, b"hello");
    }

    #[test]
    fn sets_options() {
        let publisher = Publisher::new().unwrap();
        publisher.set_linger(Duration::from_millis(250)).unwrap();
        publisher.set_send_hwm(10).unwrap();
        assert_eq!(publisher.socket.get_linger().unwrap(), 250);
        assert_eq!(publisher.socket.get_sndhwm().unwrap(), 10);

        let subscriber = Subscriber::new().unwrap();
        subscriber.set_recv_hwm(20).unwrap();
        subscriber.set_recv_timeout(Some(Duration::from_secs(1))).unwrap();
        assert_eq!(subscriber.socket.get_rcvhwm().unwrap(), 20);
        assert_eq!(subscriber.socket.get_rcvtimeo().unwrap(), 1000);

        subscriber.set_recv_timeout(None).unwrap();
        assert_eq!(subscriber.socket.get_rcvtimeo().unwrap(), -1);
    }
}
