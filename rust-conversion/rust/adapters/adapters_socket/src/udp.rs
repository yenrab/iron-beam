//! UDP Socket Module
//!
//! Provides UDP (User Datagram Protocol) socket functionality for datagram-based
//! communication.

use super::socket::{Socket, SocketError, AddressFamily};
use std::net::SocketAddr;
use adapters_nif_io::CheckIo;

/// UDP Socket
///
/// Provides UDP socket operations with integration to the NIF I/O polling system.
pub struct UdpSocket {
    socket: Socket,
    check_io: Option<CheckIo>,
}

impl UdpSocket {
    /// Create a new UDP socket
    ///
    /// # Arguments
    ///
    /// * `family` - Address family (IPv4 or IPv6)
    ///
    /// # Returns
    ///
    /// * `Ok(UdpSocket)` - Created socket
    /// * `Err(SocketError)` - Error creating socket
    pub fn new(family: AddressFamily) -> Result<Self, SocketError> {
        let socket = Socket::new(
            family,
            super::socket::SocketType::Datagram,
            super::socket::Protocol::Udp,
        )?;
        
        Ok(Self {
            socket,
            check_io: None,
        })
    }
    
    /// Create a UDP socket with I/O polling support
    ///
    /// # Arguments
    ///
    /// * `family` - Address family (IPv4 or IPv6)
    /// * `check_io` - I/O polling manager
    ///
    /// # Returns
    ///
    /// * `Ok(UdpSocket)` - Created socket
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
    
    /// Send data to a remote address
    ///
    /// # Arguments
    ///
    /// * `buf` - Data to send
    /// * `addr` - Remote address to send to
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes sent
    /// * `Err(SocketError)` - Error sending
    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize, SocketError> {
        use socket2::SockAddr;
        let sock_addr = SockAddr::from(*addr);
        self.socket.inner().send_to(buf, &sock_addr)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Receive data from any address
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to receive data into
    ///
    /// # Returns
    ///
    /// * `Ok((usize, SocketAddr))` - Number of bytes received and sender address
    /// * `Err(SocketError)` - Error receiving
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), SocketError> {
        use std::mem::MaybeUninit;
        
        // Convert &mut [u8] to &mut [MaybeUninit<u8>]
        let uninit_buf: &mut [MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut MaybeUninit<u8>,
                buf.len()
            )
        };
        
        let (n, sock_addr) = self.socket.inner().recv_from(uninit_buf)
            .map_err(|e| SocketError::from(e))?;
        
        // Safety: recv_from initializes the first n bytes of the buffer
        // No need to zero - the data is already there from recv_from
        
        let addr = sock_addr.as_socket()
            .ok_or_else(|| SocketError::InvalidAddress)?;
        
