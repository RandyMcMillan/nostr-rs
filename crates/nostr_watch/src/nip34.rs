use std::io::Read;

use reqwest::Url;

pub(crate) fn nip34() {
    let url = Url::parse("https://api.nostr.watch/v1/nip/34").unwrap();
    let mut res = reqwest::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    tmp_string = tmp_string.replace("[", "");
    tmp_string = tmp_string.replace("]", "");
    let v: Vec<&str> = tmp_string.split(",").collect();
    let mut v_json: Vec<String> = vec![];
    let mut count = 1; //skip EVENT when indexing
    v_json.push(format!("[\"RELAYS\","));
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
}
