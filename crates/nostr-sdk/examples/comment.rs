// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Key
    ///
    #[clap(short, long, value_parser, default_value = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")]
    privkey: String
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::generate();
    let client = Client::builder().signer(keys).build();

    client.add_relay("wss://relay.damus.io/").await?;
    client.add_relay("wss://relay.primal.net/").await?;

    client.connect().await;

    let event_id =
        EventId::from_bech32("note1hrrgx2309my3wgeecx2tt6fl2nl8hcwl0myr3xvkcqpnq24pxg2q06armr")?;
    let events = client
        .fetch_events(vec![Filter::new().id(event_id)], None)
        .await?;

    let comment_to = events.first().unwrap();
    let builder = EventBuilder::comment("This is a reply", comment_to, None, None);

    let output = client.send_event_builder(builder).await?;
    println!("Output: {:?}", output);

    Ok(())
}
