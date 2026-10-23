/// Where a socket is in its life: not yet attached, or attached to an endpoint.
pub trait SocketState {}

/// Freshly created: neither bound nor connected.
pub struct Unbound;
/// Listening on an endpoint (via `bind`).
pub struct Bound;
/// Attached to a listening peer (via `connect`).
pub struct Connected;

impl SocketState for Unbound {}
impl SocketState for Bound {}
impl SocketState for Connected {}

/// Marker trait: states in which messages may flow (`Bound` or `Connected`).
pub trait Sendable: SocketState {}

impl Sendable for Bound {}
impl Sendable for Connected {}
