use std::io::Read;
use std::process;

use reqwest::{Error, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

pub(crate) fn online() {
    let url = Url::parse("https://api.nostr.watch/v1/online").unwrap();
    let mut res = reqwest::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    tmp_string = tmp_string.replace("[", "");
    tmp_string = tmp_string.replace("]", "");
    let v: Vec<&str> = tmp_string.split(",").collect();
    let mut v_json: Vec<String> = vec![];
    let mut v_relay: Vec<Relay> = vec![];
    let mut count = 1; //skip EVENT when indexing
    v_json.push(format!("[\"EVENT\","));
    for relay in v {
        //print!("{{\"{:}\":{:}}},", count, relay);
        v_json.push(format!("{{\"{:}\":{:}}},", count, relay));
        count += 1;
    }
    v_json.push(format!("{{\"{}\":\"wss://relay.gnostr.org\"}}", count));
    v_json.push(format!("]"));
    let titles = v_json.iter().map(|relay| relay).collect::<Vec<&String>>();
    for t in titles {
        print!("{}", t);
    }

    //let relay: Relay = serde_json::from_str(&tmp_string).expect("REASON");
    //println!("relay: {:?}", relay);

    process::exit(0);
    let request_url = "https://jsonplaceholder.typicode.com/photos";
    let mut response = reqwest::get(request_url).unwrap();
    let json = response.json::<Vec<Photo>>().unwrap();
    println!("{} photos", json.len());
    let titles = json
        .iter()
        .map(|photo| &photo.title)
        .collect::<Vec<&String>>();
    println!("titles: {:?}", titles)
}

#[derive(Deserialize, Debug)]
struct Relay {
    count: u16,
    relay: String,
}

#[derive(Deserialize, Debug)]
struct User {
    #[serde(rename = "userId")]
    user_id: u32,
    id: u32,
    title: String,
    completed: bool,
}

#[derive(Deserialize, Debug)]
struct Photo {
    #[serde(rename = "albumId")]
    album_id: u32,
    id: u32,
    title: String,
    url: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: String,
}
