use std::{
    ffi::OsString,
    io, mem,
    os::fd::{AsRawFd, RawFd},
    str::FromStr,
};

use libc::{sockaddr, sockaddr_ll, AF_PACKET, ETH_P_ALL};
use nix::{
    errno::Errno,
    net::if_::if_nametoindex,
    sys::socket::{bind, setsockopt, sockopt::BindToDevice, LinkAddr, SockaddrLike},
};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{unix::AsyncFd, Interest};

#[derive(Debug)]
pub enum RCapError {
    IOError(io::Error),
    NixError(Errno),
}
pub type RCapResult<T> = Result<T, RCapError>;

pub struct Capture {
    socket: Socket,
}
impl Capture {
    pub fn new(iface: &str) -> RCapResult<Self> {
        // The docs around this are terrible.
        // Parameters were found through trial and error and strace.
        let socket = Socket::new_raw(
            Domain::PACKET,
            Type::RAW,
            Some(Protocol::from((ETH_P_ALL as u16).to_be() as i32)),
        )
        .map_err(RCapError::IOError)?;

        // This is infallible.
        let iface_os_string = OsString::from_str(iface).unwrap();
        setsockopt(&socket, BindToDevice, &iface_os_string).map_err(RCapError::NixError)?;

        // TODO: Make a PR for nix to make this safe.
        let if_index = if_nametoindex(iface).map_err(RCapError::NixError)?;

        let mut sockaddr: sockaddr_ll = unsafe { mem::zeroed() };

        sockaddr.sll_family = AF_PACKET as _;
        sockaddr.sll_protocol = (ETH_P_ALL as u16).to_be();
        sockaddr.sll_ifindex = if_index as _;

        let sockaddr = unsafe {
            LinkAddr::from_raw(
                &sockaddr as *const _ as *const sockaddr,
                Some(mem::size_of::<sockaddr_ll>() as u32),
            )
        }
        .unwrap();
        bind(socket.as_raw_fd(), &sockaddr).map_err(RCapError::NixError)?;

        Ok(Self { socket })
    }
    pub fn get_inner(&self) -> &Socket {
        &self.socket
    }
    pub fn get_inner_mut(&mut self) -> &mut Socket {
        &mut self.socket
    }
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(unsafe { mem::transmute(buf) })
    }
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }
}
impl AsRawFd for Capture {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}
pub struct AsyncCapture {
    capture: AsyncFd<Capture>,
}
impl AsyncCapture {
    pub fn new(iface: &str) -> RCapResult<Self> {
        let sync_capture = Capture::new(iface)?;

        Ok(Self {
            capture: AsyncFd::new(sync_capture).unwrap(),
        })
    }
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::READABLE, |inner| inner.recv(buf))
            .await
    }
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::WRITABLE, |inner| inner.send(buf))
            .await
    }
}
