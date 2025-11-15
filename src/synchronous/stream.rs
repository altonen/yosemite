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

#![cfg(feature = "sync")]

use std::{
    io::{Read, Write},
    net::TcpStream,
};

/// Read half of [`Stream`].
pub struct ReadHalf {
    stream: TcpStream,
}

/// Write half of [`Stream`].
pub struct WriteHalf {
    stream: TcpStream,
}

/// Synchronous virtual stream.
pub struct Stream {
    /// Data stream.
    stream: TcpStream,

    /// Remote destination.
    remote_destination: String,
}

impl Stream {
    /// Create new [`Stream`] from an inbound connection.
    pub(crate) fn from_stream(stream: TcpStream, remote_destination: String) -> Self {
        Self {
            stream,
            remote_destination,
        }
    }

    /// Get reference to remote destination.
    pub fn remote_destination(&self) -> &str {
        &self.remote_destination
    }

    /// Split [`Stream`] into independent read and write halves.
    ///
    /// Returns `None` if cloning the underlying socket failed.
    pub fn split(self) -> Option<(ReadHalf, WriteHalf)> {
        let write = self.stream.try_clone().ok()?;

        Some((
            ReadHalf {
                stream: self.stream,
            },
            WriteHalf { stream: write },
        ))
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Read for ReadHalf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.stream.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stream.write_fmt(fmt)
    }
}

impl Write for WriteHalf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.stream.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stream.write_fmt(fmt)
    }
}
