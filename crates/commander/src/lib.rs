use rest::rest;
mod rest;
pub fn run(name: &str) {
    match name {
        "rest" => rest(),
        //"rest" => rest(),
        _ => panic!("Invalid option"),
    };
}
