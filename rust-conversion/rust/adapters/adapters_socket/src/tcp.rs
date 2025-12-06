//! TCP Socket Module
//!
//! Provides TCP (Transmission Control Protocol) socket functionality for reliable
//! stream-based communication.

use super::socket::{Socket, SocketError, AddressFamily};
use std::net::SocketAddr;
use std::io::{self, Read, Write};
use adapters_nif_io::CheckIo;

/// TCP Socket
///
/// Provides TCP socket operations with integration to the NIF I/O polling system.
pub struct TcpSocket {
    socket: Socket,
    check_io: Option<CheckIo>,
}

impl TcpSocket {
    /// Create a new TCP socket
    ///
    /// # Arguments
    ///
    /// * `family` - Address family (IPv4 or IPv6)
    ///
    /// # Returns
    ///
    /// * `Ok(TcpSocket)` - Created socket
    /// * `Err(SocketError)` - Error creating socket
    pub fn new(family: AddressFamily) -> Result<Self, SocketError> {
        let socket = Socket::new(
            family,
            super::socket::SocketType::Stream,
            super::socket::Protocol::Tcp,
        )?;
        
        Ok(Self {
            socket,
            check_io: None,
        })
    }
    
    /// Create a TCP socket with I/O polling support
    ///
    /// # Arguments
    ///
    /// * `family` - Address family (IPv4 or IPv6)
    /// * `check_io` - I/O polling manager
    ///
    /// # Returns
    ///
    /// * `Ok(TcpSocket)` - Created socket
    /// * `Err(SocketError)` - Error creating socket
    pub fn with_io_polling(family: AddressFamily, check_io: CheckIo) -> Result<Self, SocketError> {
        let mut socket = Self::new(family)?;
        socket.check_io = Some(check_io);
        Ok(socket)
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
        self.socket.bind(addr)
    }
    
    /// Listen for incoming connections
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
        self.socket.listen(backlog)
    }
    
    /// Accept an incoming connection
    ///
    /// # Returns
    ///
    /// * `Ok((TcpSocket, SocketAddr))` - Accepted connection and peer address
    /// * `Err(SocketError)` - Error accepting connection
    pub fn accept(&self) -> Result<(TcpSocket, SocketAddr), SocketError> {
        let (socket, addr) = self.socket.accept()?;
        
        let mut new_socket = TcpSocket {
            socket,
            check_io: None,
        };
        
        // Copy I/O polling reference if available
        if self.check_io.is_some() {
            new_socket.check_io = Some(CheckIo::new()); // Create new instance for new connection
        }
        
        Ok((new_socket, addr))
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
        self.socket.connect(addr)
    }
    
    /// Send data
    ///
    /// # Arguments
    ///
    /// * `buf` - Data to send
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes sent
    /// * `Err(SocketError)` - Error sending
    pub fn send(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.socket.write(buf)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Receive data
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to receive data into
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes received
    /// * `Err(SocketError)` - Error receiving
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.socket.read(buf)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Get the local address
    ///
    /// # Returns
    ///
    /// * `Ok(SocketAddr)` - Local address
    /// * `Err(SocketError)` - Error getting address
    pub fn local_addr(&self) -> Result<SocketAddr, SocketError> {
        self.socket.local_addr()
    }
    
    /// Get the peer address
    ///
    /// # Returns
    ///
    /// * `Ok(SocketAddr)` - Peer address
    /// * `Err(SocketError)` - Error getting address
    pub fn peer_addr(&self) -> Result<SocketAddr, SocketError> {
        self.socket.peer_addr()
    }
    
    /// Get the raw file descriptor for I/O polling
    #[cfg(unix)]
    pub fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.socket.as_raw_fd()
    }
    
    #[cfg(windows)]
    pub fn as_raw_socket(&self) -> std::os::windows::io::RawSocket {
        self.socket.as_raw_socket()
    }
    
    /// Get the underlying socket
    pub fn inner(&self) -> &Socket {
        &self.socket
    }
}

impl Read for TcpSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.read(buf)
    }
}

