use crate::{
    captures::raw_socket_capture::{AsyncRawSocketCapture, RawSocketCapture},
    AirCapResult, AsyncCapture, Capture,
};

pub enum ChannelSpecification {
    TwentyMHz {
        channel: u8,
    },
    FourtyMHz {
        primary_channel: u8,
        secondary_channel_above: bool,
    },
}

/// A synchronous wifi capture.
pub struct WiFiCapture {
    pub(crate) inner: RawSocketCapture,
    pub(crate) interface: String,
}
impl WiFiCapture {
    pub fn new(interface: impl AsRef<str>) -> AirCapResult<Self> {
        Ok(Self {
            interface: interface.as_ref().to_string(),
            inner: RawSocketCapture::new(interface)?,
        })
    }
    pub const fn get_inner(&self) -> &RawSocketCapture {
        &self.inner
    }
    pub fn get_inner_mut(&mut self) -> &mut RawSocketCapture {
        &mut self.inner
    }
    pub fn set_channel(&self, channel_specification: ChannelSpecification) {
        let (channel_number, bandwidth_arg) = match channel_specification {
            ChannelSpecification::TwentyMHz { channel } => (channel, "HT20"),
            ChannelSpecification::FourtyMHz {
                primary_channel,
                secondary_channel_above,
            } => (
                primary_channel,
                if secondary_channel_above {
                    "HT40U"
                } else {
                    "HT40D"
                },
            ),
        };
        let _ = std::process::Command::new("iw")
            .args([
                "dev",
                self.interface.as_str(),
                "set",
                "channel",
                channel_number.to_string().as_str(),
                bandwidth_arg,
            ])
            .spawn();
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
/// An asynchronous wifi capture.
pub struct AsyncWiFiCapture {
    pub(crate) inner: AsyncRawSocketCapture,
    pub(crate) interface: String,
}
#[cfg(feature = "async")]
impl AsyncWiFiCapture {
    pub fn with_sync_capture(sync_capture: WiFiCapture) -> Self {
        AsyncWiFiCapture {
            inner: sync_capture.inner.to_async(),
            interface: sync_capture.interface
        }
    }
    pub const fn get_inner(&self) -> &AsyncRawSocketCapture {
        &self.inner
    }
    pub fn get_inner_mut(&mut self) -> &mut AsyncRawSocketCapture {
        &mut self.inner
    }
    pub async fn set_channel(&self, channel_specification: ChannelSpecification) {
        let (channel_number, bandwidth_arg) = match channel_specification {
            ChannelSpecification::TwentyMHz { channel } => (channel, "HT20"),
            ChannelSpecification::FourtyMHz {
                primary_channel,
                secondary_channel_above,
            } => (
                primary_channel,
                if secondary_channel_above {
                    "HT40U"
                } else {
                    "HT40D"
                },
            ),
        };
        let _ = tokio::process::Command::new("iw")
            .args([
                "dev",
                self.interface.as_str(),
                "set",
                "channel",
                channel_number.to_string().as_str(),
                bandwidth_arg,
            ])
            .spawn();
    }
}
#[cfg(feature = "async")]
impl AsyncCapture for AsyncWiFiCapture {
    fn recv_async(
        &self,
        buf: &mut [u8],
    ) -> impl std::future::Future<Output = std::io::Result<usize>> {
        self.inner.recv_async(buf)
    }
    fn send_async(&self, buf: &[u8]) -> impl std::future::Future<Output = std::io::Result<usize>> {
        self.inner.send_async(buf)
    }
}
