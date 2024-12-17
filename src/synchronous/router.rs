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

use std::{io::Write, net::TcpStream};

use crate::{options::SAMV3_TCP_PORT, proto::router::RouterApiController};

/// Router API.
///
/// `RouterApi` provides SAM functionality unrelated to active sessions.
///
/// Lookup the the destination of a host name:
///
/// ```rust
/// let router_api = RouterApi::default().host_lookup("host.i2p").unwrap();
/// ```
///
/// Generate destination:
///
/// ```rust
/// let router_api = RouterApi::default().generate_destination().unwrap();
/// ```
///
/// `RouterApi` connects to the router via the default SAMV3 TCP port (7656) but this can be
/// overridden by calling [`RouterApi::new()`] with a custom port:
///
/// ```rust
/// let router_api = RouterApi::new(8888).generate_destination().unwrap();
/// ```
pub struct RouterApi {
    /// SAMv3 TCP port.
    port: u16,
}

impl Default for RouterApi {
    fn default() -> Self {
        Self {
            port: SAMV3_TCP_PORT,
        }
    }
}

impl RouterApi {
    /// Create new [`RouterApi`].
    ///
    /// `port` specifies the SAMv3 TCP port the router is listening on.
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl RouterApi {
    /// Attempt to look up the the destination associated with `name`.
    pub fn lookup_name(&self, name: &str) -> crate::Result<String> {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))?;

        // send handhake to router
        let command = controller.handshake_router_api()?;
        stream.write_all(&command)?;

        // read handshake response
        let (mut stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // lookup hostname
        let command = controller.lookup_name(name)?;
        stream.write_all(&command)?;

        // handle hostname lookup response
        let (_session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        Ok(controller.destination())
    }

    /// Generate destination.
    pub fn generate_destination(&self) -> crate::Result<(String, String)> {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))?;

        // send handhake to router
        let command = controller.handshake_router_api()?;
        stream.write_all(&command)?;

        // read handshake response
        let (mut stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        // generate destination
        let command = controller.generate_destination()?;
        stream.write_all(&command)?;

        // read destination generation response
        let (_session_stream, response) = read_response!(stream);
        controller.handle_response(&response)?;

        Ok(controller.generated_destination())
    }
}
