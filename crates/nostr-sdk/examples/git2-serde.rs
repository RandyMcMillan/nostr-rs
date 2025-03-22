// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::{anyhow, Result};
use git2::{Commit, ObjectType, Oid, Repository};
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};

use std::borrow::Cow;
use std::collections::HashMap;
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
        //let tag = Tag::parse(TagKind::custom(String::from("owned")+String::from("")));
        let tag = Tag::parse(format!("{:?}{:?}",String::from("owned").chars(), String::from("").chars()).chars());
        builder = builder.tag(tag?);
    }

    let unsigned_event = builder.build(keys.public_key()); // Build the unsigned event
    let signed_event = unsigned_event.sign(keys); // Sign the event
    Ok(signed_event.await?)
}

async fn create_event() -> Result<()> {
    let keys = Keys::generate();
    let content = "Hello, Nostr with custom tags!";
    let mut custom_tags = HashMap::new();
    custom_tags.insert(
        "my_custom_tag".to_string(),
        vec!["value1".to_string(), "value2".to_string()],
    );
    custom_tags.insert("another_tag".to_string(), vec!["single_value".to_string()]);

    let signed_event = create_event_with_custom_tags(&keys, content, custom_tags).await?;
    println!("{}", serde_json::to_string_pretty(&signed_event)?);

    let client = Client::new(keys);

	client.add_relay("wss://relay.damus.io").await?;
	client.add_relay("wss://relay.snort.social").await?;

    // Connect to the relays.
    client.connect().await;


   // Publish the event to the relays.
    client.send_event(signed_event.clone()).await?;

    println!("Event sent: {:?}", signed_event);



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
    let time = commit.time().seconds();

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


    let signed_event = create_event().await;
	println!("{:?}", signed_event);

    // Publish the event to the relays.
    //client.send_event(signed_event.clone()).await?;

    //println!("Event sent: {}", signed_event.id());


    let repo = Repository::discover(".")?;
    let head = repo.head()?;
    let obj = head.resolve()?.peel(ObjectType::Commit)?;
    let commit = obj.peel_to_commit()?;

    let serialized_commit = serialize_commit(&commit)?;
    let commit_id = commit.id().to_string();

    let binding = serialized_commit.clone();
    let deserialized_commit = deserialize_commit(&repo, &binding)?;

    println!("Original commit ID: {}", commit_id);
    println!("Deserialized commit ID: {}", deserialized_commit.id());

    if commit.id() != deserialized_commit.id() {
        println!("Commit IDs do not match!");
    } else {
        println!("Commit IDs match!");
    }

    // Nostr Integration
    let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    println!("keys.secret_key(): {:?}", keys.secret_key());
    println!("keys.public_key(): {}", keys.public_key());
    let client = Client::new(keys);
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.connect().await;

    let builder = EventBuilder::text_note(serialized_commit);
    let output = client.send_event_builder(builder).await?;

    println!("Event ID: {}", output.id());
    println!("Event ID BECH32: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    // Create a filter for the specific event ID
    let filter = Filter::new().id(*output.id());
    println!("filter: {:?}", filter);

    // Subscribe to the filter
    //let opts = SubscribeAutoCloseOptions::default().filter(FilterOptions::ExitOnEOSE);
    let opts = SubscribeAutoCloseOptions::default();
    let subscription_id = client.subscribe(vec![filter], Some(opts)).await?;
    println!("subscription_id: {:?}", subscription_id);
    //println!("subscription_id: {:?}", subscription_id.val);
    //println!("subscription_id: {:?}", subscription_id.val);

    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event {
            relay_url: _,
            subscription_id: _,
            event: _,
        } = notification
        {
            // 'event' is a Box<Event>
            //println!("subscription_id: {:?}", subscription_id);
            //println!("Received event: {:?}", event);
            // Access event data: event.id, event.pubkey, event.content, etc.
        }
    }

    //// Wait for the event
    //let mut event_receiver = client.notifications();
    //while let Ok(notification) = event_receiver.recv().await {
    //	 if let RelayPoolNotification::Event { relay_url: _, subscription_id: _, event: EVENT } = notification {
    //      //if let RelayPoolNotification::Event(_url, subscription_id, event) = notification {
    //          if EVENT == *output {
    //              println!("Found event: {:?}", EVENT);
    //              //client.unsubscribe(subscription_id).await; //Unsubscribe after finding the event.
    //              client.disconnect().await?;
    //              return Ok(());
    //      }
    //    }
    //}

    println!("Event not found.");
    client.disconnect().await?;

    //let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    //let client = Client::new(keys);

    //client.add_relay("wss://relay.damus.io").await?;
    ////client.add_relay("wss://nostr.wine").await?;
    ////client.add_relay("wss://relay.rip").await?;

    //client.connect().await;

    //// Publish a text note
    //let builder = EventBuilder::text_note("Hello world");
    //let output = client.send_event_builder(builder).await?;
    //println!("Event ID: {}", output.id().to_bech32()?);
    //println!("Sent to: {:?}", output.success);
    //println!("Not sent to: {:?}", output.failed);

    //// Create a text note POW event to relays
    //let builder = EventBuilder::text_note("POW text note from rust-nostr").pow(20);
    //client.send_event_builder(builder).await?;

    //// Send a text note POW event to specific relays
    //let builder = EventBuilder::text_note("POW text note from rust-nostr 16").pow(16);
    //client
    //    .send_event_builder_to(["wss://relay.damus.io", "wss://relay.rip"], builder)
    //    .await?;

    Ok(())
}
