// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;
use std::io::ErrorKind;
use std::process;

use anyhow::{anyhow, Result};
use git2::{Commit, ObjectType, Oid, Repository};
use nostr_sdk::prelude::*;
use nostr_sdk::EventBuilder;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::error::Category;
use serde_json::{Error, Result as SerdeJsonResult, Value};
use sha2::{Digest, Sha256};
use tokio::time::Duration;
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

fn byte_array_to_hex_string(byte_array: &[u8; 32]) -> String {
    let mut hex_string = String::new();
    for byte in byte_array {
        write!(&mut hex_string, "{:02x}", byte).unwrap(); // Use unwrap for simplicity, handle errors in production.
    }
    hex_string
}

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

    let opts = Options::new().gossip(true);
	let client = Client::builder().signer(keys.clone()).opts(opts).build();
    //let client = Client::new(keys);

    client.add_discovery_relay("wss://relay.damus.io").await?;
    client.add_discovery_relay("wss://purplepag.es").await?;
    //client.add_discovery_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion").await?;

    // add some relays
    // TODO get_relay_list here
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://e.nos.lol").await?;
    client.add_relay("wss://nos.lol").await?;

    // Connect to the relays.
    client.connect().await;

    // Publish a text note
    let pubkey = keys.public_key();

    info!("pubkey={}", keys.public_key());
    let builder = EventBuilder::text_note(
	   format!("Hello Worlds {}", pubkey),
    )
    .tag(Tag::public_key(pubkey));
    let output = client.send_event_builder(builder).await?;
    info!("Event ID: {}", output.to_bech32()?);

    info!("Sent to:");
    for url in output.success.into_iter() {
        info!("- {url}");
    }

    info!("Not sent to:");
    for (url, reason) in output.failed.into_iter() {
        info!("- {url}: {reason:?}");
    }

    // Publish a text note
    let test_author_pubkey =
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;

    info!("test_author_pubkey={}", test_author_pubkey);


    // Get events
    let filter_one = Filter::new().author(pubkey).kind(Kind::TextNote).limit(10);
    let events = client
        .fetch_events(vec![filter_one], Duration::from_secs(10))
        .await?;

    for event in events.into_iter() {
        info!("{}", event.as_json());
    }
    let filter_test_author = Filter::new().author(test_author_pubkey).kind(Kind::TextNote).limit(10);
    let events = client
        .fetch_events(vec![filter_test_author], Duration::from_secs(10))
        .await?;

    for event in events.into_iter() {
        info!("{}", event.as_json());
    }




    // Publish the event to the relays.
    client.send_event(signed_event.clone()).await?;

    info!("{}", serde_json::to_string_pretty(&signed_event)?);
    info!("Event sent:\n{:?}", signed_event);

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
    debug!("message:\n{:?}", message);
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
    //we serialize the commit data
    //easier to grab the commit.id
    let serializable_commit: SerializableCommit = serde_json::from_str(data)?;
    //grab the commit.id
    let oid = Oid::from_str(&serializable_commit.id)?;
    //oid used to search the repo
    let commit_obj = repo.find_object(oid, Some(ObjectType::Commit))?;
    //grab the commit
    let commit = commit_obj.peel_to_commit()?;
    //confirm we grabbed the correct commit
    if commit.id().to_string() != serializable_commit.id {
        return Err(anyhow!("Commit ID mismatch during deserialization"));
    }
    //return the commit
    Ok(commit)
}

fn generate_nostr_keys_from_commit_hash(commit_id: &str) -> Result<Keys> {
    let padded_commit_id = format!("{:0>64}", commit_id);
    info!("padded_commit_id:{:?}", padded_commit_id);
    let keys = Keys::parse(&padded_commit_id);
    Ok(keys.unwrap())
}

fn parse_json(json_string: &str) -> SerdeJsonResult<Value> {
    serde_json::from_str(json_string)
}

