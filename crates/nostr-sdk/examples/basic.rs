use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::collections::HashMap;
use nostr_sdk::prelude::*;

//use nostr_sdk::Client;
//use reqwest::Client as ReqwestClient;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BlockHeight {
  height: u32,
}
#[derive(Debug, Deserialize)]
struct BlockHash {
  hash: String,
}

#[deny(warnings)]
use clap::Parser;
use nostr::prelude::*;

#[derive(Parser)]
struct Args {
    #[structopt(
        name = "secret",
        long,
        default_value = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e"
    )]
    /// Nostr secret key
    secret: String,
    #[structopt(name = "username", long, default_value = "nostr-rs user")]
    /// Nostr username
    username: String,
    #[structopt(name = "displayname", long, default_value = "nostr-rs user")]
    /// Nostr display name
    displayname: String,
    #[structopt(name = "about", long, default_value = "nostr-rs user")]
    /// Nostr about string
    about: Option<String>,
    #[structopt(
        name = "picture",
        long,
        default_value = "https://robohash.org/nostr-rs"
    )]
    /// picture url
    picture: Option<String>,
    #[structopt(name = "banner", long, default_value = "https://robohash.org/nostr-rs")]
    /// banner url
    banner: Option<String>,
    #[structopt(name = "nip05", long, default_value = "username@example.com")]
    /// nip05
    nip05: Option<String>,
    #[structopt(name = "lud16", long, default_value = "pay@yukikishimoto.com")]
    /// lud16
    lud16: Option<String>,
}

fn run(args: &Args) -> Result<()> {
    let metadata = Metadata::new()
        .name(args.username.clone())
        .display_name(args.displayname.clone())
        .about(args.about.clone().unwrap())
        .picture(Url::parse(&args.picture.clone().unwrap())?)
        .banner(Url::parse(&args.banner.clone().unwrap())?)
        .nip05(args.nip05.clone().unwrap())
        .lud16(args.lud16.clone().unwrap());

    let keys = Keys::parse(&args.secret);

    let event: Event = EventBuilder::metadata(&metadata)
        .sign_with_keys(&keys?)
        .unwrap();

    // Convert client nessage to JSON
    let json = ClientMessage::event(event).as_json();
    println!("{json}");

    return Ok(());
}

async fn parse_args() -> Result<()> {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }

    Ok(())
}

async fn block_tips_hash() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://mempool.space/api/blocks/tip/hash")
        .await?
        .text()
        .await?;
    println!("blocks_tips_hash()={resp:#?}");
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {






    let _ = parse_args().await;
    let _ = block_tips_hash().await;

    let keys = reqwest::get("https://mempool.space/api/blocks/tip/hash").await?
    .text().await?;
   
    println!("{:?}", &keys);
    let keys = Keys::parse(&keys.clone());
    println!("{:?}", &keys);

    // Configure client to use proxy for `.onion` relays
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    let connection: Connection = Connection::new()
        .embedded_tor()
        //.proxy(addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);

    // Create new client with custom options.
    // Use `Client::new(signer)` to construct the client with a custom signer and default options
    // or `Client::default()` to create one without signer and with default options.
    let client = nostr_sdk::Client::with_opts(keys.expect("REASON").clone(), opts);

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    //client.add_relay("ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion").await?;
    
    // Add read relay
    client.add_read_relay("wss://relay.nostr.info").await?;

    // Connect to relays
    client.connect().await;

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com")
        .custom_field("custom_field", "my value");

    // Update metadata
    client.set_metadata(&metadata).await?;

    // Publish a text note
    let builder = EventBuilder::text_note("My first text note from rust-nostr!");
    client.send_event_builder(builder).await?;

    // Create a POW text note
    let builder = EventBuilder::text_note("POW text note from nostr-sdk").pow(20);
    client.send_event_builder(builder).await?; // Send to all relays
    // client.send_event_builder_to(["wss://relay.damus.io"], builder).await?; // Send to specific relay




    Ok(())
}
