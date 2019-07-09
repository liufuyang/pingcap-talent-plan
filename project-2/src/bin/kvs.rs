#[macro_use]
extern crate clap;

use clap::{App, AppSettings};

use kvs::KvStore;
use kvs::KvsError;
use kvs::Result;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            let value = matches.value_of("VALUE").expect("VALUE argument missing");

            let mut store = KvStore::open("./")?;

            store.set(key.to_owned(), value.to_owned())
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            let store = KvStore::open("./")?;

            match store.get(key.to_owned())? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }

            Ok(())
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            let mut store = KvStore::open("./")?;

            match store.remove(key.to_owned()) {
                Err(KvsError::NO_KEY_ERROR) => {
                    println!("Key not found");
                    Err(KvsError::NO_KEY_ERROR)
                }
                _ => Ok(()),
            }
        }
        _ => unreachable!(),
    }
}
