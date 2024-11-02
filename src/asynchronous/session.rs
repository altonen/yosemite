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
    asynchronous::stream::Stream,
    error::{Error, ProtocolError},
    options::SessionOptions,
    proto::session::SessionController,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::TcpStream,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// Asynchronous I2P session.
pub struct Session {
    /// Session controller.
    controller: SessionController,

    /// Session options.
    options: SessionOptions,

    /// Controller stream.
    _session_stream: TcpStream,

    /// Socket that was sent the forwarding request, if any.
    _forwarding_stream: Option<TcpStream>,
}

impl Session {
    /// Create new [`Session`].
    pub async fn new(options: SessionOptions) -> crate::Result<Self> {
        let mut controller = SessionController::new(options.clone())?;
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;

        // send handhake to router
        let command = controller.handshake_session()?;
        stream.write_all(&command).await?;

        // read handshake response and create new session
        let (mut stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // create new session
        let command = controller.create_session()?;
        stream.write_all(&command).await?;

        // read handshake response and create new session
        let (_session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        Ok(Self {
            controller,
            options,
            _session_stream,
            _forwarding_stream: None,
        })
    }

    /// Create new outbound virtual stream to `destination`.
    pub async fn connect(&mut self, destination: &str) -> crate::Result<Stream> {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port)).await?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command).await?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.create_stream(&destination)?;
        stream.write_all(&command).await?;

        let (stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
        let stream = TokioAsyncWriteCompatExt::compat_write(compat);

        Ok(Stream::from_stream(stream, destination.to_string()))
    }

    /// Accept inbound virtual stream.
    pub async fn accept(&mut self) -> crate::Result<Stream> {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port)).await?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command).await?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.accept_stream()?;
        stream.write_all(&command).await?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        // read accept response from the socket
        //
        // the server may have bundled data after the newline but that should not be read by this
        // function as it's inteded for the client to read
        let response = {
            let mut response = [0u8; 1024];

            let destination = loop {
                let ready = stream.ready(Interest::READABLE).await?;

                if ready.is_readable() {
                    let nread = stream.peek(&mut response).await?;

                    if let Some(newline) = response[..nread].iter().position(|c| c == &b'\n') {
                        let _ = stream.read_exact(&mut response[..newline + 1]).await?;
                        break std::str::from_utf8(&response[..newline])
                            .map_err(|_| Error::Protocol(ProtocolError::InvalidMessage))?
                            .to_string();
                    }
                }
            };

            destination
        };

        let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
        let stream = TokioAsyncWriteCompatExt::compat_write(compat);

        Ok(Stream::from_stream(stream, response.to_string()))
    }

    /// Forward inbound virtual streams to a TCP listener at `port`.
    pub async fn forward(&mut self, port: u16) -> crate::Result<()> {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", self.options.samv3_tcp_port)).await?;
        let command = self.controller.handshake_stream()?;
        stream.write_all(&command).await?;

        let (mut stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;

        let command = self.controller.forward_stream(port)?;
        stream.write_all(&command).await?;

        let (stream, response) = read_response!(stream);
        self.controller.handle_response(&response)?;
        self._forwarding_stream = Some(stream);

        Ok(())
    }

    /// Get destination of the [`Session`].
    pub fn destination(&self) -> &str {
        self.controller.destination()
    }
}
