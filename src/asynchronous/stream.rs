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

/// Asynchronous I2P virtual stream.
pub struct Stream {
    /// TCP stream that was used to create the session.
    session_stream: TcpStream,

    /// Data stream.
    stream: Compat<TcpStream>,

    /// Stream options.
    options: StreamOptions,

    /// Stream controller.
    controller: StreamController,
}

impl Stream {
    /// Create new [`Stream`] with `options`.
    pub async fn new(destination: String, options: StreamOptions) -> crate::Result<Self> {
        let mut controller = StreamController::new(options.clone()).unwrap();
        let mut stream =
            TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // perform handshake
        {
            writer.write_all("HELLO VERSION\n".as_bytes()).await?;

            let mut response = String::new();
            reader.read_line(&mut response).await?;

            println!("response: {response}");
        }

        // create streaming session with transient destination
        {
            writer
                .write_all(
                    format!(
                "SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT i2cp.leaseSetEncType=4\n",
                options.nickname
            )
                    .as_bytes(),
                )
                .await?;

            let mut response = String::new();
            reader.read_line(&mut response).await?;

            println!("response: {response}");
        }

        let reader = reader.into_inner();
        let session_stream = reader.reunite(writer).expect("to succeed");

        // attempt to establish connection to `destination`
        let stream = {
            let mut stream =
                TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port)).await?;

            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);

            writer.write_all("HELLO VERSION\n".as_bytes()).await?;

            let mut response = String::new();
            reader.read_line(&mut response).await?;

            println!("response: {response}");

            writer
                .write_all(
                    format!(
                        "STREAM CONNECT ID={} DESTINATION={} SILENT=false\n",
                        options.nickname, destination
                    )
                    .as_bytes(),
                )
                .await?;

            let mut response = String::new();
            reader.read_line(&mut response).await?;

            println!("response: {response}");

            let reader = reader.into_inner();
            reader.reunite(writer).expect("to succeed")
        };

        let compat = TokioAsyncReadCompatExt::compat(stream).into_inner();
        let stream = TokioAsyncWriteCompatExt::compat_write(compat);

        Ok(Self {
            session_stream,
            stream,
            options,
            controller,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn create_stream_async() {
        let mut stream = Stream::new(String::from("host.i2p"), StreamOptions::default())
            .await
            .unwrap();

        stream.write_all("GET / HTTP/1.1\r\nHost: host.i2p\r\nUser-Agent: Mozilla/5.0\r\nAccept: text/html\r\n\r\n".as_bytes()).await.unwrap();

        let mut buffer = vec![0u8; 8192];

        let nread = stream.read(&mut buffer).await.unwrap();

        println!("{:?}", std::str::from_utf8(&buffer[..nread]));

        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
