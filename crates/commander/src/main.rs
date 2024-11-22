use clap::{App, AppSettings, SubCommand};
use commander::run;
fn main() {
    let matches = App::new("nostr-commander")
        .author("Nostr-SDK Developers")
        .version("v0.0.1")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("rest"))
        //.subcommand(SubCommand::with_name("rest"))
        .get_matches();

    match matches.subcommand_name() {
        Some(name) => run(name),
        None => panic!("no subcommand"),
    }
}
