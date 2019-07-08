#[macro_use]
extern crate clap;
use clap::{App, AppSettings};

use kvs::KvStore;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            eprintln!("unimplemented");
            panic!();
        }
        ("get", Some(_matches)) => {
            eprintln!("unimplemented");
            panic!();
        }
        ("rm", Some(_matches)) => {
            eprintln!("unimplemented");
            panic!();
        }
        _ => unreachable!(),
    }
}
