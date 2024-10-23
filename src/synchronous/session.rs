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

use crate::{
    options::SessionOptions, proto::session::SessionController, synchronous::stream::Stream,
};

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

macro_rules! read_response {
    ($stream:expr) => {{
        let mut reader = BufReader::new($stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        (reader.into_inner(), response)
    }};
}

/// Synchronous I2P session.
pub struct Session {
    /// Session controller.
    controller: SessionController,

    /// Session options.
    options: SessionOptions,

    /// Controller stream.
    _session_stream: TcpStream,
}

impl Session {
    /// Create new [`Session`].
    pub fn new(options: SessionOptions) -> crate::Result<Self> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port))?;
        let mut controller = SessionController::new(options.clone()).unwrap();

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
        let (_session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        Ok(Session {
            controller,
            options,
            _session_stream,
        })
    }

    /// Create new outbound virtual stream to `destination`.
    pub fn connect(&mut self, destination: &str) -> crate::Result<Stream> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port))?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command)?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.create_stream(&destination)?;
        stream.write_all(&command)?;

        let (stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        Ok(Stream::from_stream(stream, destination.to_string()))
    }

    /// Accept inbound virtual stream.
    pub fn accept(&mut self) -> crate::Result<Stream> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port))?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command)?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.accept_stream()?;
        stream.write_all(&command)?;

        let (stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        // read remote's destination which signals that the connection is open
        let (stream, response) = read_response!(stream);

        Ok(Stream::from_stream(stream, response.to_string()))
    }

    /// Get destination of the [`Session`].
    pub fn destination(&self) -> &str {
        self.controller.destination()
    }
}
