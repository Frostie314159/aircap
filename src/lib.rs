use std::io;

use nix::errno::Errno;

mod capture;
pub use capture::*;
pub mod captures;

#[derive(Debug)]
pub enum AirCapError {
    IOError(io::Error),
    NixError(Errno),
}
pub type AirCapResult<T> = Result<T, AirCapError>;
