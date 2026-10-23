/// A ZeroMQ socket SocketKind (PUB, SUB, ...).
pub trait SocketKind {
    const TYPE: zmq::SocketType;
}

pub struct Pub;
pub struct Sub;

impl SocketKind for Pub { const TYPE: zmq::SocketType = zmq::PUB; }
impl SocketKind for Sub { const TYPE: zmq::SocketType = zmq::SUB; }

/// Marker traits for what a Socket can do.
pub trait CanSend: SocketKind {}
pub trait CanRecv: SocketKind {}

impl CanSend for Pub {}
impl CanRecv for Sub {}
