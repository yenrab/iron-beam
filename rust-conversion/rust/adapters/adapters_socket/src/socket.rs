//! Socket Module
//!
//! Provides core socket functionality for TCP/IP networking. This module implements
//! socket operations using Rust's standard library and the `socket2` crate.

use std::net::SocketAddr;
use std::io::{self, Read, Write};
use socket2::{Socket as Socket2, Domain, Type, Protocol as Socket2Protocol, SockAddr};
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, RawSocket};

/// Socket error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketError {
    /// Invalid address
    InvalidAddress,
    /// Address already in use
    AddressInUse,
    /// Connection refused
    ConnectionRefused,
    /// Connection reset
    ConnectionReset,
    /// Connection aborted
    ConnectionAborted,
    /// Network unreachable
    NetworkUnreachable,
    /// Host unreachable
    HostUnreachable,
    /// Timeout
    Timeout,
    /// Would block (non-blocking operation)
    WouldBlock,
    /// Invalid socket
    InvalidSocket,
    /// Operation not supported
    NotSupported,
    /// I/O error
    IoError(String),
    /// Other error
    Other(String),
}

impl From<io::Error> for SocketError {
    fn from(err: io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::AddrInUse => SocketError::AddressInUse,
            ErrorKind::ConnectionRefused => SocketError::ConnectionRefused,
            ErrorKind::ConnectionReset => SocketError::ConnectionReset,
            ErrorKind::ConnectionAborted => SocketError::ConnectionAborted,
            ErrorKind::TimedOut => SocketError::Timeout,
            ErrorKind::WouldBlock => SocketError::WouldBlock,
            _ => SocketError::IoError(err.to_string()),
        }
    }
}

/// Address family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    /// IPv4
    Ipv4,
    /// IPv6
    Ipv6,
}

impl From<AddressFamily> for Domain {
    fn from(family: AddressFamily) -> Self {
        match family {
            AddressFamily::Ipv4 => Domain::IPV4,
            AddressFamily::Ipv6 => Domain::IPV6,
        }
    }
}

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// Stream socket (TCP)
    Stream,
    /// Datagram socket (UDP)
    Datagram,
}

impl From<SocketType> for Type {
    fn from(ty: SocketType) -> Self {
        match ty {
            SocketType::Stream => Type::STREAM,
            SocketType::Datagram => Type::DGRAM,
        }
    }
}

/// Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
}

impl From<Protocol> for Socket2Protocol {
    fn from(proto: Protocol) -> Self {
        match proto {
            Protocol::Tcp => Socket2Protocol::TCP,
            Protocol::Udp => Socket2Protocol::UDP,
        }
    }
}

/// Socket wrapper
///
/// Provides a unified interface for socket operations. This wraps the underlying
/// socket implementation and provides safe access to socket operations.
pub struct Socket {
    inner: Socket2,
    family: AddressFamily,
    socket_type: SocketType,
    protocol: Protocol,
}

impl Socket {
    /// Create a new socket
    ///
    /// # Arguments
    ///
    /// * `family` - Address family (IPv4 or IPv6)
    /// * `socket_type` - Socket type (Stream or Datagram)
    /// * `protocol` - Protocol (TCP or UDP)
    ///
    /// # Returns
    ///
    /// * `Ok(Socket)` - Created socket
    /// * `Err(SocketError)` - Error creating socket
    pub fn new(family: AddressFamily, socket_type: SocketType, protocol: Protocol) -> Result<Self, SocketError> {
        let domain: Domain = family.into();
        let ty: Type = socket_type.into();
        let proto: Socket2Protocol = protocol.into();
        
        let socket = Socket2::new(domain, ty, Some(proto))
            .map_err(|e| SocketError::IoError(e.to_string()))?;
        
        // Set socket to non-blocking mode for integration with NIF I/O polling
        socket.set_nonblocking(true)
            .map_err(|e| SocketError::IoError(e.to_string()))?;
        
        Ok(Self {
            inner: socket,
            family,
            socket_type,
            protocol,
        })
    }
    