        Ok((n, addr))
    }
    
    /// Connect to a remote address (for connected UDP sockets)
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
    
    /// Send data (for connected UDP sockets)
    ///
    /// # Arguments
    ///
    /// * `buf` - Data to send
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes sent
    /// * `Err(SocketError)` - Error sending
    pub fn send(&self, buf: &[u8]) -> Result<usize, SocketError> {
        self.socket.inner().send(buf)
            .map_err(|e| SocketError::from(e))
    }
    
    /// Receive data (for connected UDP sockets)
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to receive data into
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes received
    /// * `Err(SocketError)` - Error receiving
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, SocketError> {
        use std::mem::MaybeUninit;
        
        // Convert &mut [u8] to &mut [MaybeUninit<u8>]
        let uninit_buf: &mut [MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut MaybeUninit<u8>,
                buf.len()
            )
        };
        
        let n = self.socket.inner().recv(uninit_buf)
            .map_err(|e| SocketError::from(e))?;
        
        // Safety: recv initializes the first n bytes of the buffer
        // No need to zero - the data is already there from recv
        
        Ok(n)
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
    
    /// Get the peer address (for connected UDP sockets)
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
    pub fn as_raw_fd(&self) -> i32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_udp_socket_creation() {
        let socket = UdpSocket::new(AddressFamily::Ipv4);
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_udp_socket_bind() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let result = socket.bind(&addr);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_udp_socket_ipv6() {
        let socket = UdpSocket::new(AddressFamily::Ipv6);
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_udp_socket_with_io_polling() {
        let check_io = CheckIo::new();
        let socket = UdpSocket::with_io_polling(AddressFamily::Ipv4, check_io);
        assert!(socket.is_ok());
    }
    
    #[test]
    fn test_udp_socket_send_recv() {
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let _local_addr1 = socket1.local_addr().unwrap();
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Send data from socket1 to socket2
        let data = b"Hello, UDP!";
        let result = socket1.send_to(data, &local_addr2);
        // May fail with WouldBlock on non-blocking socket, which is expected
        assert!(result.is_ok() || matches!(result.unwrap_err(), SocketError::WouldBlock));
    }
    
    #[test]
    fn test_udp_send_to_recv_from_roundtrip() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr1 = socket1.local_addr().unwrap();
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Spawn thread to send data
        let send_addr = local_addr2;
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let data = b"Hello from socket1!";
            for _ in 0..20 {
                match socket1.send_to(data, &send_addr) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive on socket2
        let mut buf = vec![0u8; 1024];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv_from(&mut buf) {
                Ok((n, addr)) => {
                    assert_eq!(addr, local_addr1);
                    received = Some(buf[..n].to_vec());
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert_eq!(received, Some(b"Hello from socket1!".to_vec()));
    }
    
    #[test]
    fn test_udp_connected_mode() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr1 = socket1.local_addr().unwrap();
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Connect socket1 to socket2
        socket1.connect(&local_addr2).unwrap();
        
        // Verify peer address
        let peer_addr = socket1.peer_addr().unwrap();
        assert_eq!(peer_addr, local_addr2);
        
        // Spawn thread to send data using connected mode
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let data = b"Connected UDP message!";
            for _ in 0..20 {
                match socket1.send(data) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive on socket2
        let mut buf = vec![0u8; 1024];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv_from(&mut buf) {
                Ok((n, addr)) => {
                    assert_eq!(addr, local_addr1);
                    received = Some(buf[..n].to_vec());
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert_eq!(received, Some(b"Connected UDP message!".to_vec()));
    }
    
    #[test]
    fn test_udp_connected_recv() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr1 = socket1.local_addr().unwrap();
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Connect both sockets
        socket1.connect(&local_addr2).unwrap();
        socket2.connect(&local_addr1).unwrap();
        
        // Spawn thread to send data
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let data = b"Connected recv test!";
            for _ in 0..20 {
                match socket1.send(data) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive using connected mode recv
        let mut buf = vec![0u8; 1024];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv(&mut buf) {
                Ok(n) => {
                    received = Some(buf[..n].to_vec());
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert_eq!(received, Some(b"Connected recv test!".to_vec()));
    }
    
    #[test]
    fn test_udp_local_addr() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        let local_addr = socket.local_addr().unwrap();
        assert_eq!(local_addr.ip(), std::net::IpAddr::from(Ipv4Addr::LOCALHOST));
        assert!(local_addr.port() > 0);
    }
    
    #[test]
    fn test_udp_peer_addr_not_connected() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        // Peer addr should fail when not connected
        let result = socket.peer_addr();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_udp_peer_addr_connected() {
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr2 = socket2.local_addr().unwrap();
        socket1.connect(&local_addr2).unwrap();
        
        let peer_addr = socket1.peer_addr().unwrap();
        assert_eq!(peer_addr, local_addr2);
    }
    
    #[test]
    fn test_udp_ipv6_operations() {
        use std::net::Ipv6Addr;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv6).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv6).unwrap();
        
        let addr1 = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr1 = socket1.local_addr().unwrap();
        let local_addr2 = socket2.local_addr().unwrap();
        
        assert_eq!(local_addr1.ip(), std::net::IpAddr::from(Ipv6Addr::LOCALHOST));
        assert_eq!(local_addr2.ip(), std::net::IpAddr::from(Ipv6Addr::LOCALHOST));
    }
    
    #[test]
    fn test_udp_empty_packet() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Send empty packet
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let data = b"";
            for _ in 0..20 {
                match socket1.send_to(data, &local_addr2) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive empty packet
        let mut buf = vec![0u8; 1024];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv_from(&mut buf) {
                Ok((n, _)) => {
                    received = Some(n);
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert_eq!(received, Some(0));
    }
    
    #[test]
    fn test_udp_large_packet() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Send large packet (64KB)
        let large_data: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();
        let large_data_clone = large_data.clone();
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for _ in 0..20 {
                match socket1.send_to(&large_data_clone, &local_addr2) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive large packet
        let mut buf = vec![0u8; 70000];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv_from(&mut buf) {
                Ok((n, _)) => {
                    received = Some(buf[..n].to_vec());
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        if let Some(data) = received {
            assert_eq!(data.len(), 65536);
            assert_eq!(data, large_data);
        }
    }
    
    #[test]
    fn test_udp_inner() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let inner = socket.inner();
        // Inner socket may or may not have a local address when not bound
        // Just verify we can access it
        let _ = inner.local_addr();
    }
    
    #[cfg(unix)]
    #[test]
    fn test_udp_as_raw_fd() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let fd = socket.as_raw_fd();
        assert!(fd >= 0);
    }
    
    #[cfg(windows)]
    #[test]
    fn test_udp_as_raw_socket() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let raw = socket.as_raw_socket();
        assert!(raw != std::os::windows::io::INVALID_SOCKET);
    }
    
    #[test]
    fn test_udp_with_io_polling_check_io() {
        let check_io = CheckIo::new();
        let socket = UdpSocket::with_io_polling(AddressFamily::Ipv4, check_io).unwrap();
        // Verify socket was created successfully
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        assert!(socket.bind(&addr).is_ok());
    }
    
    #[test]
    fn test_udp_multiple_sends() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Send multiple packets
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for i in 0..5 {
                let data = format!("Packet {}", i).into_bytes();
                for _ in 0..20 {
                    match socket1.send_to(&data, &local_addr2) {
                        Ok(_) => break,
                        Err(SocketError::WouldBlock) => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => panic!("Send error: {:?}", e),
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        
        // Receive multiple packets
        let mut received = Vec::new();
        let mut buf = vec![0u8; 1024];
        for _ in 0..100 {
            match socket2.recv_from(&mut buf) {
                Ok((n, _)) => {
                    received.push(buf[..n].to_vec());
                    if received.len() >= 5 {
                        break;
                    }
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert_eq!(received.len(), 5);
    }
    
    #[test]
    fn test_udp_send_to_invalid_address() {
        let socket = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        socket.bind(&addr).unwrap();
        
        // Try to send to an unreachable address (may succeed or fail depending on system)
        let unreachable = SocketAddr::new(Ipv4Addr::new(192, 0, 2, 0).into(), 12345);
        let result = socket.send_to(b"test", &unreachable);
        // This may succeed (packet sent) or fail, both are valid
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_udp_recv_from_small_buffer() {
        use std::thread;
        use std::time::Duration;
        
        let socket1 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        let socket2 = UdpSocket::new(AddressFamily::Ipv4).unwrap();
        
        let addr1 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let addr2 = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        
        socket1.bind(&addr1).unwrap();
        socket2.bind(&addr2).unwrap();
        
        let local_addr2 = socket2.local_addr().unwrap();
        
        // Send large packet
        let large_data = vec![0u8; 1000];
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for _ in 0..20 {
                match socket1.send_to(&large_data, &local_addr2) {
                    Ok(_) => break,
                    Err(SocketError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("Send error: {:?}", e),
                }
            }
        });
        
        // Receive with small buffer (should truncate)
        let mut buf = vec![0u8; 100];
        let mut received = None;
        for _ in 0..20 {
            match socket2.recv_from(&mut buf) {
                Ok((n, _)) => {
                    assert_eq!(n, 100); // Truncated to buffer size
                    received = Some(buf[..n].to_vec());
                    break;
                }
                Err(SocketError::WouldBlock) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Recv error: {:?}", e),
            }
        }
        
        let _ = sender.join();
        assert!(received.is_some());
    }
}