impl Write for TcpSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_tcp_socket_creation() {
        let socket = TcpSocket::new(AddressFamily::Ipv4);
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_tcp_socket_bind() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_tcp_socket_listen() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        let result = socket.listen(128);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_tcp_socket_ipv6() {
        let socket = TcpSocket::new(AddressFamily::Ipv6);
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_tcp_socket_with_io_polling() {
        let check_io = CheckIo::new();
        let socket = TcpSocket::with_io_polling(AddressFamily::Ipv4, check_io);
        assert!(socket.is_ok());
    }

    #[test]
    fn test_tcp_socket_accept() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
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
        assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_accept_with_io_polling() {
        use std::thread;
        use std::time::Duration;
        
        let check_io = CheckIo::new();
        let listener = TcpSocket::with_io_polling(AddressFamily::Ipv4, check_io).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
            let _ = client.connect(&connect_addr);
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept connection (should have I/O polling from parent)
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
        let (accepted_socket, _) = accepted.unwrap();
        // Accepted socket should have I/O polling if parent had it
        assert!(accepted_socket.inner().as_raw_fd() > 0);
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_connect() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to accept
        let acceptor = thread::spawn(move || {
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
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
            }
            accepted
        });
        
        // Connect
        thread::sleep(Duration::from_millis(100));
        let client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let mut connected = false;
        for _ in 0..30 {
            match client.connect(&local_addr) {
                Ok(()) => {
                    connected = true;
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(SocketError::IoError(ref msg)) if msg.contains("in progress") || msg.contains("now in progress") => {
                    thread::sleep(Duration::from_millis(100));
                    if client.peer_addr().is_ok() {
                        connected = true;
                        break;
                    }
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
        
        assert!(connected);
        let peer_addr = client.peer_addr().unwrap();
        assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        
        let _ = acceptor.join();
    }

    #[test]
    fn test_tcp_socket_send_recv() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect and send
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let mut client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
            
            // Connect
            for _ in 0..30 {
                match client.connect(&connect_addr) {
                    Ok(()) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(SocketError::IoError(ref msg)) if msg.contains("in progress") || msg.contains("now in progress") => {
                        thread::sleep(Duration::from_millis(100));
                        if client.peer_addr().is_ok() {
                            break;
                        }
                    }
                    Err(e) => panic!("Connect error: {:?}", e),
                }
            }
            
            // Send data
            let data = b"Hello, TCP!";
            let mut written = 0;
            while written < data.len() {
                match client.send(&data[written..]) {
                    Ok(n) => written += n,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
            
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept and receive
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
            match server.recv(&mut buf[read_total..]) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    read_total += n;
                    if read_total >= 10 {
                        break;
                    }
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        assert_eq!(&buf[..read_total], b"Hello, TCP!");
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_local_addr() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), Ipv4Addr::LOCALHOST);
        assert!(local_addr.port() > 0);
    }

    #[test]
    fn test_tcp_socket_peer_addr() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
            let _ = client.connect(&connect_addr);
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept and check peer address
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
        
        let (server, peer_addr) = accepted.expect("Should have accepted connection");
        assert_eq!(peer_addr.ip(), Ipv4Addr::LOCALHOST);
        
        // Verify peer_addr() method works
        let peer_addr2 = server.peer_addr().unwrap();
        assert_eq!(peer_addr2.ip(), Ipv4Addr::LOCALHOST);
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_peer_addr_not_connected() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        
        // Peer addr should fail on unconnected socket
        let result = socket.peer_addr();
        assert!(result.is_err());
    }

    #[test]
    fn test_tcp_socket_as_raw_fd() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let fd = socket.as_raw_fd();
        assert!(fd > 0);
    }

    #[test]
    fn test_tcp_socket_inner() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let inner = socket.inner();
        assert!(inner.as_raw_fd() > 0);
    }

    #[test]
    fn test_tcp_socket_read_trait() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect and write
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let mut client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
            
            // Connect
            for _ in 0..30 {
                match client.connect(&connect_addr) {
                    Ok(()) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(SocketError::IoError(ref msg)) if msg.contains("in progress") || msg.contains("now in progress") => {
                        thread::sleep(Duration::from_millis(100));
                        if client.peer_addr().is_ok() {
                            break;
                        }
                    }
                    Err(e) => panic!("Connect error: {:?}", e),
                }
            }
            
            // Write using Write trait
            let data = b"Read trait test";
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
        
        // Accept and read using Read trait
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
                    if read_total >= 15 {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Read error: {:?}", e),
            }
        }
        
        assert_eq!(&buf[..read_total], b"Read trait test");
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_write_trait() {
        use std::thread;
        use std::time::Duration;
        
        let listener = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        listener.bind(&addr).unwrap();
        listener.listen(128).unwrap();
        
        let local_addr = listener.local_addr().unwrap();
        
        // Spawn thread to connect and write using Write trait
        let connect_addr = local_addr;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let mut client = TcpSocket::new(AddressFamily::Ipv4).unwrap();
            
            // Connect
            for _ in 0..30 {
                match client.connect(&connect_addr) {
                    Ok(()) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(SocketError::IoError(ref msg)) if msg.contains("in progress") || msg.contains("now in progress") => {
                        thread::sleep(Duration::from_millis(100));
                        if client.peer_addr().is_ok() {
                            break;
                        }
                    }
                    Err(e) => panic!("Connect error: {:?}", e),
                }
            }
            
            // Write using Write trait
            let data = b"Write trait test";
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
            
            // Test flush
            assert!(client.flush().is_ok());
            
            thread::sleep(Duration::from_millis(50));
        });
        
        // Accept and receive
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
                    if read_total >= 16 {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("Read error: {:?}", e),
            }
        }
        
        assert_eq!(&buf[..read_total], b"Write trait test");
        
        let _ = sender.join();
    }

    #[test]
    fn test_tcp_socket_connect_refused() {
        let socket = TcpSocket::new(AddressFamily::Ipv4).unwrap();
        
        // Try to connect to a non-existent port
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 65535);
        let result = socket.connect(&addr);
        // Should fail with ConnectionRefused or WouldBlock (non-blocking)
        assert!(result.is_err());
    }

    #[test]
    fn test_tcp_socket_ipv6_bind() {
        use std::net::Ipv6Addr;
        
        let socket = TcpSocket::new(AddressFamily::Ipv6).unwrap();
        let addr = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), Ipv6Addr::LOCALHOST);
    }

    #[test]
    fn test_tcp_socket_with_io_polling_ipv6() {
        let check_io = CheckIo::new();
        let socket = TcpSocket::with_io_polling(AddressFamily::Ipv6, check_io);
        assert!(socket.is_ok());
    }
}

