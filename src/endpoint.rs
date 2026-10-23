use std::path::PathBuf;

pub enum Endpoint {
    Tcp { host: String, port: u16 },
    Ipc(PathBuf),
}

impl Endpoint {
    pub fn tcp(host: impl Into<String>, port: u16) -> Endpoint {
        Endpoint::Tcp { host: host.into(), port }
    }

    pub fn ipc(path: impl Into<PathBuf>) -> Endpoint {
        Endpoint::Ipc(path.into())
    }
}

/// Formats as the URI string ZeroMQ expects, e.g. `tcp://127.0.0.1:5555`.
impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::Tcp { host, port } => write!(f, "tcp://{host}:{port}"),
            Endpoint::Ipc(path) => write!(f, "ipc://{}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_strings() {
        assert_eq!(Endpoint::tcp("127.0.0.1", 5555).to_string(), "tcp://127.0.0.1:5555");
        assert_eq!(Endpoint::ipc("/tmp/feed.sock").to_string(), "ipc:///tmp/feed.sock");
    }
}
