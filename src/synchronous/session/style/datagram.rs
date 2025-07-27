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

#![cfg(all(feature = "sync", not(any(feature = "tokio", feature = "smol"))))]

use crate::{
    options::{DatagramOptions, SessionOptions},
    style::{private, SessionStyle, Subsession},
    Error,
};

use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
};

/// Repliable datagrams.
pub struct Repliable {
    /// Read buffer
    buffer: Vec<u8>,

    /// Session options.
    options: SessionOptions,

    /// Server UDP address.
    server_address: SocketAddr,

    /// Datagram socket.
    socket: UdpSocket,

    /// TCP stream used to communicate with the router.
    ///
    /// `None` if the object is part of a primary session.
    stream: Option<BufReader<TcpStream>>,
}

impl Repliable {
    pub(crate) fn send_to(&mut self, buf: &[u8], destination: &str) -> crate::Result<()> {
        let mut datagram =
            format!("3.0 {} {}\n", self.options.nickname, destination).as_bytes().to_vec();
        datagram.extend_from_slice(buf);

        self.socket
            .send_to(&datagram, &self.server_address)
            .map(|_| ())
            .map_err(From::from)
    }

    pub(crate) fn send_to_with_options(
        &mut self,
        buf: &[u8],
        destination: &str,
        options: DatagramOptions,
    ) -> crate::Result<()> {
        let mut datagram = format!(
            "3.0 {} {} {} {} {} {} {}\n",
            self.options.nickname,
            destination,
            options.from_port,
            options.to_port,
            options.send_tags,
            options.tag_threshold,
            options.send_lease_set,
        )
        .as_bytes()
        .to_vec();
        datagram.extend_from_slice(buf);

        self.socket
            .send_to(&datagram, &self.server_address)
            .map(|_| ())
            .map_err(From::from)
    }

    pub(crate) fn recv_from(&mut self, buf: &mut [u8]) -> crate::Result<(usize, String)> {
        let nread = self.socket.recv(&mut self.buffer)?;

        let destination = {
            let destination_end =
                self.buffer[..nread].iter().position(|byte| byte == &b' ').unwrap();

            std::str::from_utf8(&self.buffer[..destination_end])
                .map_err(|_| Error::Malformed)?
                .to_owned()
        };

        let nread = {
            let header_end = self.buffer[..nread].iter().position(|byte| byte == &b'\n').unwrap();
            let datagram_len = nread - header_end - 1;
            buf[..datagram_len].copy_from_slice(&self.buffer[header_end + 1..nread]);

            datagram_len
        };

        Ok((nread, destination))
    }
}

impl private::SessionStyle for Repliable {
    fn new(options: SessionOptions) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", options.datagram_port))?;
        let stream = BufReader::new(TcpStream::connect(format!(
            "127.0.0.1:{}",
            options.samv3_tcp_port
        ))?);
        let server_address =
            format!("127.0.0.1:{}", options.samv3_udp_port).parse().expect("to succeed");

        Ok(Self {
            buffer: vec![0u8; 0xfff],
            options,
            server_address,
            socket,
            stream: Some(stream),
        })
    }

    fn write_command(&mut self, command: &[u8]) -> crate::Result<()> {
        match &mut self.stream {
            None => unreachable!(),
            Some(stream) => stream.get_mut().write_all(command).map_err(From::from),
        }
    }

    fn read_command(&mut self) -> crate::Result<String> {
        let mut response = String::new();

        match &mut self.stream {
            None => unreachable!(),
            Some(stream) => stream.read_line(&mut response).map(|_| response).map_err(From::from),
        }
    }

    fn create_session(&self) -> private::SessionParameters {
        let port = self.socket.local_addr().expect("to succeed").port();

        private::SessionParameters {
            style: "DATAGRAM".to_string(),
            options: Vec::from_iter([
                ("PORT".to_string(), port.to_string()),
                ("HOST".to_string(), "127.0.0.1".to_string()),
            ]),
        }
    }
}

impl SessionStyle for Repliable {}

impl private::Subsession for Repliable {
    fn new(options: SessionOptions) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", options.datagram_port))?;
        let server_address =
            format!("127.0.0.1:{}", options.samv3_udp_port).parse().expect("to succeed");

        Ok(Self {
            buffer: vec![0u8; 0xfff],
            options,
            server_address,
            socket,
            stream: None,
        })
    }
}

impl Subsession for Repliable {}

/// Anonymous datagrams.
pub struct Anonymous {
    /// Session options.
    options: SessionOptions,

    /// Server UDP address.
    server_address: SocketAddr,

    /// Datagram socket.
    socket: UdpSocket,

    /// TCP stream used to communicate with the router.
    ///
    /// `None` if the object is part of a primary session.
    stream: Option<BufReader<TcpStream>>,
}

impl Anonymous {
    pub(crate) fn send_to(&mut self, buf: &[u8], destination: &str) -> crate::Result<()> {
        let mut datagram =
            format!("3.0 {} {}\n", self.options.nickname, destination).as_bytes().to_vec();
        datagram.extend_from_slice(buf);

        self.socket
            .send_to(&datagram, &self.server_address)
            .map(|_| ())
            .map_err(From::from)
    }

    pub(crate) fn send_to_with_options(
        &mut self,
        buf: &[u8],
        destination: &str,
        options: DatagramOptions,
    ) -> crate::Result<()> {
        let mut datagram = format!(
            "3.0 {} {} {} {} {} {} {} {}\n",
            self.options.nickname,
            destination,
            options.from_port,
            options.to_port,
            options.protocol,
            options.send_tags,
            options.tag_threshold,
            options.send_lease_set,
        )
        .as_bytes()
        .to_vec();
        datagram.extend_from_slice(buf);

        self.socket
            .send_to(&datagram, &self.server_address)
            .map(|_| ())
            .map_err(From::from)
    }

    pub(crate) fn recv(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        self.socket.recv(buf).map_err(From::from)
    }
}

impl private::SessionStyle for Anonymous {
    fn new(options: SessionOptions) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", options.datagram_port))?;
        let stream = BufReader::new(TcpStream::connect(format!(
            "127.0.0.1:{}",
            options.samv3_tcp_port
        ))?);
        let server_address =
            format!("127.0.0.1:{}", options.samv3_udp_port).parse().expect("to succeed");

        Ok(Self {
            options,
            server_address,
            socket,
            stream: Some(stream),
        })
    }

    fn write_command(&mut self, command: &[u8]) -> crate::Result<()> {
        match &mut self.stream {
            None => unreachable!(),
            Some(stream) => stream.get_mut().write_all(command).map_err(From::from),
        }
    }

    fn read_command(&mut self) -> crate::Result<String> {
        let mut response = String::new();

        match &mut self.stream {
            None => unreachable!(),
            Some(stream) => stream.read_line(&mut response).map(|_| response).map_err(From::from),
        }
    }

    fn create_session(&self) -> private::SessionParameters {
        let port = self.socket.local_addr().expect("to succeed").port();

        private::SessionParameters {
            style: "RAW".to_string(),
            options: Vec::from_iter([
                ("PORT".to_string(), port.to_string()),
                ("HOST".to_string(), "127.0.0.1".to_string()),
            ]),
        }
    }
}

impl SessionStyle for Anonymous {}

impl private::Subsession for Anonymous {
    fn new(options: SessionOptions) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", options.datagram_port))?;
        let server_address =
            format!("127.0.0.1:{}", options.samv3_udp_port).parse().expect("to succeed");

        Ok(Self {
            options,
            server_address,
            socket,
            stream: None,
        })
    }
}

impl Subsession for Anonymous {}
