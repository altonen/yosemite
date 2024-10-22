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

#![cfg(feature = "async")]

use crate::{proto::stream::StreamController, StreamOptions};

use futures::{AsyncRead, AsyncWrite};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use std::{
    pin::Pin,
    task::{Context, Poll},
};

macro_rules! read_response {
    ($stream:expr) => {{
        let mut reader = BufReader::new($stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        (reader.into_inner(), response)
    }};
}

/// Virtual stream kind.
enum StreamKind {
    /// Incoming virtual stream.
    Incoming,

    /// Outgoing virtual stream.
    Outgoing {
        /// TCP stream that was used to create the session.
        session_stream: TcpStream,

        /// Stream controller.
        controller: StreamController,
    },
}

/// Asynchronous I2P virtual stream.
pub struct Stream {
    /// Stream kind.
    kind: StreamKind,

    /// Stream options.
    options: StreamOptions,

    /// Data stream.
    stream: Compat<TcpStream>,
}

impl Stream {
    /// Create new [`Stream`] with `options`.
    pub async fn new(destination: String, options: StreamOptions) -> crate::Result<Self> {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;
        let mut controller = StreamController::new(options.clone()).unwrap();

        // send handhake to router
        let command = controller.handshake_session()?;
        stream.write_all(&command).await?;

        // read handshake response and create new session
        let (mut stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // create transient session
        let command = controller.create_transient_session()?;
        stream.write_all(&command).await?;

        // read handshake response and create new session
        let (session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // session has been created, create new virtual stream
        let stream = {
            let mut stream =
                TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;
            let command = controller.handshake_stream()?;
            stream.write_all(&command).await?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            let command = controller.create_stream(&destination)?;
            stream.write_all(&command).await?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            stream
        };

        let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
        let stream = TokioAsyncWriteCompatExt::compat_write(compat);

        Ok(Self {
            kind: StreamKind::Outgoing {
                session_stream,
                controller,
            },
            options,
            stream,
        })
    }

    /// Create new [`Stream`] from an inbound connection.
    pub(crate) fn from_stream(stream: Compat<TcpStream>, options: StreamOptions) -> Self {
        Self {
            kind: StreamKind::Incoming,
            options,
            stream,
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        std::pin::pin!(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        std::pin::pin!(&mut self.stream)
            .as_mut()
            .poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        std::pin::pin!(&mut self.stream)
            .as_mut()
            .poll_write_vectored(cx, bufs)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        std::pin::pin!(&mut self.stream).as_mut().poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        std::pin::pin!(&mut self.stream).poll_close(cx)
    }
}
