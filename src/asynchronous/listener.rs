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

use crate::{asynchronous::stream::Stream, proto::listener::ListenerController, StreamOptions};

use futures::{future::BoxFuture, FutureExt};
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

/// Asynchronous stream listener.
pub struct Listener {
    /// TCP stream that was used to create the session.
    session_stream: TcpStream,

    /// Stream options.
    options: StreamOptions,

    /// Stream listener controller.
    controller: ListenerController,

    /// Connection future.
    future: BoxFuture<'static, crate::Result<Stream>>,
}

impl Listener {
    /// Create new [`Listener`] with `options`.
    pub async fn new(options: StreamOptions) -> crate::Result<Self> {
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;
        let mut controller = ListenerController::new(options.clone()).unwrap();

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

        let mut inner_controller = controller.clone();
        let options_copy = options.clone();

        Ok(Self {
            session_stream,
            future: Self::accept_future(controller.clone(), options.clone()),
            options,
            controller,
        })
    }

    /// Get reference to [`Listener`]'s destination.
    pub fn destination(&self) -> &str {
        self.controller.destination()
    }

    fn accept_future(
        mut controller: ListenerController,
        options: StreamOptions,
    ) -> BoxFuture<'static, crate::Result<Stream>> {
        Box::pin(async move {
            let mut stream =
                TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;
            let command = controller.handshake_listener()?;
            stream.write_all(&command).await?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            let command = controller.accept_stream()?;
            stream.write_all(&command).await?;

            let (mut stream, response) = read_response!(stream);
            controller.handle_response(&response)?;

            // read remote's destination which signals that the connection is open
            //
            // TODO: store remote's destination somewhere maybe?
            let (mut stream, _response) = read_response!(stream);

            let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
            let stream = TokioAsyncWriteCompatExt::compat_write(compat);

            Ok(Stream::from_stream(stream, options))
        })
    }
}

impl futures::Stream for Listener {
    type Item = Stream;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures::ready!(self.future.poll_unpin(cx)) {
            Err(_) => Poll::Ready(None),
            Ok(stream) => {
                self.future = Self::accept_future(self.controller.clone(), self.options.clone());
                Poll::Ready(Some(stream))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
    use tracing_subscriber::prelude::*;

    #[tokio::test]
    async fn listener() {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .unwrap();

        let mut listener = Listener::new(Default::default()).await.unwrap();
        let destination = listener.destination().to_owned();

        tokio::spawn(async move {
            let mut counter = 1;

            while let Some(mut stream) = listener.next().await {
                let mut buffer = vec![0u8; 14];

                stream.read_exact(&mut buffer).await.unwrap();

                tracing::info!(
                    "listener {counter} read: {:?}",
                    std::str::from_utf8(&buffer)
                );

                stream
                    .write_all(format!("goodbye, world {counter}").as_bytes())
                    .await
                    .unwrap();

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                counter += 1;
            }
        });

        for i in 0..3 {
            let mut stream = Stream::new(destination.clone(), StreamOptions::default())
                .await
                .unwrap();

            stream
                .write_all(format!("hello, world {i}").as_bytes())
                .await
                .unwrap();

            let mut buffer = vec![0u8; 16];
            stream.read_exact(&mut buffer).await.unwrap();

            tracing::info!("stream {i} read: {:?}", std::str::from_utf8(&buffer));

            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}
