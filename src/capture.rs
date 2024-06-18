use std::io;

pub trait Capture {
    #[cfg(feature = "async")]
    type AsyncCaptureType: AsyncCapture;

    fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;
    fn send(&self, buf: &[u8]) -> io::Result<usize>;

    #[cfg(feature = "async")]
    fn to_async(self) -> Self::AsyncCaptureType;
}
#[cfg(feature = "async")]
pub trait AsyncCapture {
    fn recv_async(&self, buf: &mut [u8]) -> impl std::future::Future<Output = io::Result<usize>>;
    fn send_async(&self, buf: &[u8]) -> impl std::future::Future<Output = io::Result<usize>>;
}
