use online::online;
mod online;
use rest::rest;
mod rest;
pub fn run(name: &str) {
    match name {
        "rest" => rest(),
        "online" => online(),
        _ => panic!("Invalid option"),
    };
}
