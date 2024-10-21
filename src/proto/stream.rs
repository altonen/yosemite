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

use crate::{error::ProtocolError, StreamOptions};

/// State machine for SAMv3 virtual streams.
pub struct StreamController {
    /// Stream options.
    options: StreamOptions,
}

impl StreamController {
    /// Create new [`StreamController`] from `options`.
    pub fn new(options: StreamOptions) -> Result<Self, ProtocolError> {
        Ok(Self { options })
    }

    /// Create session.
    pub fn create_session(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake() {
        // HELLO REPLY RESULT=OK VERSION=3.3

        // SESSION STATUS RESULT=OK DESTINATION=TIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJmRZ8D0ewvPmy2QKbhZTS3Y9B~nR2m~2vf3yPdVWR7pokR0PeHn-vQ8Av0VNEKUete3L7pEvwrm8CxrIY2aUkV~CpNliKwvhfsJe7tSDSL32Ia42O45KTZbGkI9jvKDdFblwoOYpcd1ToDFZ5qWQ0bxACistfpu609-1Tw1y26neAAAA08XrilOIapGsMhNO1WihrFDLOycxcJlTlqbhV1NKKgekUa-RjUuL1n2hx7VjQK2iSK4FNUprfsr1GEIrOvaNKUD4B0fc7Xshbr43oZZ-LE0FxhNdOhz5KOEzW-eqE7V84PTWIfpY9to6Mm1JObl6ARHhVxPvSVQzkNMuuoFQoB2STMOw2osPXxr7tk~qVYnBrrHpZYrfGIyO1tN1MDCJPqTbFaCNb3Jtnxz3h7B~aJFAHzzEl~sHpMJx7IWAaVr-e2mIRin7fywJq3IhuPy8DdAJiIa-8qrjDDrNNg02a3BgSN4If6sTFooGRX-cXnuCjbbqjzg3dq8parcTekauEFtlTl6d17wFQ3o~JtFQ4ObzpGuW

        // STREAM STATUS RESULT=OK
        // STREAM STATUS RESULT=CANT_REACH_PEER MESSAGE="Connection failed"
    }
}
