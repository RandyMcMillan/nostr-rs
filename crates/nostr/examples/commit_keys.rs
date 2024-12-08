// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

use git2::{Repository, Revwalk};

fn rev_walk() -> Result<(), git2::Error> {
    // Open the local repository
    let repo = Repository::open(".")?;

    // Create a new revwalk
    let mut revwalk = repo.revwalk()?;

    // Set the starting point for the walk (e.g., the HEAD commit)
    revwalk.push_head()?;

    // Filter commits by author
    //revwalk.push_filter(Some(&author_name));

    // Limit the number of commits
    //revwalk.set_max_count(10);

    // Reverse the order of commits
    //revwalk.set_sorting(git2::Sort::REVERSE);

    // Iterate over the commits
    for rev in revwalk {
        let commit = rev?;
        let oid = commit.id();
        println!("{}", oid);
    }

    Ok(())
}

fn main() -> Result<()> {
    // Random keys
    let keys = Keys::generate();
    let public_key = keys.public_key();
    let secret_key = keys.secret_key();

    println!("Public key: {}", public_key);
    println!("Public key bech32: {}", public_key.to_bech32()?);
    println!("Secret key: {}", secret_key.to_secret_hex());
    println!("Secret key bech32: {}", secret_key.to_bech32()?);

    // Bech32 keys
    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);
    println!("Public key: {}", keys.public_key());

    Ok(())
}
