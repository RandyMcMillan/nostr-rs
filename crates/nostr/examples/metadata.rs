// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
#[deny(warnings)]
use clap::Parser;
use std::error::Error;

#[derive(Parser)]
struct Args {

    #[structopt(name = "username", long)]
    /// Nostr username
    username: String,
    #[structopt(name = "displayname", long)]
    /// Nostr display name
    displayname: String,
    #[structopt(name = "about", long)]
    /// Nostr about string
    about: String,
    #[structopt(name = "picture", long)]
    /// picture url
    picture: String,
    #[structopt(name = "banner", long)]
    /// banner url
    banner: String,
    #[structopt(name = "nip05", long)]
    /// nip05
    nip05: String,
    #[structopt(name = "lud16", long)]
    /// lud16
    lud16: String,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com");

    println!("{}", metadata.as_json());

    //let event: Event = EventBuilder::metadata(&metadata).sign_with_keys(&keys)?;

    // New text note
    //let event: Event = EventBuilder::text_note("Hello from rust-nostr").sign_with_keys(&keys)?;

    // New POW text note
    //let event: Event = EventBuilder::text_note("POW text note from rust-nostr").pow(20).sign_with_keys(&keys)?;

    // Convert client nessage to JSON
    //let json = ClientMessage::event(event).as_json();
    //println!("{json}");

    return Ok(())
}


fn main() -> Result<()> {

    let args = Args::parse();
    match run(&args) {
        Ok(()) => {
            println!("Ok");
        }
        Err(e) => println!("error: {}", e),
    }

    Ok(())
}
