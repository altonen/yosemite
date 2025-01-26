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

//! Asynchronous SAMv3 session.

use crate::{
    asynchronous::{session::style::SessionStyle, stream::Stream},
    error::{Error, ProtocolError},
    options::SessionOptions,
    proto::session::SessionController,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::TcpStream,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use std::{future::Future, str};

pub mod style;

/// ### SAMv3 session.
///
/// `SessionStyle` defines the protocol of the session and can be one of three types:
///  * [`Stream`](style::Stream): virtual streams
///  * [`Repliable`](style::Repliable): repliable datagrams
///  * [`Anonymous`](style::Anonymous): anonymous datagrams
///
/// Each session style enables a set of APIs that can be used to interact with remote destinations
/// over that protocol.
///
/// ### Virtual streams
///
/// The virtual stream API allows to establish outbound connections and accept inbound connections,
/// either directly using [`Session::accept()`] or by forwarding to an active TCP listener using
/// [`Session::forward()`]. The stream APIs return opaque [`Stream`] objects which implement
/// [`AsyncRead`](futures::AsyncRead) and[`AsyncWrite`](futures::AsyncWrite) traits.
///
/// **Connecting to remote destination and exchanging data with them**
///
/// ```no_run
/// use yosemite::{Session, style::Stream};
/// use futures::{AsyncReadExt, AsyncWriteExt};
///
/// #[tokio::main]
/// async fn main() -> yosemite::Result<()> {
///     let mut session = Session::<Stream>::new(Default::default()).await?;
///     let mut stream = session.connect("host.i2p").await?;
///     let mut buffer = vec![0u8; 64];
///
///     stream.write_all(b"hello, world/\n").await?;
///     stream.read_exact(&mut buffer).await?;
///
///     Ok(())
/// }
/// ```
///
/// ### Repliable datagrams
///
/// The repliable datagram API allow sending datagrams which the remote destination can reply to as
/// the sender's destination is sent alongside the datagram.
///
/// **Echo server**
///
/// ```no_run
/// use yosemite::{Session, style::Repliable};
/// use futures::{AsyncReadExt, AsyncWriteExt};
///
/// #[tokio::main]
/// async fn main() -> yosemite::Result<()> {
///     let mut session = Session::<Repliable>::new(Default::default()).await?;
///     let mut buffer = vec![0u8; 64];
///
///     while let Ok((nread, destination)) = session.recv_from(&mut buffer).await {
///         session.send_to(&mut buffer[..nread], &destination).await?;
///     }
///
///     Ok(())
/// }
/// ```
///
/// ### Anonymous datagrams
///
/// The anonymous datagram API allow sending raw datagrams to remote destination. The destination of
/// the sender is not sent alongside the datagram so the remote destination is not able to reply to
/// these datagrams.
///
/// ```no_run
/// use yosemite::{RouterApi, Session, style::Anonymous};
/// use futures::{AsyncReadExt, AsyncWriteExt};
///
/// #[tokio::main]
/// async fn main() -> yosemite::Result<()> {
///     let mut session = Session::<Anonymous>::new(Default::default()).await?;
///     let destination = RouterApi::default().lookup_name("datagram_server.i2p").await?;
///     let mut buffer = vec![0u8; 64];
///
///     for i in 0..5 {
///         session.send_to(&[i as u8; 64], &destination).await?;
///     }
///
///     Ok(())
/// }
/// ```
///
/// See [examples](https://github.com/altonen/yosemite/tree/master/examples) for more details on how to use `yosemite`.
pub struct Session<S> {
    /// Session controller.
    controller: SessionController,

    /// Session options.
    options: SessionOptions,

    /// Context for session style.
    context: S,
}

impl<S: SessionStyle> Session<S> {
    /// Create new [`Session`].
    ///
    /// See [`SessionOptions`] for more details on how to configure the session.
    pub async fn new(options: SessionOptions) -> crate::Result<Self> {
        let mut controller = SessionController::new(options.clone())?;
        let mut context = S::new(options.clone()).await?;

        // send handhake to router
        let command = controller.handshake_session()?;
        context.write_command(&command).await?;

        // read handshake response and create new session
        let response = context.read_command().await?;
        controller.handle_response(&response)?;

        // create new session
        let command = controller.create_session(context.create_session())?;
        context.write_command(&command).await?;

        // read handshake response and create new session
        let response = context.read_command().await?;
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
    ///
    /// Destination can
    ///  * hostname such as `host.i2p`
    ///  * base32-encoded session received from
    ///    [`RouterApi::lookup_name()`](crate::RouterApi::lookup_name)
    ///  * base64-encoded string received from, e.g., [`Session::new()`]
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

    #[cfg(feature = "async-extra")]
    pub fn connect_detached(
        &mut self,
        destination: &str,
    ) -> impl Future<Output = crate::Result<Stream>> {
        let mut controller = self.controller.clone();
        let sam_tcp_port = self.options.samv3_tcp_port;
        let destination = destination.to_owned();

        async move {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", sam_tcp_port)).await?;

            let command = controller.handshake_stream()?;
            stream.write_all(&command).await?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            let command = controller.create_stream(&destination)?;
            stream.write_all(&command).await?;

            let (stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
            let stream = TokioAsyncWriteCompatExt::compat_write(compat);

            Ok(Stream::from_stream(stream, destination.to_string()))
        }
    }

    /// Accept inbound virtual stream.
    ///
    /// The function call will fail if [`Session::forward()`] has been called before.
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
                        break str::from_utf8(&response[..newline])
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
    ///
    /// The function call will fail if [`Session::accept()`] has been called before.
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

        // store the command stream into the session context so the router keeps forwarding streams
        style::Stream::store_forwarded(&mut self.context, stream);

        Ok(())
    }
}

impl Session<style::Repliable> {
    /// Send data on the socket to given `destination`.
    pub async fn send_to(&mut self, buf: &[u8], destination: &str) -> crate::Result<()> {
        style::Repliable::send_to(&mut self.context, buf, destination).await
    }

    /// Receive a single datagram on the socket.
    ///
    /// `buf` must be of sufficient size to hold the entire datagram.
    ///
    /// Returns the number of bytes read and the destination who sent the datagram.
    pub async fn recv_from(&mut self, buf: &mut [u8]) -> crate::Result<(usize, String)> {
        style::Repliable::recv_from(&mut self.context, buf).await
    }
}

impl Session<style::Anonymous> {
    /// Send data on the socket to given `destination`.
    pub async fn send_to(&mut self, buf: &[u8], destination: &str) -> crate::Result<()> {
        style::Anonymous::send_to(&mut self.context, buf, destination).await
    }

    /// Receive a single datagram on the socket.
    ///
    /// `buf` must be of sufficient size to hold the entire datagram.
    ///
    /// Returns the number of bytes read.
    pub async fn recv(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        style::Anonymous::recv(&mut self.context, buf).await
    }
}
