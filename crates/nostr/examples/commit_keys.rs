// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[deny(warnings)]
use clap::Parser;
use git2::{Error, Oid, Repository, Revwalk};

#[derive(Parser)]
struct Args {
    #[structopt(name = "topo-order", long)]
    /// sort commits in topological order
    flag_topo_order: bool,
    #[structopt(name = "date-order", long)]
    /// sort commits in date order
    flag_date_order: bool,
    #[structopt(name = "reverse", long)]
    /// sort commits in reverse
    flag_reverse: bool,
    #[structopt(name = "not")]
    /// don't show <spec>
    flag_not: Vec<String>,
    #[structopt(name = "spec", last = true)]
    arg_spec: Vec<String>,
}

fn pad_commit_hash(commit_hash: &str, desired_length: usize) -> String {
    format!("{:0>width$}", commit_hash, width = desired_length)
}

fn run(args: &Args) -> Result<(), git2::Error> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;

    let base = if args.flag_reverse {
        git2::Sort::REVERSE
    } else {
        git2::Sort::NONE
    };
    revwalk.set_sorting(
        base | if args.flag_topo_order {
            git2::Sort::TOPOLOGICAL
        } else if args.flag_date_order {
            git2::Sort::TIME
        } else {
            git2::Sort::NONE
        },
    )?;

    let specs = args
        .flag_not
        .iter()
        .map(|s| (s, true))
        .chain(args.arg_spec.iter().map(|s| (s, false)))
        .map(|(spec, hide)| {
            if spec.starts_with('^') {
                (&spec[1..], !hide)
            } else {
                (&spec[..], hide)
            }
        });
    for (spec, hide) in specs {
        let id = if spec.contains("..") {
            let revspec = repo.revparse(spec)?;
            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                return Err(Error::from_str("merge bases not implemented"));
            }
            push(&mut revwalk, revspec.from().unwrap().id(), !hide)?;
            revspec.to().unwrap().id()
        } else {
            repo.revparse_single(spec)?.id()
        };
        push(&mut revwalk, id, hide)?;
    }

    for id in revwalk {
        let id = id?;
        println!("{}", id);
        let padded_hash = pad_commit_hash(&id.to_string(), 64);
        println!("{}", padded_hash);
    }
    Ok(())
}

fn push(revwalk: &mut Revwalk, id: Oid, hide: bool) -> Result<(), Error> {
    if hide {
        revwalk.hide(id)
    } else {
        revwalk.push(id)
    }
}

fn main() -> Result<()> {
    let repo = Repository::open(".")?;

    // Get the OID of the HEAD commit
    let head = repo.head()?;
    let oid = head.target().unwrap();

    // Print the OID as a string
    println!("                        {}", oid.to_string());
    let padded_hash = pad_commit_hash(&oid.to_string(), 64);
    println!("{}", padded_hash);

    let args = Args::parse();
    match run(&args) {
        Ok(()) => {
            println!("Ok");
        }
        Err(e) => println!("error: {}", e),
    }

    let _ = run(&args);

    // Random keys
    //let keys = Keys::generate();
    let keys = Keys::parse(&padded_hash);
	let binding = keys?;
	let secret_key = binding.secret_key();

	let public_key = binding.public_key();
    //let secret_key = keys?.secret_key();

    println!("Public key: {}", public_key);
    println!("Public key bech32: {}", public_key.to_bech32()?);
    println!("Secret key: {}", secret_key.to_secret_hex());
    println!("Secret key bech32: {}", secret_key.to_bech32()?);

    // Bech32 keys
    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);
    //println!("Public key: {}", keys.public_key());

    Ok(())
}