fn split_value_by_newline(json_value: &Value) -> Option<Vec<String>> {
    if let Value::String(s) = json_value {
        let lines: Vec<String> = s.lines().map(|line| line.to_string()).collect();
        Some(lines)
    } else {
        None // Return None if the Value is not a string
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

fn split_json_string(value: &Value, separator: &str) -> Vec<String> {
    if let Value::String(s) = value {
        s.split(&separator).map(|s| s.to_string()).collect()
    } else {
        vec![String::from("")]
    }
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
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);

    //send to create_event function with &"custom content"
    let signed_event = create_event(keys, custom_tags, &"custom content").await;
    info!("signed_event:\n{:?}", signed_event);

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
    info!("commit_id:\n{}", commit_id);
    let padded_commit_id = format!("{:0>64}", commit_id);

    // commit based keys
    let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    info!("keys.secret_key():\n{:?}", keys.secret_key());
    info!("keys.public_key():\n{}", keys.public_key());

    //TODO config metadata

    //create nostr client with commit based keys
    //let client = Client::new(keys);
    let client = Client::new(keys.clone());
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.connect().await;

    //access some git info
    let serialized_commit = serialize_commit(&commit)?;
    debug!("Serialized commit:\n{}", serialized_commit);

    let binding = serialized_commit.clone();
    let deserialized_commit = deserialize_commit(&repo, &binding)?;
    info!("Deserialized commit:\n{:?}", deserialized_commit);

    //access commit summary in the deserialized commit
    info!("Original commit ID:\n{}", commit_id);
    info!("Deserialized commit ID:\n{}", deserialized_commit.id());

    //additional checking
    if commit.id() != deserialized_commit.id() {
        debug!("Commit IDs do not match!");
    } else {
        debug!("Commit IDs match!");
    }

    let value: Value = parse_json(&serialized_commit)?;
    //info!("value:\n{}", value);

    // Accessing object elements.
    if let Some(id) = value.get("id") {
        info!("id:\n{}", id.as_str().unwrap_or(""));
    }
    if let Some(tree) = value.get("tree") {
        info!("tree:\n{}", tree.as_str().unwrap_or(""));
    }
    // Accessing parent commits (merge may be array)
    if let Some(parent) = value.get("parents") {
        if let Value::Array(arr) = parent {
            if let Some(parent) = arr.get(0) {
                info!("parent:\n{}", parent.as_str().unwrap_or("initial commit"));
            }
            if let Some(parent) = arr.get(1) {
                info!("parent:\n{}", parent.as_str().unwrap_or(""));
            }
        }
    }
    if let Some(author_name) = value.get("author_name") {
        info!("author_name:\n{}", author_name.as_str().unwrap_or(""));
    }
    if let Some(author_email) = value.get("author_email") {
        info!("author_email:\n{}", author_email.as_str().unwrap_or(""));
    }
    if let Some(committer_name) = value.get("committer_name") {
        info!("committer_name:\n{}", committer_name.as_str().unwrap_or(""));
    }
    if let Some(committer_email) = value.get("committer_email") {
        info!(
            "committer_email:\n{}",
            committer_email.as_str().unwrap_or("")
        );
    }

    //split the commit message into a Vec<String>
    if let Some(message) = value.get("message") {
        let parts = split_json_string(&message, "\n");
        for part in parts {
            info!("\n{}", part);
        }
        debug!("message:\n{}", message.as_str().unwrap_or(""));
    }
    if let Value::Number(time) = &value["time"] {
        info!("time:\n{}", time);
    }

    // // Accessing array elements.
    // if let Some(items) = value.get("items") {
    //     if let Value::Array(arr) = items {
    //         if let Some(first_item) = arr.get(0) {
    //             info!("First item: {}", first_item);
    //         }
    //         if let Some(second_item) = arr.get(1){
    //             info!("second item: {}", second_item.as_str().unwrap_or(""));
    //         }
    //     }
    // }

    //TODO

    //build git gnostr event
    let builder = EventBuilder::text_note(serialized_commit);

    //send git gnostr event
    let output = client.send_event_builder(builder).await?;

    //some reporting
    println!("Event ID: {}", output.id());
    println!("Event ID BECH32: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    client.disconnect().await?;
    Ok(())
}
