use clap::{App, AppSettings, SubCommand};
use nostr_watch::run;
fn main() {
    let matches = App::new("nostr_watch")
        .author("Nostr-SDK Developers")
        .version("v0.0.1")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("rest"))
        .subcommand(SubCommand::with_name("online"))
        .get_matches();

    match matches.subcommand_name() {
        Some(name) => run(name),
        None => panic!("no subcommand"),
    }
}
