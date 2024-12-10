// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#[deny(warnings)]
use clap::Parser;
use nostr::prelude::*;
use std::error::Error;

#[derive(Parser)]
struct Args {
    #[structopt(name = "username", long, default_value = "nostr user")]
    /// Nostr username
    username: String,
    #[structopt(name = "displayname", long, default_value = "nostr user")]
    /// Nostr display name
    displayname: String,
    #[structopt(name = "about", long, default_value = "nostr user")]
    /// Nostr about string
    about: Option<String>,
    #[structopt(name = "picture", long, default_value = "https://robohash.org/0")]
    /// picture url
    picture: Option<String>,
    #[structopt(name = "banner", long, default_value = "https://robohash.org/0")]
    /// banner url
    banner: Option<String>,
    #[structopt(name = "nip05", long, default_value = "username@example.com")]
    /// nip05
    nip05: Option<String>,
    #[structopt(name = "lud16", long, default_value = "pay@yukikishimoto.com")]
    /// lud16
    lud16: Option<String>,
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let metadata = Metadata::new()
        .name(args.username.clone())
        .display_name(args.displayname.clone())
        .about(args.about.clone().unwrap())
		.picture(Url::parse(&args.picture.clone().unwrap())?)
		.banner(Url::parse(&args.banner.clone().unwrap())?)
        .nip05(args.nip05.clone().unwrap())
		.lud16(args.lud16.clone().unwrap());

    println!("{}", metadata.as_json());

    //let event: Event = EventBuilder::metadata(&metadata).sign_with_keys(&keys)?;

    // New text note
    //let event: Event = EventBuilder::text_note("Hello from rust-nostr").sign_with_keys(&keys)?;

    // New POW text note
    //let event: Event = EventBuilder::text_note("POW text note from rust-nostr").pow(20).sign_with_keys(&keys)?;

    // Convert client nessage to JSON
    //let json = ClientMessage::event(event).as_json();
    //println!("{json}");

    return Ok(());
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
