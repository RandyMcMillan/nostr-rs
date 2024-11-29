use reqwest::Error;
use serde::{Deserialize, Serialize};

//use serde_json::{Result, Value};

pub(crate) fn rest() {
    let request_url = "https://jsonplaceholder.typicode.com/todos/1";
    //    println!("{}", request_url);
    let mut response = reqwest::get(request_url).unwrap();

    //    println!("resp: {:?}", response);
    let json = response.json::<User>().unwrap();
    println!("json: {:?}", json);

    //    let users: Vec<User> = response.json()?;
    //    println!("{:?}", users);
    //    Ok(())

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
