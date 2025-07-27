// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![doc = include_str!("../README.md")]

#[cfg(all(feature = "sync", feature = "tokio"))]
compile_error!("feature \"sync\" and feature \"tokio\" cannot be enabled at the same time");

#[cfg(all(feature = "sync", feature = "smol"))]
compile_error!("feature \"sync\" and feature \"smol\" cannot be enabled at the same time");

mod error;
mod options;
mod proto;

pub use error::{Error, I2pError, ProtocolError};
pub use options::{DatagramOptions, DestinationKind, SessionOptions, StreamOptions};

#[cfg(any(feature = "tokio", feature = "smol"))]
mod asynchronous;

#[cfg(all(not(feature = "sync"), any(feature = "tokio", feature = "smol")))]
pub use {
    asynchronous::router::RouterApi,
    asynchronous::session::{style, Session},
    asynchronous::stream::Stream,
};

#[cfg(feature = "sync")]
mod synchronous;

#[cfg(all(feature = "sync", not(any(feature = "tokio", feature = "smol"))))]
pub use {
    synchronous::router::RouterApi,
    synchronous::session::{style, Session},
    synchronous::stream::Stream,
};

/// Result type of the crate.
pub type Result<T> = core::result::Result<T, error::Error>;
