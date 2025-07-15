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

use crate::{
    error::ProtocolError,
    options::{SessionOptions, StreamOptions},
    proto::parser::Response,
    style::private::SessionParameters,
    DestinationKind,
};

/// Logging target for the file.
const LOG_TARGET: &str = "yosemite::proto::session";

/// Stream kind
#[derive(Debug, PartialEq, Eq, Clone)]
enum StreamKind {
    /// `STREAM ACCEPT` has been sent.
    Accept,

    /// `STREAM CONNECT` has been sent.
    Connect,

    /// `STREAM FORWARD` has been sent.
    Forward,
}

/// Virtual stream state.
#[derive(Debug, PartialEq, Eq, Clone)]
enum StreamState {
    /// Stream state is uninitialized.
    Uninitialized,

    /// Stream is being handshaked.
    Handshaking,

    /// Stream has been handshaked.
    Handshaked,

    /// `STREAM CONNECT`/`STREAM ACCEPT` is pending.
    Pending(StreamKind),
}

/// Session state.
#[derive(Debug, PartialEq, Eq, Clone)]
enum SessionState {
    /// Session is uninitialized.
    Uninitialized,

    /// Handshake has been sent to router.
    Handshaking,

    /// Session has been handshaked.
    Handshaked,

    /// `SESSION CREATE` message has been sent.
    SessionCreatePending,

    /// `SESSION ADD` message has been sent.
    SubsessionCreatePending {
        /// Created destination.
        destination: String,
    },

    /// Session is active.
    Active {
        /// Created destination.
        destination: String,

        /// Stream state
        stream_state: StreamState,
    },

    /// Session state has been poisoned.
    Poisoned,
}

/// State machine for SAMv3 virtual streams.
#[derive(Clone)]
pub struct SessionController {
    /// Session options.
    options: SessionOptions,

    /// Session state.
    state: SessionState,
}

impl SessionController {
    /// Create new [`SessionController`] from `options`.
    pub fn new(options: SessionOptions) -> Result<Self, ProtocolError> {
        Ok(Self {
            options,
            state: SessionState::Uninitialized,
        })
    }

    /// Create new [`SessionController`] for a subsession from primary session's state.
    pub(crate) fn new_for_subsession(&self, options: SessionOptions) -> Self {
        match &self.state {
            SessionState::Active {
                destination,
                stream_state: StreamState::Uninitialized,
            } => Self {
                options,
                state: SessionState::Active {
                    destination: destination.clone(),
                    stream_state: StreamState::Uninitialized,
                },
            },
            _ => unreachable!(),
        }
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

    /// Create new session with either transient or persistent destination.
    pub fn create_session(
        &mut self,
        parameters: SessionParameters,
    ) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaked => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    destination = ?self.options.destination,
                    "create new session",
                );
                self.state = SessionState::SessionCreatePending;

                let mut command = format!(
                    "SESSION CREATE STYLE={} ID={} ",
                    parameters.style, self.options.nickname
                );

