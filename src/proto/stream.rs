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

use std::mem;

/// Logging target for the file.
const LOG_TARGET: &str = "yosemite::proto::stream";

/// Stream state.
#[derive(Debug, PartialEq, Eq)]
enum StreamState {
    /// Stream state is uninitialized.
    Uninitialized,

    /// Stream is being handshaked.
    Handshaking,

    /// Stream has been handshaked.
    Handshaked,

    /// `STREAM CONNECT` is pending.
    ConnectPending,

    /// Stream is active.
    Active,
}

/// Session state.
#[derive(Debug, PartialEq, Eq)]
enum SessionState {
    /// Session is uninitialized.
    Uninitialized,

    /// Handshake has been sent to router.
    Handshaking,

    /// Session has been handshaked.
    Handshaked,

    /// `SESSION CREATE` message has been sent.
    SessionCreatePending,

    /// Session is active.
    Active {
        /// Created destination.
        destination: String,

        /// Virtual stream state.
        stream_state: StreamState,
    },

    /// Session state has been poisoned.
    Poisoned,
}

/// State machine for SAMv3 virtual streams.
pub struct StreamController {
    /// Stream options.
    options: StreamOptions,

    /// Stream state.
    state: SessionState,
}

impl StreamController {
    /// Create new [`StreamController`] from `options`.
    pub fn new(options: StreamOptions) -> Result<Self, ProtocolError> {
        Ok(Self {
            options,
            state: SessionState::Uninitialized,
        })
    }

    /// Initialize new session by handshaking with the router.
    pub fn handshake_session(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Uninitialized => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "send handshake for session",
                );
                self.state = SessionState::Handshaking;

                Ok(String::from("HELLO VERSION").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Create new session with transient destination.
    pub fn create_transient_session(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaked => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "create new transient session",
                );
                self.state = SessionState::SessionCreatePending;

                Ok(format!(
                    "SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT i2cp.leaseSetEncType=4\n",
                    self.options.nickname
                )
                .into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Initialize new session by handshaking with the router using the created session.
    pub fn handshake_stream(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Uninitialized,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "send handshake for stream",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Handshaking,
                };

                Ok(String::from("HELLO VERSION").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    ///
    pub fn create_stream(&mut self, remote_destination: &str) -> Result<Vec<u8>, ProtocolError> {
        match mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    %destination,
                    "open connection to destination",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::ConnectPending,
                };

                Ok(format!(
                    "STREAM CONNECT ID={} DESTINATION={} SILENT=false\n",
                    self.options.nickname, remote_destination
                )
                .into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Handle response from router.
    pub fn handle_response(&mut self, response: &str) -> Result<(), ProtocolError> {
        match mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaking => {
                // TODO: parse handshake
                self.state = SessionState::Handshaked;

                Ok(())
            }
            SessionState::SessionCreatePending => {
                // TODO: parse response
                // TODO: extract destination
                self.state = SessionState::Active {
                    destination: String::from("todo"),
                    stream_state: StreamState::Uninitialized,
                };

                Ok(())
            }
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaking,
            } => {
                // TODO: parse response

                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Handshaked,
                };

                Ok(())
            }
            SessionState::Active {
                destination,
                stream_state: StreamState::ConnectPending,
            } => {
                // TODO: parse response

                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Active,
                };

                Ok(())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot handle response, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_virtual_stream() {
        let mut controller = StreamController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("response").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        assert!(controller.create_transient_session().is_ok());
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller.handle_response("response").is_ok());
        assert!(std::matches!(controller.state, SessionState::Active { .. }));

        // handshake virtual stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("response").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination").is_ok());

        let SessionState::Active {
            stream_state: StreamState::ConnectPending,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("response").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Active,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn handshake() {
        // HELLO REPLY RESULT=OK VERSION=3.3

        // SESSION STATUS RESULT=OK DESTINATION=TIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJmRZ8D0ewvPmy2QKbhZTS3Y9B~nR2m~2vf3yPdVWR7pokR0PeHn-vQ8Av0VNEKUete3L7pEvwrm8CxrIY2aUkV~CpNliKwvhfsJe7tSDSL32Ia42O45KTZbGkI9jvKDdFblwoOYpcd1ToDFZ5qWQ0bxACistfpu609-1Tw1y26neAAAA08XrilOIapGsMhNO1WihrFDLOycxcJlTlqbhV1NKKgekUa-RjUuL1n2hx7VjQK2iSK4FNUprfsr1GEIrOvaNKUD4B0fc7Xshbr43oZZ-LE0FxhNdOhz5KOEzW-eqE7V84PTWIfpY9to6Mm1JObl6ARHhVxPvSVQzkNMuuoFQoB2STMOw2osPXxr7tk~qVYnBrrHpZYrfGIyO1tN1MDCJPqTbFaCNb3Jtnxz3h7B~aJFAHzzEl~sHpMJx7IWAaVr-e2mIRin7fywJq3IhuPy8DdAJiIa-8qrjDDrNNg02a3BgSN4If6sTFooGRX-cXnuCjbbqjzg3dq8parcTekauEFtlTl6d17wFQ3o~JtFQ4ObzpGuW

        // STREAM STATUS RESULT=OK
        // STREAM STATUS RESULT=CANT_REACH_PEER MESSAGE="Connection failed"
    }
}
