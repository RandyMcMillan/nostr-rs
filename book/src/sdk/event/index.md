## Event

<!--
## borrow.rs

## builder.rs

## error.rs

## id.rs

## kind.rs

## mod.rs

## tag/

## unsigned.rs
//
-->
<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

<!--
```rust,ignore
{{#include ../../../snippets/rust/src/event/json.rs:deserialize}}
```
-->

### [Event](https://docs.rs/nostr/latest/nostr/event/struct.Event.html)

```rust,ignore
    pub struct Event {
        pub id: EventId,
        pub pubkey: PublicKey,
        pub created_at: Timestamp,
        pub kind: Kind,
        pub tags: Tags,
        pub content: String,
        pub sig: Signature,
    }
```

### [EventBuilder](https://docs.rs/nostr/latest/nostr/event/builder/struct.EventBuilder.html)


https://docs.rs/nostr/latest/nostr/event/kind/index.html

[nostr::event::kind](https://docs.rs/nostr/latest/nostr/event/kind/index.html)

[EventBuilder::new](<https://docs.rs/nostr/latest/src/nostr/event/builder.rs.html?new#154>)
```rust,ignore

    let secret_key =
        SecretKey::from_hex("deadbeef...abcdef0123456789")?;
    let keys = Keys::new(secret_key);

    let event_builder = EventBuilder::new(
        Kind::TextNote,
        "Hello, Nostr!".to_string(), // Content of the event
        [], // Tags (optional)
    );

    let tags = vec![
        Tag::Event(EventId::from_hex("abcdef...0123456789")?),
        Tag::PubKey(keys.public_key()),
        Tag::Identifier("my_identifier".to_string()),
        Tag::Hashtag("nostr".to_string()),
        Tag::Custom(vec!["custom".to_string(), "value".to_string()]),
        Tag::Reference("https://example.com".to_string()),
        Tag::RelayMetadata(vec!["wss://relay.example.com".to_string(), "read".to_string()]),
    ];

    let event_builder = EventBuilder::new(
        Kind::TextNote,
        "Hello, Nostr with tags!".to_string(),
        tags
    );


```


### [EventBorrow](https://docs.rs/nostr/latest/nostr/event/borrow/struct.EventBorrow.html)


```rust,ignore
use nostr_sdk::prelude::*;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate a new keypair and an event
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Hello, BorrowedEvent!".to_string(), []).to_event(&keys)?;

    // Create a BorrowedEvent from the Event
    let borrowed_event: BorrowedEvent = event.as_borrowed();

    // Access properties of the BorrowedEvent
    println!("Borrowed Event ID: {}", borrowed_event.id);
    println!("Borrowed Event Pubkey: {}", borrowed_event.pubkey);
    println!("Borrowed Event Kind: {}", borrowed_event.kind);
    println!("Borrowed Event Content: {}", borrowed_event.content);
    println!("Borrowed Event Tags: {:?}", borrowed_event.tags);
    println!("Borrowed Event Sig: {}", borrowed_event.sig);
    println!("Borrowed Event Created At: {}", borrowed_event.created_at);

    //You can also convert back to an owned event if needed.
    let owned_event: Event = borrowed_event.to_owned();

    println!("Owned Event ID: {}", owned_event.id);

    Ok(())
}

```

```rust,ignore
use nostr_sdk::prelude::*;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate a new keypair (or load an existing one)
    let keys = Keys::generate(); // Or load from a hex string: Keys::from_str("your_hex_private_key")?;

    // Create a new event builder
    let event_builder = EventBuilder::new(
        Kind::TextNote, // Event kind (e.g., text note, metadata update, etc.)
        "Hello, Nostr!".to_string(), // Content of the event
        [], // Tags (optional)
    );

    // Sign the event
    let event = event_builder.to_event(&keys)?;

    // Print the event (for demonstration)
    println!("Event: {}", serde_json::to_string_pretty(&event)?);
    println!("Event ID: {}", event.id);
    println!("Public Key: {}", keys.public_key());

    // You would typically publish the event to relays here.
    // Example (requires a relay URL):
    /*
    let client = Client::new(&keys);
    client.add_relay("wss://relay.example.com").await?;
    client.connect().await;
    client.publish_event(event).await?;
    client.disconnect().await?;
    */

    Ok(())
}


```

</section>

## builder.md

## id.md

## index.md

## json.md

## kind.md

## tag.md


## Event JSON

### Deserialization

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/json.rs:deserialize}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/json.py:deserialize}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/json.ts:deserialize}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/src/main/kotlin/event/Json.kt:deserialize}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Json.swift:deserialize}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/json.dart:deserialize}}
```

</section>
</custom-tabs>

### Serialization

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/json.rs:serialize}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/json.py:serialize}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/json.ts:serialize}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/src/main/kotlin/event/Json.kt:serialize}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Json.swift:serialize}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/json.dart:serialize}}
```

</section>
</custom-tabs>

### Full example

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/json.rs:full}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/json.py:full}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/json.ts:full}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/src/main/kotlin/event/Json.kt:full}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Json.swift:full}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/json.dart:full}}
```

</section>
</custom-tabs>