                match parameters.style.as_str() {
                    "STREAM" => {
                        for (key, value) in parameters.options {
                            command += format!("{key}={value} ").as_str();
                        }

                        match &self.options.destination {
                            DestinationKind::Transient => {
                                command += "DESTINATION=TRANSIENT ";
                            }
                            DestinationKind::Persistent { private_key } => {
                                command += format!("DESTINATION={private_key} ").as_str();
                            }
                        }

                        if !self.options.publish_lease_set {
                            command += "i2cp.dontPublishLeaseSet=true ";
                        }

                        command += format!(
                            "inbound.length={} inbound.quantity={} ",
                            self.options.inbound_len,
                            self.options.inbound_qty
                        )
                        .as_str();

                        command += format!(
                            "outbound.length={} outbound.quantity={} ",
                            self.options.outbound_len,
                            self.options.outbound_qty
                        )
                        .as_str();

                        command += "SIGNATURE_TYPE=7 i2cp.leaseSetEncType=4\n";

                        Ok(command.into_bytes())
                    }

                    "RAW" => {
                        for (key, value) in parameters.options {
                            command += format!("{key}={value} ").as_str();
                        }

                        command += format!(
                            "PORT={} HOST={} FROM_PORT={} TO_PORT={} PROTOCOL={} HEADER={}",
                            self.options.datagram_port,
                            self.options.datagram_host,
                            self.options.from_port,
                            self.options.to_port,
                            self.options.protocol,
                            self.options.header,
                        )
                        .as_str();

                        if !self.options.publish_lease_set {
                            command += "i2cp.dontPublishLeaseSet=true ";
                        }

                        command += format!(
                            "inbound.length={} inbound.quantity={} ",
                            self.options.inbound_len,
                            self.options.inbound_qty
                        )
                        .as_str();

                        command += format!(
                            "outbound.length={} outbound.quantity={} ",
                            self.options.outbound_len,
                            self.options.outbound_qty
                        )
                        .as_str();

                        command += "SIGNATURE_TYPE=7 i2cp.leaseSetEncType=4\n";

                        Ok(command.into_bytes())
                    }

                    "DATAGRAM" => {
                        for (key, value) in parameters.options {
                            command += format!("{key}={value} ").as_str();
                        }

                        match &self.options.destination {
                            DestinationKind::Transient => {
                                command += "DESTINATION=TRANSIENT ";
                            }
                            DestinationKind::Persistent { private_key } => {
                                command += format!("DESTINATION={private_key} ").as_str();
                            }
                        }

                        command += format!(
                            "PORT={} HOST={} FROM_PORT={} TO_PORT={}",
                            self.options.datagram_port,
                            self.options.datagram_host,
                            self.options.from_port,
                            self.options.to_port,
                        )
                        .as_str();

                        if !self.options.publish_lease_set {
                            command += "i2cp.dontPublishLeaseSet=true ";
                        }

                        command += format!(
                            "inbound.length={} inbound.quantity={} ",
                            self.options.inbound_len,
                            self.options.inbound_qty
                        )
                        .as_str();

                        command += format!(
                            "outbound.length={} outbound.quantity={} ",
                            self.options.outbound_len,
                            self.options.outbound_qty
                        )
                        .as_str();

                        command += "SIGNATURE_TYPE=7 i2cp.leaseSetEncType=4\n";

                        Ok(command.into_bytes())
                    }

                    _ => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            style = %parameters.style,
                            "cannot create session, non-supported session style",
                        );
                        Err(ProtocolError::InvalidMessage)
                    }
                }
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

    /// Create new subsession.
    pub fn create_subsession(
        &mut self,
        nickname: &str,
        parameters: SessionParameters,
    ) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active { destination, .. } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    %nickname,
                    style = %parameters.style,
                    "create new subsession",
                );
                self.state = SessionState::SubsessionCreatePending { destination };

                let mut command = format!("SESSION ADD STYLE={} ID={nickname} ", parameters.style);

                for (key, value) in parameters.options {
                    command += format!("{key}={value} ").as_str();
                }
                command += "\n";

                Ok(command.into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create subsession, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Handshake stream, either inbound or outbound.
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

    /// Open virtual stream to `destination`.
    pub fn create_stream(
        &mut self,
        remote_destination: &str,
        options: StreamOptions,
    ) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::info!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    remote_destination = %format!("{}...", &destination[..10]),
                    "open stream to remote destination",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Connect),
                };

                Ok(format!(
                    "STREAM CONNECT ID={} DESTINATION={} FROM_PORT={} TO_PORT={} SILENT=false\n",
                    self.options.nickname, remote_destination, options.src_port, options.dst_port,
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

    /// Start accepting a new virtual stream.
    pub fn accept_stream(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "start listening for virtual stream",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Accept),
                };

                Ok(
                    format!("STREAM ACCEPT ID={} SILENT=false\n", self.options.nickname)
                        .into_bytes(),
                )
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

    /// Forward inbound virtual streams to a TCP listener listening to `port`.
    pub fn forward_stream(&mut self, port: u16) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    ?port,
                    "forward incoming connections",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Forward),
                };

                Ok(format!(
                    "STREAM FORWARD ID={} PORT={port} SILENT={}\n",
                    self.options.nickname,
                    self.options.silent_forward.to_string(),
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
            SessionState::Handshaking => match Response::parse(response) {
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

                    Ok(())
                }
                Some(Response::Hello {
                    version: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router session `HELLO`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        "unexpected response from router session `HELLO`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            SessionState::SessionCreatePending => match Response::parse(response) {
                Some(Response::Session {
                    destination: Ok(destination),
                }) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        "session created",
                    );

                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    Ok(())
                }
                Some(Response::Session {
                    destination: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router for `SESSION CREATE`",
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
            },
            SessionState::SubsessionCreatePending { destination } =>
                match Response::parse(response) {
                    Some(Response::Subsession {
                        session_id: Ok(session_id),
                    }) => {
                        tracing::info!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            %session_id,
                            "subsession created",
                        );

                        self.state = SessionState::Active {
                            destination,
                            stream_state: StreamState::Uninitialized,
                        };

                        Ok(())
                    }
                    Some(Response::Subsession {
                        session_id: Err(error),
                    }) => return Err(ProtocolError::Router(error)),
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router for `SESSION ADD`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router to `SESSION ADD`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                },
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaking,
            } => match Response::parse(response) {
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

                    Ok(())
                }
                Some(Response::Hello {
                    version: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router stream `HELLO`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        "unexpected response from router stream `HELLO`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            SessionState::Active {
                destination,
                stream_state: StreamState::Pending(direction),
            } => match Response::parse(response) {
                Some(Response::Stream { result: Ok(()) }) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?direction,
                        "stream status ok",
                    );

                    // after the stream is opened/accepted, the stream is handed off
                    // to user and the stream state can be reset
                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    Ok(())
                }
                Some(Response::Stream { result: Err(error) }) => {
                    // stream failed to open, reset state back to uninitialized
                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    return Err(ProtocolError::Router(error));
                }
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        ?direction,
                        "invalid response from router to `STREAM CREATE`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        ?direction,
                        "unexpected response from router to `STREAM CREATE`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
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

    /// Get reference to [`SessionController`]'s destination.
    ///
    /// Panics if called before the session is active.
    pub fn destination(&self) -> &str {
        let SessionState::Active { destination, .. } = &self.state else {
            panic!("invalid state");
        };

        &destination
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_virtual_stream() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
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
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn accept_virtual_stream() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
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
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.accept_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Accept),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn dont_publish_lease_set() {
        let mut controller = SessionController::new(SessionOptions {
            publish_lease_set: false,
            ..Default::default()
        })
        .unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(command.contains("i2cp.dontPublishLeaseSet=true"));
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
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn stream_fails_to_open() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
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
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect failure
        assert!(controller.handle_response("STREAM STATUS RESULT=CANT_REACH_PEER\n").is_err());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // try to open another stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn create_primary_and_subsession() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "PRIMARY".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
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

        assert!(controller
            .create_subsession(
                "test",
                SessionParameters {
                    style: "STREAM".to_string(),
                    options: Vec::new()
                }
            )
            .is_ok());

        let SessionState::SubsessionCreatePending { .. } = controller.state else {
            panic!("invalid state");
        };

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK ID=\"lS24mtNyeNVMf2bZ\" MESSAGE=\"ADD lS24mtNyeNVMf2bZ\"\n\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }
    }
}
