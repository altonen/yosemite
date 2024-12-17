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

//! Asynchronous SAMv3 client library.
//!
//! ### [`Session`] object
//!
//! `SessionStyle` defines the protocol of the session and can be one of three types:
//!  * [`Stream`](style::Stream): virtual streams
//!  * [`Repliable`](style::Repliable): repliable datagrams
//!  * [`Anonymous`](style::Anonymous): anonymous datagrams
//!
//! Each session style enables a set of APIs that can be used to interact with remote destinations
//! over that protocol.
//!
//! ### Virtual streams
//!
//! Each style enables a set of APIs that can be used to interact with remote destinations. The
//! datagram APIs allow sending and receiving data and the stream API allows to establish outbound
//! connections and accept inbound connections, either directly using [`Session::accept()`] or by
//! forwarding to an active TCP listener using [`Session::forward()`]. The stream APIs return opaque
//! [`Stream`] objects which implement [`AsyncRead`](futures::AsyncRead)
//! and[`AsyncWrite`](futures::AsyncWrite) traits.
//!
//! **Connecting to remote destination**
//!
//! ```rust
//! let session = Session::<Stream>::new(Default::default()).await?;
//! let mut stream = session.connect("zzz.i2p").await?;
//!
//! let (nread, dest) = session.recv_from(&mut buf).await?;
//! session.send_to(&buf[..nread], &dest).await?;
//! ```
//!
//! ### Repliable datagrams
//!
//! ### Anonymous datagrams
//!
//! See [examples](https://github.com/altonen/yosemite/tree/master/examples) for more details on how to use `yosemite`.

#[cfg(all(feature = "sync", feature = "async"))]
compile_error!("feature \"sync\" and feature \"async\" cannot be enabled at the same time");

mod error;
mod options;
mod proto;

pub use error::{Error, I2pError, ProtocolError};
pub use options::{DestinationKind, SessionOptions};

#[cfg(feature = "async")]
mod asynchronous;

#[cfg(all(feature = "async", not(feature = "sync")))]
pub use {
    asynchronous::router::RouterApi,
    asynchronous::session::{style, Session},
    asynchronous::stream::Stream,
};

#[cfg(feature = "sync")]
mod synchronous;

#[cfg(all(feature = "sync", not(feature = "async")))]
pub use {
    synchronous::router::RouterApi,
    synchronous::session::{style, Session},
    synchronous::stream::Stream,
};

/// Result type of the crate.
pub type Result<T> = core::result::Result<T, error::Error>;
