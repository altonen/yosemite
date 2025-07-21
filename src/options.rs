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

use std::{fmt, time::Duration};

/// Default port for UDP.
pub(crate) const SAMV3_UDP_PORT: u16 = 7655;

/// Default port for TCP.
pub(crate) const SAMV3_TCP_PORT: u16 = 7656;

/// Destination kind.
#[derive(Clone, PartialEq, Eq)]
pub enum DestinationKind {
    /// Transient session.
    Transient,

    /// Session from pre-generated destination data.
    Persistent {
        /// Base64 of the concatenation of the destination followed by the private key followed by
        /// the signing private key.
        private_key: String,
    },
}

impl fmt::Debug for DestinationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transient => f.debug_struct("DestinationKind::Transient").finish(),
            Self::Persistent { .. } =>
                f.debug_struct("DestinationKind::Persistent").finish_non_exhaustive(),
        }
    }
}

/// Session options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionOptions {
    /// Nickname.
    ///
    /// Name that uniquely identifies the session.
    ///
    /// Defaults to a random alphanumeric string.
    pub nickname: String,

    /// Destination kind.
    ///
    /// Defaults to [`DestinationKind::Transient`].
    pub destination: DestinationKind,

    /// Signature type.
    ///
    /// Default to '7' i.e. EdDSA_SHA512_Ed25519
    pub signature_type: u16,

    /// Port where the datagram socket should be bound to.
    ///
    /// Defaults to `0`.
    pub datagram_port: u16,

    /// Defaults to `0`.
    pub from_port: u16,

    /// Defaults to `0`.
    pub to_port: u16,

    /// Defaults to `18`.
    pub protocol: u8,

    /// Defaults to `false`.
    pub header: bool,

    /// Should the session's lease set be published to NetDb.
    ///
    /// Outbound-only sessions (clients) shouldn't be published whereas servers (accepting inbound
    /// connections) need to be published.
    ///
    /// Corresponds to `i2cp.dontPublishLeaseSet`.
    ///
    /// Defaults to `true`.
    pub publish_lease_set: bool,

    /// Minimum number of ElGamal/AES Session Tags before we send more. Recommended: approximately
    /// tagsToSend * 2/3
    ///
    /// /// Defaults to `30`.
    pub crypto_low_tag_threshold: usize,

    /// Inbound tag window for ECIES-X25519-AEAD-Ratchet. Local inbound tagset size.
    ///
    /// /// Defaults to `160`.
    pub crypto_ratchet_inbound_tags: usize,

    /// Outbound tag window for ECIES-X25519-AEAD-Ratchet. Advisory to send to the far-end in the
    /// options block.
    ///
    /// /// Defaults to `160`.
    pub crypto_ratchet_outbound_tags: usize,

    /// Number of ElGamal/AES Session Tags to send at a time.
    ///
    /// /// Defaults to `40`.
    pub crypto_tags_to_send: usize,

    /// For authorization, if required by the router.
    pub username: Option<String>,

    /// For authorization, if required by the router.
    pub password: Option<String>,

    /// If incoming zero hop tunnel is allowed
    ///
    /// Defaults to 'false'
    pub inbound_allow_zero_hop: bool,

    /// How many hops do the inbound tunnels of the session have.
    ///
    /// Defaults to `3`.
    pub inbound_len: usize,

    /// Random amount to add or subtract to the length of tunnels in.
    ///
    /// Defaults to `0`.
    pub inbound_len_variance: isize,

    /// How many inbound tunnels does the tunnel pool of the session have.
    ///
    /// Defaults to `2`.
    pub inbound_quantity: usize,

    /// Number of redundant fail-over for tunnels in
    ///
    /// Defaults to `0`.
    pub inbound_backup_qty: usize,

    /// Number of IP bytes to match to determine if two routers should not be in the same tunnel. 0
    /// to disable.
    ///
    /// Defaults to `0`.
    pub inbound_ip_restriction: usize,

    /// Used for consistent peer ordering across restarts.
    pub inbound_random_key: Option<String>,

    /// Name of inbound tunnels - generally used in routerconsole, which will use
    /// the first few characters of the Base64 hash of the destination by default.
    ///
    /// Defauts to 'None'
    pub inbound_nickname: Option<String>,

    /// If outgoing zero hop tunnel is allowed
    ///
    ///  Defaults to 'false'
    pub outbound_allow_zero_hop: bool,

    /// How many hops do the outbound tunnels of the session have.
    ///
    /// Defaults to `3`.
    pub outbound_len: usize,

    /// Random amount to add or subtract to the length of tunnels in.
    ///
    /// Defaults to `0`.
    pub outbound_len_variance: isize,

    /// How many outbound tunnels does the tunnel pool of the session have.
    ///
    /// Defaults to `2`.
    pub outbound_quantity: usize,

    /// Number of redundant fail-over for tunnels out
    ///
    /// Defaults to `0`.
    pub outbound_backup_qty: usize,

    /// Number of IP bytes to match to determine if two routers should not be in the same tunnel. 0
    /// to disable.
    ///
    /// Defaults to `0`.
    pub outbound_ip_restriction: usize,

    /// Priority adjustment for outbound messages. Higher is higher priority.
    ///
    /// Defaults to `0`.
    pub outbound_priority: isize,

    /// Used for consistent peer ordering across restarts.
    pub outbound_random_key: Option<String>,

    /// Name of outbound tunnels - generally ignored unless inbound.nickname is unset.
    ///
    /// Defauts to 'None'
    pub outbound_nickname: Option<String>,

    /// Set to false to disable ever bundling a reply LeaseSet.
    ///
    /// Defaults to `true`.
    pub should_bundle_reply_info: bool,

    /// Close I2P session when idle
    ///
    /// Defaults to 'false'
    pub close_on_idle: bool,

    /// (ms) Idle time required before closing session
    ///
    /// Defaults to '1800000' ms (i.e. 30 minutes)
    pub close_idle_time: Duration,

    /// Encrypt the lease
    ///
    /// Defaults to `false`.
    pub encrypt_lease_set: bool,

    /// Gzip outbound data
    ///
    /// Defaults to `true`.
    pub gzip: bool,

    /// The type of authentication for encrypted LS2. 0 for no per-client authentication ;
    /// 1 for DH per-client authentication; 2 for PSK per-client authentication.
    ///
    /// Defaults to `0`.
    pub lease_set_auth_type: usize,

    /// The sig type of the blinded key for encrypted LS2. Default depends on the destination sig
    /// type.
    ///
    /// Defaults to `0`.
    pub lease_set_blinded_type: usize,

    /// The encryption type to be used.
    ///
    /// Defaults to '4' i.e. ECIES-X25519
    pub lease_set_enc_type: usize,

    /// For encrypted leasesets. Base 64 SessionKey (44 characters)
    pub lease_set_key: Option<String>,

    /// Base 64 private keys for encryption.
    pub lease_set_private_key: Option<String>,

    /// Base 64 encoded UTF-8 secret used to blind the leaseset address.
    pub lease_set_secret: Option<String>,

    /// The type of leaseset to be sent in the CreateLeaseSet2 Message.
    pub lease_set_signing_private_key: Option<String>,

    /// Reduce tunnel quantity when idle
    ///
    /// Defaults to 'false'
    pub reduce_on_idle: bool,

    /// (ms) Idle time required before reducing tunnel quantity
    ///
    /// Defaults to '1200000' ms (i.e. 20 minutes)
    pub reduce_idle_time: Duration,

    /// Tunnel quantity when reduced (applies to both inbound and outbound)
    pub reduce_quantity: usize,

    /// Connect to the router using SSL.
    pub ssl: bool,

    /// TCP port of the listening SAMv3 server.
    ///
    /// Defaults to `7656`.
    pub samv3_tcp_port: u16,

    /// UDP port of the listening SAMv3 server.
    ///
    /// Defaults to `7655`
    pub samv3_udp_port: u16,

    /// Should `STREAM FORWARD` be silent.
    ///
    /// If set to false, the first message read from the socket is the destination of the remote
    /// peer.
    ///
    /// Defaults to `false`.
    pub silent_forward: bool,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            nickname: Alphanumeric.sample_string(&mut thread_rng(), 16),
            destination: DestinationKind::Transient,
            signature_type: 7u16,
            datagram_port: 0u16,
            from_port: 0u16,
            to_port: 0u16,
            protocol: 18u8,
            header: false,
            publish_lease_set: true,
            crypto_low_tag_threshold: 30usize,
            crypto_ratchet_inbound_tags: 160usize,
            crypto_ratchet_outbound_tags: 160usize,
            crypto_tags_to_send: 40usize,
            username: None,
            password: None,
            inbound_allow_zero_hop: false,
            inbound_len: 3usize,
            inbound_len_variance: 0isize,
            inbound_quantity: 2usize,
            inbound_backup_qty: 0usize,
            inbound_ip_restriction: 0usize,
            inbound_random_key: None,
            inbound_nickname: None,
            outbound_allow_zero_hop: false,
            outbound_len: 3usize,
            outbound_len_variance: 0isize,
            outbound_quantity: 2usize,
            outbound_backup_qty: 0usize,
            outbound_ip_restriction: 0usize,
            outbound_priority: 0isize,
            outbound_random_key: None,
            outbound_nickname: None,
            should_bundle_reply_info: true,
            close_on_idle: false,
            close_idle_time: Duration::from_millis(1800000),
            encrypt_lease_set: false,
            gzip: true,
            lease_set_auth_type: 0usize,
            lease_set_blinded_type: 0usize,
            lease_set_enc_type: 4usize,
            lease_set_key: None,
            lease_set_private_key: None,
            lease_set_secret: None,
            lease_set_signing_private_key: None,
            reduce_on_idle: false,
            reduce_idle_time: Duration::from_millis(1200000),
            reduce_quantity: 1usize,
            ssl: false,
            samv3_tcp_port: SAMV3_TCP_PORT,
            samv3_udp_port: SAMV3_UDP_PORT,
            silent_forward: false,
        }
    }
}

/// Stream options.
#[derive(Debug, Default, Clone, Copy)]
pub struct StreamOptions {
    /// Destination port.
    ///
    /// Defaults to `0`.
    pub dst_port: u16,

    /// Source port.
    ///
    /// Defaults to `0`.
    pub src_port: u16,
}
