//! Test TCP and UDP Echo Servers

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio_util::sync::CancellationToken;
use vpnbridge_core::error::{Error, Result};

pub struct TcpEchoServer {
    _listener_addr: SocketAddr,
    bytes_echoed: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

impl TcpEchoServer {
    pub async fn start() -> Result<(Self, SocketAddr)> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| Error::Io(e.to_string()))?;
        let addr = listener.local_addr().map_err(|e| Error::Io(e.to_string()))?;
        let cancel_token = CancellationToken::new();
        let bytes_echoed = Arc::new(AtomicU64::new(0));

        let token_clone = cancel_token.clone();
        let bytes_clone = bytes_echoed.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = token_clone.cancelled() => break,
                    accept_res = listener.accept() => {
                        if let Ok((mut stream, _)) = accept_res {
                            let bytes_echo = bytes_clone.clone();
                            tokio::spawn(async move {
                                let mut buf = vec![0u8; 8192];
                                loop {
                                    match stream.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            if stream.write_all(&buf[..n]).await.is_err() {
                                                break;
                                            }
                                            bytes_echo.fetch_add(n as u64, Ordering::Relaxed);
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        Ok((
            Self {
                _listener_addr: addr,
                bytes_echoed,
                cancel_token,
            },
            addr,
        ))
    }

    pub fn bytes_echoed(&self) -> u64 {
        self.bytes_echoed.load(Ordering::Relaxed)
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}

pub struct UdpEchoServer {
    _socket_addr: SocketAddr,
    packets_echoed: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

impl UdpEchoServer {
    pub async fn start() -> Result<(Self, SocketAddr)> {
        let socket = Arc::new(
            UdpSocket::bind("127.0.0.1:0")
                .await
                .map_err(|e| Error::Io(e.to_string()))?,
        );
        let addr = socket.local_addr().map_err(|e| Error::Io(e.to_string()))?;
        let cancel_token = CancellationToken::new();
        let packets_echoed = Arc::new(AtomicU64::new(0));

        let token_clone = cancel_token.clone();
        let packets_clone = packets_echoed.clone();
        let sock_clone = socket.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                tokio::select! {
                    _ = token_clone.cancelled() => break,
                    recv_res = sock_clone.recv_from(&mut buf) => {
                        if let Ok((len, peer)) = recv_res {
                            if sock_clone.send_to(&buf[..len], peer).await.is_ok() {
                                packets_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        });

        Ok((
            Self {
                _socket_addr: addr,
                packets_echoed,
                cancel_token,
            },
            addr,
        ))
    }

    pub fn packets_echoed(&self) -> u64 {
        self.packets_echoed.load(Ordering::Relaxed)
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}
