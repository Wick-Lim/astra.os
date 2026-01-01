// Network stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/net.rs

use crate::fmt;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::sys::unsupported;
use crate::time::Duration;

// TODO: Integrate with kernel's smoltcp network stack
// For now, all network operations return "unsupported"

#[derive(Debug)]
pub struct TcpStream;

impl TcpStream {
    pub fn connect(_addr: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        unsupported!()
    }

    pub fn connect_timeout(_addr: &SocketAddr, _timeout: Duration) -> io::Result<TcpStream> {
        unsupported!()
    }

    pub fn set_read_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        unsupported!()
    }

    pub fn set_write_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        unsupported!()
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported!()
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported!()
    }

    pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn write(&self, _buf: &[u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        unsupported!()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unsupported!()
    }

    pub fn shutdown(&self, _how: Shutdown) -> io::Result<()> {
        unsupported!()
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        unsupported!()
    }

    pub fn set_linger(&self, _linger: Option<Duration>) -> io::Result<()> {
        unsupported!()
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        unsupported!()
    }

    pub fn set_nodelay(&self, _nodelay: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        unsupported!()
    }

    pub fn set_ttl(&self, _ttl: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unsupported!()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        unsupported!()
    }
}

#[derive(Debug)]
pub struct TcpListener;

impl TcpListener {
    pub fn bind(_addr: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        unsupported!()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unsupported!()
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        unsupported!()
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        unsupported!()
    }

    pub fn set_ttl(&self, _ttl: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unsupported!()
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        unsupported!()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        unsupported!()
    }
}

#[derive(Debug)]
pub struct UdpSocket;

impl UdpSocket {
    pub fn bind(_addr: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        unsupported!()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        unsupported!()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unsupported!()
    }

    pub fn recv_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        unsupported!()
    }

    pub fn peek_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        unsupported!()
    }

    pub fn send_to(&self, _buf: &[u8], _addr: &SocketAddr) -> io::Result<usize> {
        unsupported!()
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        unsupported!()
    }

    pub fn set_read_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        unsupported!()
    }

    pub fn set_write_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        unsupported!()
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported!()
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported!()
    }

    pub fn set_broadcast(&self, _broadcast: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        unsupported!()
    }

    pub fn set_multicast_loop_v4(&self, _multicast_loop_v4: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        unsupported!()
    }

    pub fn set_multicast_ttl_v4(&self, _multicast_ttl_v4: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        unsupported!()
    }

    pub fn set_multicast_loop_v6(&self, _multicast_loop_v6: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        unsupported!()
    }

    pub fn join_multicast_v4(&self, _multiaddr: &Ipv4Addr, _interface: &Ipv4Addr) -> io::Result<()> {
        unsupported!()
    }

    pub fn join_multicast_v6(&self, _multiaddr: &Ipv6Addr, _interface: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn leave_multicast_v4(&self, _multiaddr: &Ipv4Addr, _interface: &Ipv4Addr) -> io::Result<()> {
        unsupported!()
    }

    pub fn leave_multicast_v6(&self, _multiaddr: &Ipv6Addr, _interface: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn set_ttl(&self, _ttl: u32) -> io::Result<()> {
        unsupported!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unsupported!()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        unsupported!()
    }

    pub fn recv(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn send(&self, _buf: &[u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn connect(&self, _addr: io::Result<&SocketAddr>) -> io::Result<()> {
        unsupported!()
    }
}

pub fn lookup_host(_host: &str) -> io::Result<LookupHost> {
    unsupported!()
}

pub struct LookupHost;

impl Iterator for LookupHost {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
