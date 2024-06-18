use std::{
    ffi::OsString,
    io, mem,
    ops::{Deref, DerefMut},
    os::fd::{AsRawFd, RawFd},
    str::FromStr,
};

use libc::{sockaddr, sockaddr_ll, AF_PACKET, ETH_P_ALL};
use nix::{
    net::if_::if_nametoindex,
    sys::socket::{bind, setsockopt, sockopt::BindToDevice, LinkAddr, SockaddrLike},
};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{unix::AsyncFd, Interest};

use crate::{AirCapError, AirCapResult, AsyncCapture, Capture};

/// A synchronous and blocking raw socket capture.
///
/// It's called a raw socket capture, since internally it contains a raw socket, which is bound to the requested interface.
pub struct RawSocketCapture {
    raw_socket: Socket,
}
impl RawSocketCapture {
    /// Instantiate a new raw socket capture.
    ///
    /// This creates the raw socket and sets all the required parameters.
    ///
    /// # Errors
    /// This should only fail, if the requested interface doesn't exist, or you don't have the required permissions.
    /// Any panic is a bug.
    pub fn new(interface: impl AsRef<str>) -> AirCapResult<Self> {
        // The docs around this are *terrible* on the linux side.
        // Parameters were found through trial and error, strace and reading the libpcap source code (or atleast attempting that, since it's absolute pasta).
        let raw_socket = Socket::new_raw(
            Domain::PACKET,
            Type::RAW,
            Some(Protocol::from((ETH_P_ALL as u16).to_be() as i32)),
        )
        .map_err(AirCapError::IOError)?;

        // This is infallible.
        let iface_os_string = OsString::from_str(interface.as_ref()).unwrap();
        setsockopt(&raw_socket, BindToDevice, &iface_os_string).map_err(AirCapError::NixError)?;

        let if_index = if_nametoindex(interface.as_ref()).map_err(AirCapError::NixError)?;

        // TODO: Make a PR for nix to make this safe.
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
        bind(raw_socket.as_raw_fd(), &sockaddr).map_err(AirCapError::NixError)?;

        Ok(Self { raw_socket })
    }
    pub fn get_inner(&self) -> &Socket {
        &self.raw_socket
    }
    pub fn get_inner_mut(&mut self) -> &mut Socket {
        &mut self.raw_socket
    }
}
impl Capture for RawSocketCapture {
    #[cfg(feature = "async")]
    type AsyncCaptureType = AsyncRawSocketCapture;

    fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.raw_socket.recv(unsafe { mem::transmute(buf) })
    }
    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.raw_socket.send(buf)
    }

    #[cfg(feature = "async")]
    fn to_async(self) -> Self::AsyncCaptureType {
        AsyncRawSocketCapture::with_sync_capture(self)
    }
}
impl AsRawFd for RawSocketCapture {
    fn as_raw_fd(&self) -> RawFd {
        self.raw_socket.as_raw_fd()
    }
}

#[cfg(feature = "async")]
/// An asynchronous raw socket capture.
///
/// This internally contains an [AsyncFd], which in turn contains a normal [RawSocketCapture].
pub struct AsyncRawSocketCapture {
    /// This is just the internal blocking capture.
    capture: AsyncFd<RawSocketCapture>,
}
#[cfg(feature = "async")]
impl AsyncRawSocketCapture {
    /// Instantiate a new async capture.
    ///
    /// # Errors
    /// This should only fail, if the requested interface doesn't exist, or you don't have the required permissions.
    /// Under Linux you either need to be superuser, or set the `CAP_NET_RAW` capability.
    pub fn new(interface: impl AsRef<str>) -> AirCapResult<Self> {
        // Suprise, it's a blocking capture in disguise.
        // I only noticed, this was a rhyme, after I wrote it.

        Ok(Self::with_sync_capture(RawSocketCapture::new(interface)?))
    }
    pub fn with_sync_capture(sync_capture: RawSocketCapture) -> Self {
        Self {
            capture: AsyncFd::new(sync_capture).unwrap(),
        }
    }
    /// Receive data from the capture asynchronously.
    pub async fn recv_async(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::READABLE, |inner| inner.recv(buf))
            .await
    }
    /// Send data to the capture asynchronously.
    pub async fn send_async(&self, buf: &[u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::WRITABLE, |inner| inner.send(buf))
            .await
    }
}
#[cfg(feature = "async")]
impl AsyncCapture for AsyncRawSocketCapture {
    async fn recv_async(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::READABLE, |inner| inner.recv(buf))
            .await
    }
    async fn send_async(&self, buf: &[u8]) -> io::Result<usize> {
        self.capture
            .async_io(Interest::WRITABLE, |inner| inner.send(buf))
            .await
    }
}
#[cfg(feature = "async")]
impl Deref for AsyncRawSocketCapture {
    type Target = RawSocketCapture;
    fn deref(&self) -> &Self::Target {
        self.capture.get_ref()
    }
}
#[cfg(feature = "async")]
impl DerefMut for AsyncRawSocketCapture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.capture.get_mut()
    }
}
