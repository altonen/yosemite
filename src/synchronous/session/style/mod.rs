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

//! Session style.

#![cfg(all(feature = "sync", not(any(feature = "tokio", feature = "smol"))))]

pub use datagram::{Anonymous, Repliable};
pub use primary::Primary;
pub use stream::Stream;

mod datagram;
mod primary;
mod stream;

pub(crate) mod private {
    /// Session parameters.
    pub struct SessionParameters {
        /// Session style.
        pub(crate) style: String,

        /// Session options.
        pub(crate) options: Vec<(String, String)>,
    }

    pub trait SessionStyle {
        /// Create new `SessionStyle` object.
        fn new(options: crate::options::SessionOptions) -> crate::Result<Self>
        where
            Self: Sized;

        /// Send command to router.
        fn write_command(&mut self, command: &[u8]) -> crate::Result<()>;

        /// Read command from router.
        fn read_command(&mut self) -> crate::Result<String>;

        /// Get `SESSION CREATE` command for this session style.
        fn create_session(&self) -> SessionParameters;
    }

    pub trait Subsession: SessionStyle {
        /// Create new `SessionStyle` object for a subsession.
        ///
        /// This function doesn't establish a TCP connection to the router as the primary session
        /// already has an active connection.
        fn new(options: crate::options::SessionOptions) -> crate::Result<Self>
        where
            Self: Sized;
    }
}

/// Session style.
pub trait SessionStyle: private::SessionStyle {}

/// Subsession.
///
/// Implemented for `Stream`, `Anonymous` and `Datagram`.
pub trait Subsession: SessionStyle + private::Subsession {}
