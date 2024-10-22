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

use std::fmt;

/// `yosemite` error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("i/o error: `{0}`")]
    IoError(#[from] std::io::Error),

    /// Protocol error.
    #[error("protocol error: `{0}`")]
    Protocol(ProtocolError),

    /// I2P error, received from the router.
    #[error("i2p error: `{0}`")]
    I2p(I2pError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// Invalid state for an operation.
    InvalidState,

    /// Router sent an invalid message.
    InvalidMessage,

    /// Router error.
    Router(I2pError),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState => write!(f, "invalid state"),
            Self::InvalidMessage => write!(f, "invalid message from router"),
            Self::Router(error) => write!(f, "router error: {error:?}"),
        }
    }
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        Error::Protocol(value)
    }
}

/// I2P error.
#[derive(Debug, PartialEq, Eq)]
pub enum I2pError {
    /// The peer exists, but cannot be reached.
    CantReachPeer,

    /// The specified destination is already in use.
    DuplicatedDest,

    /// A generic I2P error (e.g., I2CP disconnection).
    I2pError(Option<String>),

    /// The specified key is not valid (e.g., bad format).
    InvalidKey,

    /// The naming system can't resolve the given name.
    KeyNotFound,

    /// The peer cannot be found on the network.
    PeerNotFound,

    /// Timeout while waiting for an event (e.g. peer answer).
    Timeout,
}

impl fmt::Display for I2pError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CantReachPeer => write!(f, "the peer exists, but cannot be reached"),
            Self::DuplicatedDest => write!(f, "the specified destination is already in use"),
            Self::I2pError(message) => write!(
                f,
                "generic i2p error (e.g., i2cp disconnection): {message:?}"
            ),
            Self::InvalidKey => write!(f, "the specified key is not valid (e.g., bad format)"),
            Self::KeyNotFound => write!(f, "the naming system can't resolve the given name"),
            Self::PeerNotFound => write!(f, "the peer cannot be found on the network"),
            Self::Timeout => write!(f, "timeout while waiting for an event (e.g. peer answer)"),
        }
    }
}

impl TryFrom<(&str, Option<&str>)> for I2pError {
    type Error = ();

    fn try_from(value: (&str, Option<&str>)) -> Result<Self, Self::Error> {
        match value.0 {
            "CANT_REACH_PEER" => Ok(I2pError::CantReachPeer),
            "DUPLICATED_DEST" => Ok(I2pError::DuplicatedDest),
            "I2P_ERROR" => Ok(I2pError::I2pError(
                value.1.map(|message| message.to_string()),
            )),
            "INVALID_KEY" => Ok(I2pError::InvalidKey),
            "KEY_NOT_FOUND" => Ok(I2pError::KeyNotFound),
            "PEER_NOT_FOUND" => Ok(I2pError::PeerNotFound),
            "TIMEOUT" => Ok(I2pError::Timeout),
            _ => Err(()),
        }
    }
}
