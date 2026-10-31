// src/lib.rs
pub mod error;
pub mod proto;
pub mod service;

use crate::error::Result;
use std::net::Ipv4Addr;
use std::net::UdpSocket as StdUdpSocket;
use tokio::net::UdpSocket;

/// Build a Tokio UdpSocket from a configured std::net::UdpSocket.
/// Useful when we must tune options (broadcast, reuse, join_multicast).
pub fn udp_from_std(std: StdUdpSocket) -> Result<UdpSocket> {
    std.set_nonblocking(true)?;
    Ok(UdpSocket::from_std(std)?)
}

/// Create a UDP socket with broadcast enabled bound to address
pub fn make_broadcast_socket(bind_addr: &str) -> Result<UdpSocket> {
    let std = StdUdpSocket::bind(bind_addr)?;
    std.set_broadcast(true)?;
    std.set_nonblocking(true)?;
    Ok(UdpSocket::from_std(std)?)
}

/// Create a multicast socket, bind to the given interface addr (0.0.0.0 usually)
/// and join the multicast group `group`.
pub fn make_multicast_socket(
    bind_addr: &str,
    group: Ipv4Addr,
    interface: Ipv4Addr,
) -> Result<UdpSocket> {
    let std = StdUdpSocket::bind(bind_addr)?;
    std.set_nonblocking(true)?;
    std.join_multicast_v4(&group, &interface)?;
    Ok(UdpSocket::from_std(std)?)
}
