//! Android Socket-to-Network Binding Layer
//!
//! Provides direct NDK bindings to `android_setsocknetwork` ensuring that every
//! upstream TCP and UDP socket created by VPNBridge is bound to the validated
//! Android `TRANSPORT_VPN` network handle before connection and transmission.

pub mod jni;

use async_trait::async_trait;
use std::net::SocketAddr;
#[cfg(target_os = "android")]
use std::os::raw::c_int;
use tokio::net::{TcpStream, UdpSocket};
use vpnbridge_core::error::{Error, Result};
use vpnbridge_core::state::{VpnBindingReceipt, VpnGeneration};
use vpnbridge_core::traits::ProtectedSocketBinder;

#[cfg(target_os = "android")]
extern "C" {
    /// NDK API: Binds the socket to the specified network.
    /// Returns 0 on success, or -1 with errno set on failure.
    fn android_setsocknetwork(network_handle: u64, fd: c_int) -> c_int;
}

/// Production Android Socket Binder using NDK `android_setsocknetwork`.
pub struct AndroidSocketBinder {
    generation: VpnGeneration,
}

impl AndroidSocketBinder {
    pub fn new(generation: VpnGeneration) -> Self {
        Self { generation }
    }
}

#[async_trait]
impl ProtectedSocketBinder for AndroidSocketBinder {
    fn generation(&self) -> &VpnGeneration {
        &self.generation
    }

    async fn connect_tcp(&self, target_addr: SocketAddr) -> Result<(TcpStream, VpnBindingReceipt)> {
        let handle = self.generation.current_network_handle();
        let gen = self.generation.current_generation();

        if handle == 0 {
            return Err(Error::VpnNotActive);
        }

        let domain = if target_addr.is_ipv4() {
            socket2::Domain::IPV4
        } else {
            socket2::Domain::IPV6
        };

        let socket = socket2::Socket::new(domain, socket2::Type::STREAM, Some(socket2::Protocol::TCP))
            .map_err(|e| Error::Io(format!("Failed to create TCP socket: {e}")))?;

        #[cfg(target_os = "android")]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            let ret = unsafe { android_setsocknetwork(handle, fd) };
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                return Err(Error::SocketBindFailed {
                    network_handle: handle,
                    reason: format!("android_setsocknetwork failed: {err}"),
                });
            }
        }

        socket
            .set_nonblocking(true)
            .map_err(|e| Error::Io(format!("Failed to set nonblocking: {e}")))?;

        // Initiate connection on the pre-bound socket
        match socket.connect(&target_addr.into()) {
            Ok(()) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            #[cfg(unix)]
            Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {}
            Err(e) => {
                return Err(Error::Io(format!("Failed to initiate connect to {target_addr}: {e}")));
            }
        }

        let std_stream: std::net::TcpStream = socket.into();
        let tokio_stream = TcpStream::from_std(std_stream)
            .map_err(|e| Error::Io(format!("Failed to convert to Tokio TcpStream: {e}")))?;

        // Wait until socket is writable/connected
        tokio_stream
            .writable()
            .await
            .map_err(|e| Error::Io(format!("Failed waiting for socket write readiness: {e}")))?;

        if let Some(err) = tokio_stream
            .take_error()
            .map_err(|e| Error::Io(format!("take_error failed: {e}")))?
        {
            return Err(Error::Io(format!("Socket connect error: {err}")));
        }

        let receipt = VpnBindingReceipt {
            generation: gen,
            network_handle: handle,
            created_at_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        Ok((tokio_stream, receipt))
    }

    async fn create_bound_udp(&self) -> Result<(UdpSocket, VpnBindingReceipt)> {
        let handle = self.generation.current_network_handle();
        let gen = self.generation.current_generation();

        if handle == 0 {
            return Err(Error::VpnNotActive);
        }

        let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))
            .map_err(|e| Error::Io(format!("Failed to create UDP socket: {e}")))?;

        #[cfg(target_os = "android")]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            let ret = unsafe { android_setsocknetwork(handle, fd) };
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                return Err(Error::SocketBindFailed {
                    network_handle: handle,
                    reason: format!("android_setsocknetwork UDP failed: {err}"),
                });
            }
        }

        socket
            .set_nonblocking(true)
            .map_err(|e| Error::Io(format!("Failed to set nonblocking: {e}")))?;

        let std_socket: std::net::UdpSocket = socket.into();
        let tokio_udp = UdpSocket::from_std(std_socket)
            .map_err(|e| Error::Io(format!("Failed to convert to Tokio UdpSocket: {e}")))?;

        let receipt = VpnBindingReceipt {
            generation: gen,
            network_handle: handle,
            created_at_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        Ok((tokio_udp, receipt))
    }
}
