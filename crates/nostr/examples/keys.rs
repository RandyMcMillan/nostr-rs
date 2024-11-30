// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use git2::Repository;
use nostr::prelude::*;
use nostr::Keys;

fn walk() -> Result<()> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    let mut parent_privkey =
        Keys::parse("0000000000000000000000000000000000000000000000000000000000000001")?;
    #[allow(unused_assignments)]
    let mut parent_pubkey = parent_privkey.public_key();
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        println!();
        parent_pubkey = parent_privkey.public_key();
        println!("parent_pubkey  hash:{}", parent_pubkey);
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;
        let pk = format!("{:0>64?}", commit.id());
        println!("padded commit  hash:{}", pk);
        let keys = Keys::parse(pk.clone())?.clone();
        println!("pubkey from  padded:{}", keys.public_key());
        parent_privkey = Keys::parse(pk)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut keys: Keys;
    // Bech32 keys
    let secret_key_from_bech32 =
        SecretKey::from_bech32("nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36")?;
    keys = Keys::new(secret_key_from_bech32);
    println!("key: {}", keys.public_key());
    let secret_key =
        SecretKey::parse("nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36")?;
    keys = Keys::new(secret_key);
    println!("key: {}", keys.public_key());
    keys = Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")?;
    println!("key: {}", keys.public_key());
    let _ = walk();
    Ok(())
}
