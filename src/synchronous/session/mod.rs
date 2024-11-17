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
    options::SessionOptions, proto::session::SessionController, style::SessionStyle,
    synchronous::stream::Stream,
};

use std::{io::Write, net::TcpStream};

pub mod style;

/// Synchronous I2P session.
pub struct Session<S> {
    /// Session controller.
    controller: SessionController,

    /// Session options.
    options: SessionOptions,

    /// Session style context.
    context: S,
}

impl<S: SessionStyle> Session<S> {
    /// Create new [`Session`].
    pub fn new(options: SessionOptions) -> crate::Result<Self> {
        let mut controller = SessionController::new(options.clone())?;
        let mut context = S::new(options.clone())?;

        // send handhake to router
        let command = controller.handshake_session()?;
        context.write_command(&command)?;

        // read handshake response and create new session
        let response = context.read_command()?;
        controller.handle_response(&response)?;

        // create new session
        let command = controller.create_session(context.create_session())?;
        context.write_command(&command)?;

        // read handshake response and create new session
        let response = context.read_command()?;
        controller.handle_response(&response)?;

        Ok(Self {
            controller,
            options,
            context,
        })
    }

    /// Get destination of the [`Session`].
    pub fn destination(&self) -> &str {
        self.controller.destination()
    }
}

impl Session<style::Stream> {
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

    /// Forward inbound virtual streams to a TCP listener at `port`.
    pub fn forward(&mut self, port: u16) -> crate::Result<()> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port))?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command)?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.forward_stream(port)?;
        stream.write_all(&command)?;

        let (stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        // store the command stream into the session context so the router keeps forwarding streams
        style::Stream::store_forwarded(&mut self.context, stream);

        Ok(())
    }
}
