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

use crate::{error::ProtocolError, proto::parser::Response};

/// Logging target for the file.
const LOG_TARGET: &str = "yosemite::proto::router-api";

/// Router API controller state.
#[derive(Debug)]
enum RouterApiControllerState {
    /// State is uninitialized.
    Uninitialized,

    /// Handshaking with router.
    Handshaking,

    /// Router API has been handshaked.
    Handshaked,

    /// Awaiting response to `NAMING LOOKUP`.
    AwaitingLookupResponse,

    /// Awaiting response to `DEST GENERATE`.
    AwaitingDestinationResponse,

    /// Naming lookup succeeded.
    //
    // TODO this is kind of hackish.
    LookupSucceeded {
        /// Base64-encoded destination.
        destination: String,
    },

    /// Destination generation.
    //
    // TODO this is kind of hackish.
    DestinationGenerated {
        /// Base64-encoded destination.
        destination: String,

        /// Base64 of the concatenation of the destination followed by the private key followed by
        /// the signing private key.
        private_key: String,
    },

    /// State has been poisoned.
    Poisoned,
}

/// Router API controller.
pub struct RouterApiController {
    /// State of the router API controller.
    state: RouterApiControllerState,
}

impl RouterApiController {
    /// Create new [`RouterApiController`].
    pub fn new() -> Self {
        Self {
            state: RouterApiControllerState::Uninitialized,
        }
    }

    /// Initialize router API by handshaking with the router.
    pub fn handshake_router_api(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Poisoned) {
            RouterApiControllerState::Uninitialized => {
                tracing::trace!(
                    target: LOG_TARGET,
                    "send handshake for router api",
                );
                self.state = RouterApiControllerState::Handshaking;

                Ok(String::from("HELLO VERSION\n").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot handshake router api, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Lookup destination associated with `name`.
    pub fn lookup_name(&mut self, name: &str) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Poisoned) {
            RouterApiControllerState::Handshaked => {
                tracing::info!(
                    target: LOG_TARGET,
                    %name,
                    "lookup destination",
                );
                self.state = RouterApiControllerState::AwaitingLookupResponse;

                Ok(format!("NAMING LOOKUP NAME={name}\n").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot lookup hostname, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Lookup destination associated with `name`.
    pub fn generate_destination(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Poisoned) {
            RouterApiControllerState::Handshaked => {
                tracing::info!(
                    target: LOG_TARGET,
                    "generate destination",
                );
                self.state = RouterApiControllerState::AwaitingDestinationResponse;

                Ok(format!("DEST GENERATE SIGNATURE_TYPE=7\n").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot generate destination, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Handle response from router.
    pub fn handle_response(&mut self, response: &str) -> Result<(), ProtocolError> {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Poisoned) {
            RouterApiControllerState::Handshaking => match Response::parse(response) {
                Some(Response::Hello {
                    version: Ok(version),
                }) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        %version,
                        "router api handshake done",
                    );
                    self.state = RouterApiControllerState::Handshaked;

                    Ok(())
                }
                Some(Response::Hello {
                    version: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?response,
                        "invalid response from router for `HELLO`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?response,
                        "unexpected response from router for `HELLO`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            RouterApiControllerState::AwaitingLookupResponse => match Response::parse(response) {
                Some(Response::NamingLookup {
                    result: Ok(destination),
                }) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        "destination found",
                    );

                    self.state = RouterApiControllerState::LookupSucceeded { destination };
                    Ok(())
                }
                Some(Response::NamingLookup { result: Err(error) }) =>
                    return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?response,
                        "invalid response from router for `NAMING LOOKUP`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?response,
                        "unexpected response from router for `NAMING LOOKUP`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            RouterApiControllerState::AwaitingDestinationResponse =>
                match Response::parse(response) {
                    Some(Response::DestinationGeneration {
                        destination,
                        private_key,
                    }) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            "destination generated",
                        );

                        self.state = RouterApiControllerState::DestinationGenerated {
                            destination,
                            private_key,
                        };
                        Ok(())
                    }
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            ?response,
                            "invalid response from router for `DEST GENERATE`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            ?response,
                            "unexpected response from router for `DEST GENERATE`",
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

    /// Get destination of the hostname.
    pub fn destination(&mut self) -> String {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Uninitialized) {
            RouterApiControllerState::LookupSucceeded { destination } => destination,
            _ => panic!("invalid state"),
        }
    }

    /// Get generated destination and private key.
    pub fn generated_destination(&mut self) -> (String, String) {
        match std::mem::replace(&mut self.state, RouterApiControllerState::Uninitialized) {
            RouterApiControllerState::DestinationGenerated {
                destination,
                private_key,
            } => (destination, private_key),
            _ => panic!("invalid state"),
        }
    }
}
