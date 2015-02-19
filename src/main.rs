#![feature(box_syntax, collections, core, io, path, std_misc)]
extern crate irc;
extern crate openssl;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

#[cfg(not(test))] use data::world::World;
#[cfg(not(test))] use irc::client::server::{IrcServer, Server};
#[cfg(not(test))] use irc::client::server::utils::Wrapper;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let mut world = World::new();
    let server = IrcServer::new("config.json").unwrap();
    for message in server.iter() {
        let message = message.unwrap();
        println!("{}", message.into_string());
        let mut args: Vec<_> = message.args.iter().map(|s| &s[]).collect();
        if let Some(ref suffix) = message.suffix {
            args.push(&suffix[])
        }
        let source = message.prefix.unwrap_or(String::new());
        let mut token_store = Vec::new();
        func::process_world(&Wrapper::new(&server), &source[], &message.command[], &args[],
                            &mut token_store, &mut world).unwrap();
    }
}
