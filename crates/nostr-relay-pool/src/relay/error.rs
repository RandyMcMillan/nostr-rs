// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;

use async_utility::thread;
use nostr::message::relay::NegentropyErrorCode;
use nostr::message::MessageHandleError;
use nostr::{event, negentropy, EventId, Kind, PublicKey};
use nostr_database::DatabaseError;
use thiserror::Error;

/// [`Relay`](super::Relay) error
#[derive(Debug, Error)]
pub enum Error {
    /// MessageHandle error
    #[error(transparent)]
    MessageHandle(#[from] MessageHandleError),
    /// Event error
    #[error(transparent)]
    Event(#[from] event::Error),
    /// Event ID error
    #[error(transparent)]
    EventId(#[from] event::id::Error),
    /// Partial Event error
    #[error(transparent)]
    PartialEvent(#[from] event::partial::Error),
    /// Negentropy error
    #[error(transparent)]
    Negentropy(#[from] negentropy::Error),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Thread error
    #[error(transparent)]
    Thread(#[from] thread::Error),
    /// Message response timeout
    #[error("recv message response timeout")]
    RecvTimeout,
    /// WebSocket timeout
    #[error("WebSocket timeout")]
    WebSocketTimeout,
    /// Generic timeout
    #[error("timeout")]
    Timeout,
    /// Message response timeout
    #[error("Can't send message to the '{channel}' channel")]
    CantSendChannelMessage {
        /// Name of channel
        channel: String,
    },
    /// Message not sent
    #[error("message not sent")]
    MessageNotSent,
    /// Relay not connected
    #[error("relay not connected")]
    NotConnected,
    /// Relay not connected
    #[error("relay not connected (status changed)")]
    NotConnectedStatusChanged,
    /// Event not published
    #[error("event not published: {0}")]
    EventNotPublished(String),
    /// No event is published
    #[error("events not published: {0:?}")]
    EventsNotPublished(HashMap<EventId, String>),
    /// Only some events
    #[error("partial publish: published={}, missing={}", published.len(), not_published.len())]
    PartialPublish {
        /// Published events
        published: Vec<EventId>,
        /// Not published events
        not_published: HashMap<EventId, String>,
    },
    /// Batch event empty
    #[error("batch event cannot be empty")]
    BatchEventEmpty,
    /// Impossible to receive oneshot message
    #[error("impossible to recv msg")]
    OneShotRecvError,
    /// Read actions disabled
    #[error("read actions are disabled for this relay")]
    ReadDisabled,
    /// Write actions disabled
    #[error("write actions are disabled for this relay")]
    WriteDisabled,
    /// Filters empty
    #[error("filters empty")]
    FiltersEmpty,
    /// Reconciliation error
    #[error("negentropy reconciliation error: {0}")]
    NegentropyReconciliation(NegentropyErrorCode),
    /// Negentropy not supported
    #[error("negentropy not supported")]
    NegentropyNotSupported,
    /// Unknown negentropy error
    #[error("unknown negentropy error")]
    UnknownNegentropyError,
    /// Relay message too large
    #[error("Received message too large: size={size}, max_size={max_size}")]
    RelayMessageTooLarge {
        /// Message size
        size: usize,
        /// Max message size
        max_size: usize,
    },
    /// Event too large
    #[error("Received event too large: size={size}, max_size={max_size}")]
    EventTooLarge {
        /// Event size
        size: usize,
        /// Max event size
        max_size: usize,
    },
    /// Too many tags
    #[error("Received event with too many tags: tags={size}, max_tags={max_size}")]
    TooManyTags {
        /// Tags num
        size: usize,
        /// Max tags num
        max_size: usize,
    },
    /// Event expired
    #[error("event expired")]
    EventExpired,
    /// POW difficulty too low
    #[error("POW difficulty too low (min. {min})")]
    PowDifficultyTooLow {
        /// Min. difficulty
        min: u8,
    },
    /// Event ID blacklisted
    #[error("Received event with blacklisted ID: {0}")]
    EventIdBlacklisted(EventId),
    /// Public key blacklisted
    #[error("Received event authored by blacklisted public key: {0}")]
    PublicKeyBlacklisted(PublicKey),
    /// Unexpected kind
    #[error("Unexpected kind: expected={expected}, found={found}")]
    UnexpectedKind {
        /// Expected kind
        expected: Kind,
        /// Found kind
        found: Kind,
    },
    /// Notification Handler error
    #[error("notification handler error: {0}")]
    Handler(String),
    /// WebSocket error
    #[error("{0}")]
    WebSocket(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    #[inline]
    pub(super) fn websocket<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WebSocket(Box::new(error))
    }
}
