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
        let mut controller = StreamController::new(options.clone()).unwrap();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port))?;

        // perform handshake
        let mut stream = {
            stream.write_all("HELLO VERSION\n".as_bytes())?;

            {
                let mut stream = BufReader::new(stream);
                let mut response = String::new();
                stream.read_line(&mut response)?;

                println!("response: {response}");

                stream.into_inner()
            }
        };

        // create streaming session with transient destination
        let session_stream = {
            stream.write_all(
                format!(
                "SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT i2cp.leaseSetEncType=4\n",
                options.nickname
            )
                .as_bytes(),
            )?;

            {
                let mut stream = BufReader::new(stream);
                let mut response = String::new();
                stream.read_line(&mut response)?;

                println!("response: {response}");

                stream.into_inner()
            }
        };

        // attempt to establish connection to `destination`
        let stream = {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", options.samv3_tcp_port))?;

            stream.write_all("HELLO VERSION\n".as_bytes())?;

            let mut stream = {
                let mut stream = BufReader::new(stream);
                let mut response = String::new();
                stream.read_line(&mut response)?;

                println!("response: {response}");

                stream.into_inner()
            };

            stream.write_all(
                format!(
                    "STREAM CONNECT ID={} DESTINATION={} SILENT=false\n",
                    options.nickname, destination
                )
                .as_bytes(),
            )?;

            {
                let mut stream = BufReader::new(stream);
                let mut response = String::new();
                stream.read_line(&mut response)?;

                println!("response: {response}");

                stream.into_inner()
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_stream_sync() {
        let mut stream = Stream::new(String::from("host.i2p"), StreamOptions::default()).unwrap();

        stream.write_all("GET / HTTP/1.1\r\nHost: host.i2p\r\nUser-Agent: Mozilla/5.0\r\nAccept: text/html\r\n\r\n".as_bytes()).unwrap();

        let mut buffer = vec![0u8; 8192];

        let nread = stream.read(&mut buffer).unwrap();

        println!("{:?}", std::str::from_utf8(&buffer[..nread]));

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
