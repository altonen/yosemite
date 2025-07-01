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

#![cfg(all(not(feature = "sync"), any(feature = "tokio", feature = "smol")))]

use crate::{
    options::SessionOptions,
    style::{private, SessionStyle, Subsession},
};

#[cfg(feature = "tokio")]
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[cfg(feature = "smol")]
use smol::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use std::future::Future;

/// Virtual streams.
pub struct Stream {
    /// TCP stream used to communicate with router.
    ///
    /// `None` if the object is part of a primary session.
    stream: Option<BufReader<TcpStream>>,

    /// Session options.
    _options: SessionOptions,

    /// Socket that was sent the forwarding request, if any.
    _forwarding_stream: Option<TcpStream>,
}

impl Stream {
    /// Store the TCP used to send the forwarding command into [`Stream`]'s context.
    pub(crate) fn store_forwarded(&mut self, stream: TcpStream) {
        self._forwarding_stream = Some(stream);
    }
}

impl private::SessionStyle for Stream {
    fn new(_options: SessionOptions) -> impl Future<Output = crate::Result<Self>>
    where
        Self: Sized,
    {
        async {
            Ok(Self {
                stream: Some(BufReader::new(
                    TcpStream::connect(format!("127.0.0.1:{}", _options.samv3_tcp_port)).await?,
                )),
                _options,
                _forwarding_stream: None,
            })
        }
    }

    fn write_command(&mut self, command: &[u8]) -> impl Future<Output = crate::Result<()>> {
        async {
            match &mut self.stream {
                None => unreachable!(),
                Some(stream) => stream.write_all(command).await.map_err(From::from),
            }
        }
    }

    fn read_command(&mut self) -> impl Future<Output = crate::Result<String>> {
        async {
            match &mut self.stream {
                None => unreachable!(),
                Some(stream) => {
                    let mut response = String::new();

                    stream.read_line(&mut response).await.map(|_| response).map_err(From::from)
                }
            }
        }
    }

    fn create_session(&self) -> private::SessionParameters {
        private::SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        }
    }
}

impl SessionStyle for Stream {}

impl private::Subsession for Stream {
    fn new(_options: SessionOptions) -> impl Future<Output = crate::Result<Self>>
    where
        Self: Sized,
    {
        async {
            Ok(Self {
                stream: None,
                _options,
                _forwarding_stream: None,
            })
        }
    }
}

impl Subsession for Stream {}
