// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect client

use std::collections::HashMap;
use std::time::Duration;

use async_utility::time;
use nostr::nips::nip46::{Message, NostrConnectURI, Request, ResponseResult};
use nostr::prelude::*;
use nostr_relay_pool::{
    RelayOptions, RelayPool, RelayPoolNotification, RelaySendOptions, SubscribeOptions,
};
use tokio::sync::broadcast::Receiver;

use super::Error;

/// Nostr Connect Client
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone)]
pub struct Nip46Signer {
    app_keys: Keys,
    signer_public_key: PublicKey,
    pool: RelayPool,
    timeout: Duration,
    secret: Option<String>,
}

impl Nip46Signer {
    /// Construct Nostr Connect client
    pub async fn new(
        uri: NostrConnectURI,
        app_keys: Keys,
        timeout: Duration,
        opts: Option<RelayOptions>,
    ) -> Result<Self, Error> {
        // Check app keys
        if let NostrConnectURI::Client { public_key, .. } = &uri {
            if *public_key != app_keys.public_key() {
                return Err(Error::PublicKeyNotMatchAppKeys);
            }
        }

        // Compose pool
        let pool: RelayPool = RelayPool::default();

        // Add relays
        let opts: RelayOptions = opts.unwrap_or_default();
        for url in uri.relays().into_iter() {
            pool.add_relay(url, opts.clone()).await?;
        }

        // Connect to relays
        pool.connect(Some(Duration::from_secs(10))).await;

        // Subscribe
        let notifications = subscribe(&app_keys, &pool).await?;

        // Get signer public key
        let signer_public_key: PublicKey = match uri.signer_public_key() {
            Some(public_key) => public_key.clone(),
            None => get_signer_public_key(&app_keys, notifications, timeout).await?,
        };

        // Compose
        let this = Self {
            app_keys,
            signer_public_key,
            pool,
            timeout,
            secret: uri.secret(),
        };

        // Send `connect` command if bunker URI
        if uri.is_bunker() {
            this.connect().await?;
        }

        Ok(this)
    }

    /// Get local app keys
    #[inline]
    pub fn local_keys(&self) -> &Keys {
        &self.app_keys
    }

    /// Get signer relays
    pub async fn relays(&self) -> Vec<Url> {
        self.pool.relays().await.into_keys().collect()
    }

    /// Get signer [PublicKey]
    #[inline]
    pub fn signer_public_key(&self) -> PublicKey {
        self.signer_public_key.clone()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> NostrConnectURI {
        NostrConnectURI::Bunker {
            signer_public_key: self.signer_public_key(),
            relays: self.relays().await,
            secret: self.secret.clone(),
        }
    }

    async fn send_request(&self, req: Request) -> Result<ResponseResult, Error> {
        let secret_key: &SecretKey = self.app_keys.secret_key();
        let signer_public_key: PublicKey = self.signer_public_key();

        // Convert request to event
        let msg = Message::request(req);
        tracing::debug!("Sending '{msg}' NIP46 message");

        let req_id = msg.id().to_string();
        let event: Event = EventBuilder::nostr_connect(&self.app_keys, signer_public_key, msg)?
            .to_event(&self.app_keys)?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool.send_event(event, RelaySendOptions::new()).await?;

        time::timeout(Some(self.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::NostrConnect {
                        let msg = nip04::decrypt(secret_key, &event.pubkey, &event.content)?;
                        let msg = Message::from_json(msg)?;

                        tracing::debug!("Received NIP46 message: '{msg}'");

                        if let Message::Response { id, result, error } = &msg {
                            if &req_id == id {
                                if msg.is_auth_url() {
                                    tracing::warn!("Received 'auth_url': {error:?}");
                                } else {
                                    if let Some(result) = result {
                                        return Ok(result.clone());
                                    }

                                    if let Some(error) = error {
                                        return Err(Error::Response(error.to_owned()));
                                    }

                                    break;
                                }
                            }
                        }
                    }
                }
            }

            Err(Error::Timeout)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Connect msg
    async fn connect(&self) -> Result<(), Error> {
        let req = Request::Connect {
            public_key: self.signer_public_key(),
            secret: self.secret.clone(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_connect()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn get_relays(&self) -> Result<HashMap<Url, RelayPermissions>, Error> {
        let req = Request::GetRelays;
        let res = self.send_request(req).await?;
        Ok(res.to_get_relays()?)
    }

    /// Sign an [UnsignedEvent]
    pub async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let req = Request::SignEvent(unsigned);
        let res = self.send_request(req).await?;
        Ok(res.to_sign_event()?)
    }

    /// NIP04 encrypt
    pub async fn nip04_encrypt<T>(&self, public_key: PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let content: &[u8] = content.as_ref();
        let req = Request::Nip04Encrypt {
            public_key,
            text: String::from_utf8_lossy(content).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP04 decrypt
    pub async fn nip04_decrypt<S>(
        &self,
        public_key: PublicKey,
        ciphertext: S,
    ) -> Result<String, Error>
    where
        S: Into<String>,
    {
        let req = Request::Nip04Decrypt {
            public_key,
            ciphertext: ciphertext.into(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP44 encrypt
    pub async fn nip44_encrypt<T>(&self, public_key: PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let content: &[u8] = content.as_ref();
        let req = Request::Nip44Encrypt {
            public_key,
            text: String::from_utf8_lossy(content).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// NIP44 decrypt
    pub async fn nip44_decrypt<T>(&self, public_key: PublicKey, payload: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let payload: &[u8] = payload.as_ref();
        let req = Request::Nip44Decrypt {
            public_key,
            ciphertext: String::from_utf8_lossy(payload).to_string(),
        };
        let res = self.send_request(req).await?;
        Ok(res.to_encrypt_decrypt()?)
    }

    /// Completely shutdown
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
    }
}

async fn subscribe(
    app_keys: &Keys,
    pool: &RelayPool,
) -> Result<Receiver<RelayPoolNotification>, Error> {
    let public_key: PublicKey = app_keys.public_key();

    let filter = Filter::new()
        .pubkey(public_key)
        .kind(Kind::NostrConnect)
        .limit(0);

    let notifications = pool.notifications();

    // Subscribe
    pool.subscribe(vec![filter], SubscribeOptions::default())
        .await?;

    Ok(notifications)
}

async fn get_signer_public_key(
    app_keys: &Keys,
    mut notifications: Receiver<RelayPoolNotification>,
    timeout: Duration,
) -> Result<PublicKey, Error> {
    time::timeout(Some(timeout), async {
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::NostrConnect {
                    // Decrypt content
                    let msg: String =
                        nip04::decrypt(app_keys.secret_key(), &event.pubkey, event.content)?;

                    tracing::debug!("Received Nostr Connect message: '{msg}'");

                    // Parse message
                    let msg: Message = Message::from_json(msg)?;

                    // Match message
                    match msg {
                        Message::Request {
                            req: Request::Connect { public_key, .. },
                            ..
                        } => {
                            return Ok(public_key);
                        }
                        Message::Response {
                            result: Some(ResponseResult::Connect),
                            error: None,
                            ..
                        } => return Ok(event.pubkey),
                        _ => {}
                    }
                }
            }
        }

        Err(Error::SignerPublicKeyNotFound)
    })
    .await
    .ok_or(Error::Timeout)?
}
