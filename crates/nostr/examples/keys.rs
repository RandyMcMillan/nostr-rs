// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

use std::{cmp::max, collections::HashMap, path::PathBuf};
use git2::{Repository, Error};

fn walk() -> Result<(), Error> {
    let mut mtimes: HashMap<PathBuf, i64> = HashMap::new();
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;
        println!("{:0>64?}", commit.id());
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
    let secret_key_from_bech32 =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key_from_bech32);
    println!("nsec:Public key: {}", keys.public_key());
    let secret_key =
        SecretKey::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);
    println!("parse:Public key: {}", keys.public_key());
    let keys = Keys::parse("0000000000000000000000000000000000000000000000000000000000000001")?;
    println!("01:Public key: {}", keys.public_key());
    let keys = Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")?;

    println!("e3:Public key: {}", keys.public_key());
    walk();
    Ok(())
}
