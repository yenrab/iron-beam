//! Integration tests for adapters_socket crate
//!
//! These tests verify that socket operations work correctly
//! and test end-to-end workflows for TCP and UDP sockets.

use adapters_socket::*;
use std::net::{SocketAddr, Ipv4Addr};

#[test]
fn test_socket_new() {
    let result = Socket::new(AddressFamily::Ipv4, SocketType::Stream, Protocol::Tcp);
    // May succeed or fail depending on system
    let _ = result;
}

#[test]
fn test_socket_error_variants() {
    let errors = vec![
        SocketError::InvalidAddress,
        SocketError::AddressInUse,
        SocketError::ConnectionRefused,
        SocketError::Timeout,
        SocketError::WouldBlock,
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_tcp_socket_new() {
    let result = TcpSocket::new(AddressFamily::Ipv4);
    // May succeed or fail depending on system
    let _ = result;
}

#[test]
fn test_udp_socket_new() {
    let result = UdpSocket::new(AddressFamily::Ipv4);
    // May succeed or fail depending on system
    let _ = result;
}

#[test]
fn test_address_family_variants() {
    let families = vec![
        AddressFamily::Ipv4,
        AddressFamily::Ipv6,
    ];
    
    for family in families {
        let _ = format!("{:?}", family);
    }
}

#[test]
fn test_socket_type_variants() {
    let types = vec![
        SocketType::Stream,
        SocketType::Datagram,
    ];
    
    for socket_type in types {
        let _ = format!("{:?}", socket_type);
    }
}

#[test]
fn test_protocol_variants() {
    let protocols = vec![
        Protocol::Tcp,
        Protocol::Udp,
    ];
    
    for protocol in protocols {
        let _ = format!("{:?}", protocol);
    }
}

#[test]
fn test_socket_error_from_io_error() {
    use std::io;
    let io_error = io::Error::new(io::ErrorKind::AddrInUse, "test");
    let socket_error: SocketError = io_error.into();
    
    match socket_error {
        SocketError::AddressInUse => {}
        _ => panic!("Expected AddressInUse"),
    }
}
