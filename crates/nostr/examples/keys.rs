// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp::max;
use std::collections::HashMap;
use std::path::PathBuf;

use bitcoin::key::Parity;
use git2::{Error, Repository};
use nostr::prelude::*;

fn walk() -> Result<()> {
    let mut mtimes: HashMap<PathBuf, i64> = HashMap::new();
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    let mut parent_privkey =
        Keys::parse("0000000000000000000000000000000000000000000000000000000000000001")?;
    let mut parent_pubkey = parent_privkey.public_key();
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        //println!("");
        //println!("parent_privkey hash:{:?}", parent_privkey);
        println!("");
        parent_pubkey = parent_privkey.public_key();
        println!(
            "parent_pubkey  hash:{}",
            parent_privkey.public_key()
        );
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;
        let pk = format!("{:0>64?}", commit.id());
        println!("padded commit  hash:{}", pk);
        let keys = Keys::parse(pk.clone())?.clone();
        println!("pubkey from    hash:{}", keys.public_key());
        parent_privkey = Keys::parse(pk)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    // Random keys
    let keys = Keys::generate();
    let public_key = keys.public_key();
    let secret_key = keys.secret_key();

    //println!("Public key: {}", public_key);
    //println!("Public key bech32: {}", public_key.to_bech32()?);
    //println!("Secret key: {}", secret_key.to_secret_hex());
    //println!("Secret key bech32: {}", secret_key.to_bech32()?);

    // Bech32 keys
    let secret_key_from_bech32 =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key_from_bech32);
    println!("nsec:Public key: {}", keys.public_key());
    let secret_key =
        SecretKey::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);
    //println!("parse:Public key: {}", keys.public_key());
    let keys = Keys::parse("0000000000000000000000000000000000000000000000000000000000000001")?;
    //println!("01:Public key: {}", keys.public_key());
    let keys = Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")?;

    //println!("e3:Public key: {}", keys.public_key());
    let _ = walk();
    Ok(())
}
