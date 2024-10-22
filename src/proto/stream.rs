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

use crate::{error::ProtocolError, proto::parser::Response, StreamOptions};

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
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Uninitialized => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "send handshake for session",
                );
                self.state = SessionState::Handshaking;

                Ok(String::from("HELLO VERSION\n").into_bytes())
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
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
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
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
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

                Ok(String::from("HELLO VERSION\n").into_bytes())
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

    /// Create new virtual stream by connecting to `remote_destination`.
    pub fn create_stream(&mut self, remote_destination: &str) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    %remote_destination,
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
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaking => {
                match Response::parse(response) {
                    None => return Err(ProtocolError::InvalidMessage),
                    Some(Response::Hello {
                        version: Ok(version),
                    }) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            %version,
                            "session handshake done",
                        );
                        self.state = SessionState::Handshaked;
                    }
                    Some(Response::Hello {
                        version: Err(error),
                    }) => return Err(ProtocolError::Router(error)),
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router `HELLO`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router `HELLO`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                }

                Ok(())
            }
            SessionState::SessionCreatePending => {
                match Response::parse(response) {
                    None => return Err(ProtocolError::InvalidMessage),
                    Some(Response::Session {
                        destination: Ok(destination),
                    }) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            "session created",
                        );

                        self.state = SessionState::Active {
                            destination,
                            stream_state: StreamState::Uninitialized,
                        };
                    }
                    Some(Response::Session {
                        destination: Err(error),
                    }) => return Err(ProtocolError::Router(error)),
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router `SESSION CREATE`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router to `SESSION CREATE`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                }

                Ok(())
            }
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaking,
            } => {
                match Response::parse(response) {
                    None => return Err(ProtocolError::InvalidMessage),
                    Some(Response::Hello {
                        version: Ok(version),
                    }) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            %version,
                            "stream handshake done",
                        );

                        self.state = SessionState::Active {
                            destination,
                            stream_state: StreamState::Handshaked,
                        };
                    }
                    Some(Response::Hello {
                        version: Err(error),
                    }) => return Err(ProtocolError::Router(error)),
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router to `HELLO`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router to `HELLO`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                }

                Ok(())
            }
            SessionState::Active {
                destination,
                stream_state: StreamState::ConnectPending,
            } => {
                match Response::parse(response) {
                    None => return Err(ProtocolError::InvalidMessage),
                    Some(Response::Stream { result: Ok(()) }) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            "stream created",
                        );

                        self.state = SessionState::Active {
                            destination,
                            stream_state: StreamState::Active,
                        };
                    }
                    Some(Response::Stream { result: Err(error) }) => {
                        return Err(ProtocolError::Router(error))
                    }
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router to `STREAM CREATE`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router to `STREAM CREATE`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                }

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
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller
            .handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n")
            .is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        assert!(controller.create_transient_session().is_ok());
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

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
        assert!(controller
            .handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n")
            .is_ok());

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
        assert!(controller
            .handle_response("STREAM STATUS RESULT=OK\n")
            .is_ok());

        let SessionState::Active {
            stream_state: StreamState::Active,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }
}
