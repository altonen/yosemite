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
pub struct RouterApi {}

impl RouterApi {
    /// Attempt to look up the the destination associated with `name`.
    //
    // TODO: allow specifying port?
    pub fn lookup_name(name: &str) -> crate::Result<String> {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{SAMV3_TCP_PORT}"))?;

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
    //
    // TODO: allow specifying port?
    pub fn generate_destination() -> crate::Result<(String, String)> {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{SAMV3_TCP_PORT}"))?;

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
