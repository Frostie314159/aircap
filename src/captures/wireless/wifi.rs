use crate::{
    captures::raw_socket_capture::{AsyncRawSocketCapture, RawSocketCapture}, AirCapResult, AsyncCapture, Capture
};

pub struct WiFiCapture {
    pub(crate) inner: RawSocketCapture,
}
impl WiFiCapture {
    pub fn new(interface: impl AsRef<str>) -> AirCapResult<Self> {
        Ok(Self {
            inner: RawSocketCapture::new(interface)?,
        })
    }
    pub const fn get_inner(&self) -> &RawSocketCapture {
        &self.inner
    }
    pub fn get_inner_mut(&mut self) -> &mut RawSocketCapture {
        &mut self.inner
    }
}
impl Capture for WiFiCapture {
    #[cfg(feature = "async")]
    type AsyncCaptureType = AsyncWiFiCapture;

    fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.recv(buf)
    }
    fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.send(buf)
    }

    #[cfg(feature = "async")]
    fn to_async(self) -> Self::AsyncCaptureType {
        AsyncWiFiCapture::with_sync_capture(self)
    }
}

#[cfg(feature = "async")]
pub struct AsyncWiFiCapture {
    pub(crate) inner: AsyncRawSocketCapture,
}
#[cfg(feature = "async")]
impl AsyncWiFiCapture {
    pub fn with_sync_capture(sync_capture: WiFiCapture) -> Self {
        AsyncWiFiCapture {
            inner: sync_capture.inner.to_async(),
        }
    }
    pub const fn get_inner(&self) -> &AsyncRawSocketCapture {
        &self.inner
    }
    pub fn get_inner_mut(&mut self) -> &mut AsyncRawSocketCapture {
        &mut self.inner
    }
}
#[cfg(feature = "async")]
impl AsyncCapture for AsyncWiFiCapture {
    fn recv_async(&self, buf: &mut [u8]) -> impl std::future::Future<Output = std::io::Result<usize>> {
        self.inner.recv_async(buf)
    }
    fn send_async(&self, buf: &[u8]) -> impl std::future::Future<Output = std::io::Result<usize>> {
        self.inner.send_async(buf)
    }
}
