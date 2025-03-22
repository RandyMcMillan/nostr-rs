// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeMap, HashMap};
use std::collections::BTreeSet;
use anyhow::{anyhow, Result};
use git2::{Commit, ObjectType, Oid, Repository};
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Debug)]
struct SerializableCommit {
    id: String,
    tree: String,
    parents: Vec<String>,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
    message: String,
    time: i64,
}

use nostr_sdk::EventBuilder;

async fn create_event_with_custom_tags(
    keys: &Keys,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> Result<Event> {
    let mut builder = EventBuilder::new(Kind::TextNote, content);

    for (tag_name, tag_values) in custom_tags {
        info!("tag_name={:?}", tag_name);
        info!("tag_values={:?}", tag_values);
        //pops &tag_values[0]
        let tag: Tag = Tag::parse([&tag_name, &tag_values[0]]).unwrap();
        builder = builder.tag(tag);
    }

    let unsigned_event = builder.build(keys.public_key()); // Build the unsigned event
    let signed_event = unsigned_event.sign(keys); // Sign the event
    Ok(signed_event.await?)
}

async fn create_event(
    keys: Keys,
    custom_tags: HashMap<String, Vec<String>>,
    content: &str,
) -> Result<()> {
    //let content = "Hello, Nostr with custom tags!";

    let signed_event = create_event_with_custom_tags(&keys, content, custom_tags).await?;
    info!("{}", serde_json::to_string_pretty(&signed_event)?);

    let client = Client::new(keys);

    // add some relays
    // TODO get_relay_list here
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://relay.snort.social").await?;

    // Connect to the relays.
    client.connect().await;

    // Publish the event to the relays.
    client.send_event(signed_event.clone()).await?;

    info!("{}", serde_json::to_string_pretty(&signed_event)?);
    info!("Event sent: {:?}", signed_event);

    Ok(())
}

fn serialize_commit(commit: &Commit) -> Result<String> {
    let id = commit.id().to_string();
    let tree = commit.tree_id().to_string();
    let parents = commit.parent_ids().map(|oid| oid.to_string()).collect();
    let author = commit.author();
    let committer = commit.committer();
    let message = commit
        .message()
        .ok_or(anyhow!("No commit message"))?
        .to_string();
    debug!("message: {:?}", message);
    let time = commit.time().seconds();
    debug!("time: {:?}", time);

    let serializable_commit = SerializableCommit {
        id,
        tree,
        parents,
        author_name: author.name().unwrap_or_default().to_string(),
        author_email: author.email().unwrap_or_default().to_string(),
        committer_name: committer.name().unwrap_or_default().to_string(),
        committer_email: committer.email().unwrap_or_default().to_string(),
        message,
        time,
    };

    let serialized = serde_json::to_string(&serializable_commit)?;
    debug!("serialized_commit: {:?}", serialized);
    Ok(serialized)
}

fn deserialize_commit<'a>(repo: &'a Repository, data: &'a str) -> Result<Commit<'a>> {
    let serializable_commit: SerializableCommit = serde_json::from_str(data)?;

    let oid = Oid::from_str(&serializable_commit.id)?;
    let commit_obj = repo.find_object(oid, Some(ObjectType::Commit))?;

    let commit = commit_obj.peel_to_commit()?;

    if commit.id().to_string() != serializable_commit.id {
        return Err(anyhow!("Commit ID mismatch during deserialization"));
    }

    Ok(commit)
}

fn generate_nostr_keys_from_commit_hash(commit_id: &str) -> Result<Keys> {
    let mut hasher = Sha256::new();
    hasher.update(commit_id.as_bytes());
    let hash = hasher.finalize();

    let mut padded_hash = [0u8; 32];
    padded_hash.copy_from_slice(&hash[..]);

    let secret_key = SecretKey::from_slice(&padded_hash)?;
    let keys = Keys::new(secret_key);

    Ok(keys)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    //parse keys from sha256 hash
    let keys =
        Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("first_tag".to_string(), vec!["first_value".to_string()]);
    custom_tags.insert("another_tag".to_string(), vec!["another_value".to_string()]);

    //send to create_event function with &"custom content"
    let signed_event = create_event(keys, custom_tags, &"custom content").await;
    info!("signed_event={:?}", signed_event);

    //initialize git repo
    let repo = Repository::discover(".")?;

    //gather some repo info
    //find HEAD
    let head = repo.head()?;
    let obj = head.resolve()?.peel(ObjectType::Commit)?;

    //read top commit
    let commit = obj.peel_to_commit()?;
    let commit_id = commit.id().to_string();
    //some info wrangling
    info!("commit_id: {}", commit_id);

    // commit based keys
    let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    info!("keys.secret_key(): {:?}", keys.secret_key());
    info!("keys.public_key(): {}", keys.public_key());

    //TODO config metadata

    //create nostr client with commit based keys
    //let client = Client::new(keys);
    let client = Client::new(keys.clone());
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.connect().await;

    //access some git info
    let serialized_commit = serialize_commit(&commit)?;
    info!("Serialized commit: {}", serialized_commit);

    let binding = serialized_commit.clone();
    let deserialized_commit = deserialize_commit(&repo, &binding)?;
    info!("Deserialized commit: {:?}", deserialized_commit);

    info!("Original commit ID: {}", commit_id);
    info!("Deserialized commit ID: {}", deserialized_commit.id());

    //additional checking
    if commit.id() != deserialized_commit.id() {
        info!("Commit IDs do not match!");
    } else {
        info!("Commit IDs match!");
    }

    //build git gnostr event
    let builder = EventBuilder::text_note(serialized_commit);

    //send git gnostr event
    let output = client.send_event_builder(builder).await?;

    //some reporting
    info!("Event ID: {}", output.id());
    info!("Event ID BECH32: {}", output.id().to_bech32()?);
    info!("Sent to: {:?}", output.success);
    info!("Not sent to: {:?}", output.failed);

    // Create a filter for the specific event ID
    // format
    // filter: Filter { ids: Some({EventId(76f7789cfe0b636222ef4825a9e3e2ac580d942bf7212655e5f5ee1161264870)}), authors: None, kinds: None, search: None, since: None, until: None, limit: None, generic_tags: {} }


    let mut generic_tag_set = BTreeSet::new();
    generic_tag_set.insert(String::from("A Dance With Dragons"));

    let mut generic_tags: BTreeMap<nostr::Alphabet, BTreeSet<String>> = BTreeMap::new();
    generic_tags.insert(Alphabet::C, generic_tag_set.clone());

    generic_tags.insert(Alphabet::A, generic_tag_set);
    //generic_tags.insert(Alphabet::B, Alphabet::B);

    let filter = Filter::new()
        .id(*output.id())
        .authors([keys.public_key()])
        .kinds([])
        .event(*output.id());
		{generic_tags};
    info!("filter: {:?}", filter);

    //
    //
    //

    //
    //
    //

    // Subscribe to the filter
    let opts = SubscribeAutoCloseOptions::default();
    let subscription_id = client.subscribe(vec![filter], Some(opts)).await?;
    info!("subscription_id: {:?}", subscription_id);
    info!("subscription_id.val: {:?}", subscription_id.val);
    info!("subscription_id.success: {:?}", subscription_id.success);
    info!("subscription_id.failed: {:?}", subscription_id.failed);

    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event {
            relay_url: ref relay_url,
            subscription_id: ref subsciption_id,
            event: ref output,
        } = notification
        {
            info!("subscription_id: {:?}", subscription_id);
            info!("subscription_id.val: {:?}", subscription_id.val);
            info!("subscription_id.success: {:?}", subscription_id.success);
            //info!("success: {:?}", success);
            //info!("event: {:?}", event);
        }
        if let RelayPoolNotification::Message {
            relay_url: _,
            message: message,
        } = notification
        {
            info!("subscription_id: {:?}", subscription_id);
            info!("subscription_id.val: {:?}", subscription_id.val);
            info!("subscription_id.success: {:?}", subscription_id.success);
            //info!("success: {:?}", success);
            //info!("event: {:?}", event);
        }
    }

    client.disconnect().await?;
    Ok(())
}