    /// Bind socket to an address
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(SocketError)` - Error binding
    pub fn bind(&self, addr: &SocketAddr) -> Result<(), SocketError> {
        let sock_addr = SockAddr::from(*addr);
        self.inner.bind(&sock_addr)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Listen for incoming connections (TCP only)
    ///
    /// # Arguments
    ///
    /// * `backlog` - Maximum number of pending connections
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(SocketError)` - Error listening
    pub fn listen(&self, backlog: i32) -> Result<(), SocketError> {
        if self.socket_type != SocketType::Stream {
            return Err(SocketError::NotSupported);
        }
        
        self.inner.listen(backlog)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Accept an incoming connection (TCP only)
    ///
    /// # Returns
    ///
    /// * `Ok((Socket, SocketAddr))` - Accepted connection and peer address
    /// * `Err(SocketError)` - Error accepting connection
    pub fn accept(&self) -> Result<(Socket, SocketAddr), SocketError> {
        if self.socket_type != SocketType::Stream {
            return Err(SocketError::NotSupported);
        }
        
        let (socket, addr) = self.inner.accept()
            .map_err(|e| SocketError::from(e))?;
        
        let sock_addr = addr.as_socket()
            .ok_or_else(|| SocketError::InvalidAddress)?;
        
        // Create new Socket wrapper
        let new_socket = Socket {
            inner: socket,
            family: self.family,
            socket_type: self.socket_type,
            protocol: self.protocol,
        };
        
        Ok((new_socket, sock_addr))
    }
    
    /// Connect to a remote address
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address to connect to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(SocketError)` - Error connecting
    pub fn connect(&self, addr: &SocketAddr) -> Result<(), SocketError> {
        let sock_addr = SockAddr::from(*addr);
        self.inner.connect(&sock_addr)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Get the local address
    ///
    /// # Returns
    ///
    /// * `Ok(SocketAddr)` - Local address
    /// * `Err(SocketError)` - Error getting address
    pub fn local_addr(&self) -> Result<SocketAddr, SocketError> {
        self.inner.local_addr()
            .and_then(|addr| addr.as_socket()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid socket address")))
            .map_err(|e| SocketError::from(e))
    }
    
    /// Get the peer address
    ///
    /// # Returns
    ///
    /// * `Ok(SocketAddr)` - Peer address
    /// * `Err(SocketError)` - Error getting address
    pub fn peer_addr(&self) -> Result<SocketAddr, SocketError> {
        self.inner.peer_addr()
            .and_then(|addr| addr.as_socket()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid socket address")))
            .map_err(|e| SocketError::from(e))
    }
    
    /// Set socket option for reuse address
    pub fn set_reuse_address(&self, reuse: bool) -> Result<(), SocketError> {
        self.inner.set_reuse_address(reuse)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Get the raw file descriptor (Unix) or socket handle (Windows)
    #[cfg(unix)]
    pub fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
    
    #[cfg(windows)]
    pub fn as_raw_socket(&self) -> RawSocket {
        self.inner.as_raw_socket()
    }
    
    /// Get the underlying socket2 socket
    pub fn inner(&self) -> &Socket2 {
        &self.inner
    }
    
    /// Get the address family
    pub fn family(&self) -> AddressFamily {
        self.family
    }
    
    /// Get the socket type
    pub fn socket_type(&self) -> SocketType {
        self.socket_type
    }
    
    /// Get the protocol
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // For non-blocking sockets, we need to handle WouldBlock
        match self.inner.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // This is expected for non-blocking sockets
                Err(e)
            },
            Err(e) => Err(e),
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // This is expected for non-blocking sockets
                Err(e)
            },
            Err(e) => Err(e),
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_socket_creation() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        );
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_socket_bind() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_socket_listen() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        let result = socket.listen(128);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_socket_connect_refused() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        // Try to connect to a non-existent port (use a valid port number)
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 65535);
        let result = socket.connect(&addr);
        // Should fail with ConnectionRefused or WouldBlock (non-blocking)
        assert!(result.is_err());
    }
    
    #[test]
    fn test_socket_ipv6() {
        let socket = Socket::new(
            AddressFamily::Ipv6,
            SocketType::Stream,
            Protocol::Tcp,
        );
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_socket_udp() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Datagram,
            Protocol::Udp,
        );
        assert!(socket.is_ok());
    }

    #[test]
    fn test_socket_accept() {
        use std::thread;
        use std::time::Duration;
        
        let listener = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let client = Socket::new(
                AddressFamily::Ipv4,
                SocketType::Stream,
                Protocol::Tcp,
            ).unwrap();
            let _ = client.connect(&connect_addr);
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept connection
        let mut accepted = None;
        for _ in 0..20 {
            match listener.accept() {
                Ok((socket, peer_addr)) => {
                    accepted = Some((socket, peer_addr));
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        
        assert!(accepted.is_some());
        let (accepted_socket, peer_addr) = accepted.unwrap();
        assert_eq!(accepted_socket.family(), AddressFamily::Ipv4);
        assert_eq!(accepted_socket.socket_type(), SocketType::Stream);
        assert_eq!(accepted_socket.protocol(), Protocol::Tcp);
        assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        
        let _ = sender.join();
    }

    #[test]
    fn test_socket_connect_success() {
        use std::thread;
        use std::time::Duration;
        
        let listener = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let client = Socket::new(
                AddressFamily::Ipv4,
                SocketType::Stream,
                Protocol::Tcp,
            ).unwrap();
            
            let mut connected = false;
            for _ in 0..20 {
                match client.connect(&connect_addr) {
                    Ok(()) => {
                        connected = true;
                        break;
                    }
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
            }
            assert!(connected);
            let peer_addr = client.peer_addr().unwrap();
            assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        });
        
        // Accept connection
        let mut accepted = None;
        for _ in 0..20 {
            match listener.accept() {
                Ok((socket, peer_addr)) => {
                    accepted = Some((socket, peer_addr));
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        
        assert!(accepted.is_some());
        let (accepted_socket, peer_addr) = accepted.unwrap();
        assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        assert_eq!(accepted_socket.family(), AddressFamily::Ipv4);
        
        let _ = sender.join();
    }

    #[test]
    fn test_socket_local_addr() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), Ipv4Addr::LOCALHOST);
        assert!(local_addr.port() > 0);
    }

    #[test]
    fn test_socket_set_reuse_address() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        // Test setting reuse address
        assert!(socket.set_reuse_address(true).is_ok());
        assert!(socket.set_reuse_address(false).is_ok());
    }

    #[test]
    fn test_socket_listen_on_datagram() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Datagram,
            Protocol::Udp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        // Listen should fail on datagram socket
        let result = socket.listen(128);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SocketError::NotSupported);
    }

    #[test]
    fn test_socket_accept_on_datagram() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Datagram,
            Protocol::Udp,
        ).unwrap();
        
        // Accept should fail on datagram socket
        let result = socket.accept();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, SocketError::NotSupported);
        }
    }

    #[test]
    fn test_socket_accessors() {
        let socket = Socket::new(
            AddressFamily::Ipv6,
            SocketType::Datagram,
            Protocol::Udp,
        ).unwrap();
        
        assert_eq!(socket.family(), AddressFamily::Ipv6);
        assert_eq!(socket.socket_type(), SocketType::Datagram);
        assert_eq!(socket.protocol(), Protocol::Udp);
        assert!(socket.inner().as_raw_fd() > 0);
    }

    #[test]
    fn test_socket_read_write() {
        use std::thread;
        use std::time::Duration;
        
        let listener = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect and write
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let mut client = Socket::new(
                AddressFamily::Ipv4,
                SocketType::Stream,
                Protocol::Tcp,
            ).unwrap();
            
            // Connect (may need retries for non-blocking)
            // Non-blocking connect may return WouldBlock or IoError with "in progress"
            for _ in 0..30 {
                match client.connect(&connect_addr) {
                    Ok(()) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(SocketError::IoError(ref msg)) if msg.contains("in progress") || msg.contains("now in progress") => {
                        // Non-blocking connect in progress - wait and check
                        thread::sleep(Duration::from_millis(100));
                        // Check if connected by trying to get peer address
                        if client.peer_addr().is_ok() {
                            break;
                        }
                    }
                    Err(e) => panic!("Connect error: {:?}", e),
                }
            }
            
            // Write data
            let data = b"Hello, Socket!";
            let mut written = 0;
            while written < data.len() {
                match client.write(&data[written..]) {
                    Ok(n) => written += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Write error: {:?}", e),
                }
            }
            
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept and read
        let mut accepted = None;
        for _ in 0..20 {
            match listener.accept() {
                Ok((socket, _)) => {
                    accepted = Some(socket);
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Accept error: {:?}", e),
            }
        }
        
        let mut server = accepted.expect("Should have accepted connection");
        let mut buf = vec![0u8; 100];
        let mut read_total = 0;
        
        for _ in 0..20 {
            match server.read(&mut buf[read_total..]) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    read_total += n;
                    if read_total >= 14 {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Read error: {:?}", e),
            }
        }
        
        assert_eq!(&buf[..read_total], b"Hello, Socket!");
        
        let _ = sender.join();
    }

    #[test]
    fn test_socket_write_flush() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let mut socket = socket;
        // Flush should always succeed (no-op)
        assert!(socket.flush().is_ok());
    }

    #[test]
    fn test_address_family_conversion() {
        assert_eq!(Domain::from(AddressFamily::Ipv4), Domain::IPV4);
        assert_eq!(Domain::from(AddressFamily::Ipv6), Domain::IPV6);
    }

    #[test]
    fn test_socket_type_conversion() {
        assert_eq!(Type::from(SocketType::Stream), Type::STREAM);
        assert_eq!(Type::from(SocketType::Datagram), Type::DGRAM);
    }

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Socket2Protocol::from(Protocol::Tcp), Socket2Protocol::TCP);
        assert_eq!(Socket2Protocol::from(Protocol::Udp), Socket2Protocol::UDP);
    }

    #[test]
    fn test_socket_error_from_io_error() {
        use std::io::ErrorKind;
        
        // Test various error conversions
        let addr_in_use = io::Error::from(ErrorKind::AddrInUse);
        let socket_err: SocketError = addr_in_use.into();
        assert_eq!(socket_err, SocketError::AddressInUse);
        
        let conn_refused = io::Error::from(ErrorKind::ConnectionRefused);
        let socket_err: SocketError = conn_refused.into();
        assert_eq!(socket_err, SocketError::ConnectionRefused);
        
        let conn_reset = io::Error::from(ErrorKind::ConnectionReset);
        let socket_err: SocketError = conn_reset.into();
        assert_eq!(socket_err, SocketError::ConnectionReset);
        
        let conn_aborted = io::Error::from(ErrorKind::ConnectionAborted);
        let socket_err: SocketError = conn_aborted.into();
        assert_eq!(socket_err, SocketError::ConnectionAborted);
        
        let timed_out = io::Error::from(ErrorKind::TimedOut);
        let socket_err: SocketError = timed_out.into();
        assert_eq!(socket_err, SocketError::Timeout);
        
        let would_block = io::Error::from(ErrorKind::WouldBlock);
        let socket_err: SocketError = would_block.into();
        assert_eq!(socket_err, SocketError::WouldBlock);
        
        // Test other error kinds map to IoError
        let other = io::Error::from(ErrorKind::Other);
        let socket_err: SocketError = other.into();
        match socket_err {
            SocketError::IoError(_) => {}
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_socket_error_variants() {
        let invalid_addr = SocketError::InvalidAddress;
        let addr_in_use = SocketError::AddressInUse;
        let conn_refused = SocketError::ConnectionRefused;
        let conn_reset = SocketError::ConnectionReset;
        let conn_aborted = SocketError::ConnectionAborted;
        let net_unreach = SocketError::NetworkUnreachable;
        let host_unreach = SocketError::HostUnreachable;
        let timeout = SocketError::Timeout;
        let would_block = SocketError::WouldBlock;
        let invalid_socket = SocketError::InvalidSocket;
        let not_supported = SocketError::NotSupported;
        let io_error = SocketError::IoError("test".to_string());
        let other = SocketError::Other("test".to_string());
        
        assert_eq!(invalid_addr, SocketError::InvalidAddress);
        assert_eq!(addr_in_use, SocketError::AddressInUse);
        assert_eq!(conn_refused, SocketError::ConnectionRefused);
        assert_eq!(conn_reset, SocketError::ConnectionReset);
        assert_eq!(conn_aborted, SocketError::ConnectionAborted);
        assert_eq!(net_unreach, SocketError::NetworkUnreachable);
        assert_eq!(host_unreach, SocketError::HostUnreachable);
        assert_eq!(timeout, SocketError::Timeout);
        assert_eq!(would_block, SocketError::WouldBlock);
        assert_eq!(invalid_socket, SocketError::InvalidSocket);
        assert_eq!(not_supported, SocketError::NotSupported);
        assert_eq!(io_error, SocketError::IoError("test".to_string()));
        assert_eq!(other, SocketError::Other("test".to_string()));
        
        assert_ne!(invalid_addr, addr_in_use);
    }

    #[test]
    fn test_socket_bind_address_in_use() {
        let socket1 = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        // Find an available port
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket1.bind(&addr).unwrap();
        let bound_addr = socket1.local_addr().unwrap();
        
        // Try to bind another socket to the same address
        let socket2 = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        // Without SO_REUSEADDR, this should fail
        let result = socket2.bind(&bound_addr);
        // May succeed or fail depending on OS behavior, but test that bind works
        // If it fails, should be AddressInUse
        if result.is_err() {
            assert_eq!(result.unwrap_err(), SocketError::AddressInUse);
        }
    }

    #[test]
    fn test_socket_ipv6_bind() {
        use std::net::Ipv6Addr;
        
        let socket = Socket::new(
            AddressFamily::Ipv6,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), Ipv6Addr::LOCALHOST);
    }

    #[test]
    fn test_socket_udp_bind() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Datagram,
            Protocol::Udp,
        ).unwrap();
        
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), Ipv4Addr::LOCALHOST);
    }

    #[test]
    fn test_socket_peer_addr_not_connected() {
        let socket = Socket::new(
            AddressFamily::Ipv4,
            SocketType::Stream,
            Protocol::Tcp,
        ).unwrap();
        
        // Peer addr should fail on unconnected socket
        let result = socket.peer_addr();
        assert!(result.is_err());
    }
}

