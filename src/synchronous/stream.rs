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

use crate::{proto::stream::StreamController, StreamOptions};

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    pin::Pin,
    task::{Context, Poll},
};

macro_rules! read_response {
    ($stream:expr) => {{
        let mut reader = BufReader::new($stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        (reader.into_inner(), response)
    }};
}

/// Synchronous I2P virtual stream.
pub struct Stream {
    /// TCP stream that was used to create the session.
    session_stream: TcpStream,

    /// Data stream.
    stream: TcpStream,

    /// Stream options.
    options: StreamOptions,

    /// Stream controller.
    controller: StreamController,
}

impl Stream {
    /// Create new [`Stream`] with `options`.
    pub fn new(destination: String, options: StreamOptions) -> crate::Result<Self> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port))?;
        let mut controller = StreamController::new(options.clone()).unwrap();

        // send handhake to router
        let command = controller.handshake_session()?;
        stream.write_all(&command)?;

        // read handshake response and create new session
        let (mut stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // create transient session
        let command = controller.create_transient_session()?;
        stream.write_all(&command)?;

        // read handshake response and create new session
        let (session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // session has been created, create new virtual stream
        let stream = {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port))?;
            let command = controller.handshake_stream()?;
            stream.write_all(&command)?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            let command = controller.create_stream(&destination)?;
            stream.write_all(&command)?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            stream
        };

        Ok(Self {
            session_stream,
            stream,
            options,
            controller,
        })
    }
}

impl Read for Stream {
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

    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stream.write_fmt(fmt)
    }
}
