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

#![cfg(all(feature = "sync", not(feature = "async")))]

use crate::{
    options::SessionOptions,
    style::{private, SessionStyle},
    DestinationKind,
};

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

/// Stream.
pub struct Stream {
    /// TCP stream used to communicate with router.
    stream: BufReader<TcpStream>,

    /// Session options.
    options: SessionOptions,

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
    fn new(options: SessionOptions) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            stream: BufReader::new(TcpStream::connect(format!(
                "127.0.0.1:{}",
                options.samv3_tcp_port
            ))?),
            options,
            _forwarding_stream: None,
        })
    }

    fn write_command(&mut self, command: &[u8]) -> crate::Result<()> {
        self.stream.get_mut().write_all(command).map_err(From::from)
    }

    fn read_command(&mut self) -> crate::Result<String> {
        let mut response = String::new();

        self.stream.read_line(&mut response).map(|_| response).map_err(From::from)
    }

    fn create_session(&self) -> String {
        match &self.options.destination {
            DestinationKind::Transient => format!(
                "SESSION CREATE \
                        STYLE=STREAM \
                        ID={} \
                        DESTINATION=TRANSIENT \
                        SIGNATURE_TYPE=7 \
                        i2cp.leaseSetEncType=4\n",
                self.options.nickname
            ),
            DestinationKind::Persistent { private_key } => format!(
                "SESSION CREATE \
                        STYLE=STREAM \
                        ID={} \
                        DESTINATION={private_key} \
                        SIGNATURE_TYPE=7 \
                        i2cp.leaseSetEncType=4\n",
                self.options.nickname
            ),
        }
    }
}

impl SessionStyle for Stream {}
