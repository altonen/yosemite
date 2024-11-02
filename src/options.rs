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

use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

/// Default port for UDP.
pub(crate) const SAMV3_UDP_PORT: u16 = 7655;

/// Default port for TCP.
pub(crate) const SAMV3_TCP_PORT: u16 = 7656;

/// Session options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionOptions {
    /// Nickname.
    ///
    /// Name that uniquely identifies the session.
    ///
    /// If not specified, `yosemite` generates a random alphanmeric nickname.
    pub nickname: String,

    /// TCP port of the listening SAMv3 server.
    ///
    /// Defaults to `7656`.
    pub samv3_tcp_port: u16,

    /// Should `STREAM FORWARD` be silent.
    ///
    /// If set to false (default), the first message read from the TCP stream accepted by the TCP
    /// listener where incoming streams are forwarded to is destination of the remote peer.
    ///
    /// If the application where incoming streams should be forwarded to isn't expecting a
    /// destination to be read from the socket, the forwarded stream can be set to silent. This
    /// means, however, that destination of the connecting peer cannot be recovered.
    pub silent_forward: bool,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            nickname: Alphanumeric.sample_string(&mut thread_rng(), 16),
            samv3_tcp_port: SAMV3_TCP_PORT,
            silent_forward: false,
        }
    }
}

/// Stream options.
//
// TODO: these should actually be stream options, i.e., `i2cp.streaming`
#[derive(Debug, Clone)]
pub struct StreamOptions {
    /// Nickname.
    ///
    /// Name that uniquely identifies the session.
    ///
    /// If not specified, `yosemite` generates a random alphanmeric nickname.
    pub nickname: String,

    /// Port where the stream socket should be bound.
    ///
    /// By default the stream socket is bound to random port assigned by the OS.
    pub port: u16,

    /// TCP port of the listening SAMv3 server.
    ///
    /// Defaults to `7656`.
    pub samv3_tcp_port: u16,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            nickname: Alphanumeric.sample_string(&mut thread_rng(), 16),
            port: 0u16,
            samv3_tcp_port: SAMV3_TCP_PORT,
        }
    }
}

/// Datagram options.
#[derive(Debug)]
pub struct DatagramOptions {
    /// Port where the datagram should be bound to, if any.
    ///
    /// By default the socket is not bound to any port.
    pub port: Option<u16>,

    /// UDP port of the listening SAMv3 server.
    ///
    /// Defaults to `7655`
    pub samv3_udp_port: u16,
}

impl Default for DatagramOptions {
    fn default() -> Self {
        Self {
            port: None,
            samv3_udp_port: SAMV3_UDP_PORT,
        }
    }
}
